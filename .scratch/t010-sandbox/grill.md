# Grill — T-010: isolated Python workers (SandboxProvider)

Self-interview (Phase 2, perpetual-directive cycle 7). Frontier emptied in
one round.

Goal: *isolated Python workers behind a SandboxProvider capability contract —
bwrap namespaces, minimal bind mounts, env scrub, network off by default,
declared tools only, safe output collection, resource metering, boundary
attestation (R-059/AT-059, R-110/AT-110).*

## Evidence gathered

- Environment: `/usr/bin/bwrap` works (`--ro-bind / --unshare-net` smoke
  OK); `unprivileged_userns_clone=1`; `unshare --user` OK; `prlimit`,
  `setpriv` present; Python 3.14.7. **bwrap has no rlimit flags** — resource
  limits come from a `prlimit` wrapper. bwrap does have `--clearenv`,
  `--unshare-all`, `--share-net` (the only network knob).
- MASTER_SPEC:389 — "Process boundaries alone are insufficient… document
  sandbox limitations and **test path traversal, symlinks, environment-
  variable leakage, subprocess escape, network exfiltration, dependency
  installation, and denial-of-service behavior**" — this is the test matrix.
  :400/R-110 — verify effective configuration before each dispatch; deny on
  attestation failure. :333 — TaskSpec carries tools, isolation profile,
  timeout, resource limits, output schema. :445 — `workers/python sandboxed
  scientific workers`.
- OBLIGATIONS: R-059/AT-059 (Contract M0, **Runtime M1**) — positive: bounded
  worker + collected outputs + attested mounts/env/descriptors/subprocess/
  network limits. R-110/AT-110 — action: change mounts / probe symlinks,
  sockets, DNS routes, dependency hooks; expected: dispatch denied on
  attestation or effective-boundary failure.
- `python/hephaestus_workers/__init__.py` — T-001 skeleton: "no tools, no
  sandbox, no execution path yet (T-010+)" — the payload's home.
- T-009's `TaskExecutor` is the integration seam (grill Q9).

## Round 1

❓ **Q1 Scope cut**: in — `sandbox` module (Rust): `SandboxProvider` trait,
bwrap-based implementation, `IsolationSpec`, attestation, wall-timeout
supervisor, prlimit resource wrapper, declared-tools enforcement, safe
output collection; python `hephaestus_workers.runner` job protocol; a
`SandboxExecutor` adapter implementing T-009's `TaskExecutor`. Out:
network *brokering* (M1 is network-OFF only — anything else is
`Unsupported`, fail-closed; AT-110's "allowed brokered routes" belongs to a
later broker), GPU/cgroup quotas (rlimits only), persistence of workdirs,
the mission compiler that will *produce* IsolationSpecs (callers construct
them). *source: plan T-010 + threat-model reading of :389/:400.*

❓ **Q2 Location**: `crates/hephaestus/src/sandbox/` (SandboxProvider is a
core interface per MASTER_SPEC:410; parallel to policy/budget/scheduler);
payload in `python/hephaestus_workers/` (plan: "workers/python sandboxed
scientific workers" — the repo's `python/` package). *source: MASTER_SPEC
:410/:445.*

❓ **Q3 Isolation construction (the security core)**: **minimal binds —
never `--ro-bind / /`** (that would expose `~/.ssh`, defeating R-059):
`--unshare-all` (user/mount/pid/ipc/uts/net/cgroup), `--die-with-parent`,
`--new-session`, explicit `--ro-bind` set: `/usr`, `/lib`, `/lib64`
(globbed by existence), `/etc/ld.so.cache` + `/etc/python*` if present,
`/proc` via `--proc`, `--dev /dev`, tmpfs `/work` (job + outputs), repo
worker package `--ro-bind` at a fixed path, `--clearenv` then `--setenv`
for an explicit allowlist (`PATH=/usr/bin`, `HOME=/work`,
`HEPHAESTUS_SANDBOX=1`), `--chdir /work`. **Host home/tmp never bound** ⇒
secret test = `$HOME/.ssh` unreadable inside. Network: `--unshare-net`
always (only supported policy). *source: :389 + R-059 + capability probe.*

❓ **Q4 Timeout & rlimits**: supervisor in Rust — spawn, poll with
`SystemTime` deadline, `kill` then reap (clock is executor-side, same
discipline as T-009's executor-enforced timeouts). Resource limits via
`prlimit --as= --cpu= --nproc= --fsize=` wrapping the bwrap invocation.
Defaults: memory 512 MiB, cpu 10 s, nproc 64, fsize 64 MiB, wall 30 s — all
overridable per spec, all validated > 0. *source: probe (no bwrap rlimit)
+ plan "resource metering".*

❓ **Q5 Declared tools**: `IsolationSpec.declared_tools: Vec<String>` —
`argv[0]` (basename) must appear in the list or the provider refuses
(`UndeclaredTool`) *before* spawning; the runner payload itself
(`python3` + fixed module path) is implicitly declared. No shell, no
`bash -c` string execution: argv is passed as a vector, never through a
shell. *source: plan "declared tools" + :333 TaskSpec.*

❓ **Q6 Safe output collection**: worker writes results under `/work/out/`
only; the provider copies back **from that directory only**, refusing
symlinks (`symlink_metadata().is_symlink()` → `OutputRefused`), capping
file count, per-file and total bytes (`OutputTruncated`), and ignoring
everything else in `/work`. stdout/stderr captured through pipes with byte
caps (truncation flagged). Workdir destroyed after collection.
*source: :389 symlink/exfil test list.*

❓ **Q7 Attunement to R-110**: `SandboxProvider::attest(&spec) ->
Result<Attestation, SandboxError>` runs **before every run**: bwrap present
and executable (spawn `bwrap --version`), spec self-consistent (network must
be Off; limits positive; declared tools non-empty; binds stay inside the
allowed set — no user-home binds), and returns an `Attestation` (tool
version, exact argv preview, flags). `run()` calls `attest()` first and
**denies** on failure. AT-110's *effective* probes (symlinks/egress) are
exercised as payload-level tests (the boundary tests), since in-process
static attestation cannot observe a mutated host mount namespace.
*source: AT-110 text + honest split between static attestation and
effective-boundary tests.*

❓ **Q8 Python worker protocol**: `hephaestus_workers.runner` reads
`/work/job.json` (`{tool: "module:function", input: <json>, limits…}`),
validates `tool` against the job's declared list (defense in depth),
invokes the function with the input, writes `/work/out/result.json` +
any bytes the function places under `/work/out/`, exits 0/1. M1 ships two
declared demo tools in-package (identity echo, failing tool for error
paths). No network calls ever (policy is network-off anyway). *source:
plan + Q3 layout.*

❓ **Q9 Scheduler integration**: `SandboxExecutor` implements T-009's
`TaskExecutor` — `Task` maps to a job (declared tool + input from task id /
inline payload; timeout_ms → wall deadline; cost unchanged), outcomes map:
exit 0 → `Succeeded{cost}`, nonzero → `Failed`, wall-kill → `TimedOut`,
provider error (attest/undecclared) → `Failed{reason}`; `run_batch` loops
(trivial batching stays a worker-efficiency concern). One e2e test: a
scheduler DAG runs through `SandboxExecutor` with budget settle. *source:
T-009 seam + plan "develop against mock executor" inversion.*

❓ **Q10 CI reality**: GitHub ubuntu runner may lack bwrap → the gates job
gains `sudo apt-get install -y bubblewrap` (gate-file edit: sealed, ADR-024
citation at eventual commit; decision row now). Tests **hard-require**
bwrap: absence = test failure, never a skip (green must mean sandboxed —
"subprocess is not a completed sandbox"). Local machine has bwrap ✓.
*source: honest-green requirement + gate mechanics from runs 4-6.*

❓ **Q11 Tests (the :389 matrix)**: deny-first `tests/sandbox_deny.rs` +
behavioral `tests/sandbox_run.rs` — (1) network: TCP connect + DNS fail
inside; (2) env: host secret env var invisible; (3) mounts: write to `/usr`
fails, `/work` writable; (4) path traversal: `../` from `/work` cannot write
host; (5) symlink: planted symlink in `/work/out` refused on collection;
(6) subprocess: nproc bomb fails; (7) dependency install: pip needs net →
fails; (8) DoS: infinite loop killed by wall/cpu, memory bomb dies
(rlimit-as or kill), fsize caps huge writes; (9) declared tools: undeclared
argv refused pre-spawn; (10) output caps truncate; (11) attest denies when
bwrap missing (PATH override) and on spec violations; (12) e2e worker
result collected; (13) scheduler e2e via SandboxExecutor. *source: :389 +
AT-059/:400.*

❓ **Q12 Docs/obligations**: R-059/AT-059 + R-110/AT-110 cited in test
headers; **Sandbox limitations documented in module docs** (:389's explicit
demand — list: user-namespace dependent, no seccomp hardening yet, no
brokered egress, host-kernel attack surface, not a multi-tenant boundary);
GLOSSARY +3 (sandbox provider, isolation spec, attestation) row-first; no
ADR (plan execution; the threat model lives in module docs); zero new Rust
deps (std + existing); python package gains no third-party deps.
*source: :389 + domain.md + plan.*

## Frontier status

Empty. No refusal-category item (no secrets handled — tests *assert absence*
of secrets; no external publication; no money).
