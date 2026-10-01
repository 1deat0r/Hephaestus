# Spec — T-010: isolated Python workers (SandboxProvider)

Status: ready-for-agent
Goal source: derived:roadmap (perpetual directive, cycle 7)
Grill record: `.scratch/t010-sandbox/grill.md` (12 questions, all self-answered)

## Problem Statement

M1 promises "durable local execution" including isolated workers, but the
worker package is a T-001 skeleton ("no tools, no sandbox, no execution
path"), and nothing can run untrusted generated code: there is no
SandboxProvider, no attestation, no resource metering, and no safe output
path. MASTER_SPEC:389 is explicit that process boundaries alone are not a
security claim and that the implementation must both document limitations
and *test* traversal, symlink, env-leak, subprocess, egress, dependency-
install, and DoS behavior — none of which exists. R-059/R-110 (Runtime M1)
have no runtime home, and T-009's executor seam has no real executor.

## Solution

1. **`sandbox` module (Rust)** — `SandboxProvider` capability contract with
   a bwrap implementation: minimal explicit bind mounts (never a whole-root
   bind), `--unshare-all` (network off — the only supported policy),
   `--clearenv` + explicit allowlist, tmpfs workdir, declared-tools argv
   enforcement (no shell), Rust wall-timeout supervisor, `prlimit` resource
   wrapper (memory/cpu/nproc/fsize), byte-capped stdio, output collection
   from `/work/out` only with symlink refusal and size caps, and
   `attest()` before every run that denies on missing tool or spec
   violations (AT-110 dispatch denial).
2. **`hephaestus_workers.runner` (Python)** — job protocol: read
   `/work/job.json`, validate the tool against the declared list, invoke,
   write `/work/out/result.json`; two in-package demo tools.
3. **`SandboxExecutor`** — implements T-009's `TaskExecutor` so a scheduler
   DAG runs real sandboxed work (the "integrated execution path" T-011
   needs), mapping exit/kill/provider-errors to task outcomes.
4. **Documented limitations** in module docs (:389's demand) + CI gains a
   bubblewrap install step (tests hard-require the tool — absence fails).

## User Stories

1. As a caller, I want an isolation profile whose root contains only
   explicitly bound paths, so that host secrets (`~/.ssh`, home, /tmp) are
   simply not mounted rather than "protected".
2. As a caller, I want network namespace isolation as the only supported
   policy, so that egress fails closed and "allowed brokered routes" cannot
   be claimed until a broker exists.
3. As a caller, I want the environment cleared and rebuilt from an
   allowlist, so that host variable leakage is a test, not a hope.
4. As a caller, I want argv[0] checked against declared tools before spawn,
   so that undeclared binaries never start.
5. As an operator, I want wall/CPU/memory/nproc/fsize bounds enforced by
   supervisor + prlimit, so that a runaway or fork bomb dies boundedly.
6. As an auditor, I want stdout/stderr byte-capped, so that output floods
   cannot exhaust the collector.
7. As an auditor, I want outputs collected only from `/work/out`, symlinks
   refused, and caps on count/size, so that exfiltration-by-output is a
   typed error.
8. As an auditor, I want `attest()` consulted before every run with denial
   on failure, so that R-110's "dispatch denied when attestation fails"
   holds for the static checks.
9. As a maintainer, I want the sandbox's real limitations documented in the
   module docs, so that the security claim never outruns the mechanism.
10. As a developer, I want a worker runner speaking a tiny JSON protocol
    with tool validated in-package too, so that defense in depth survives a
    provider bug.
11. As a reviewer, I want the :389 test matrix implemented as deny-first
    tests (traversal, symlink, env, subprocess, egress, dependency install,
    DoS ×3), so that every mandated failure class has a red-turned-green
    witness.
12. As a scheduler, I want `SandboxExecutor` to map outcomes (0 → success,
    nonzero → failed, wall-kill → timed out, provider denial → failed) so
    that T-009 needs no changes to run real work.
13. As a reviewer, I want R-059/R-110 and their AT ids in test headers, so
    traceability is checkable.
14. As an operator, I want CI to install bubblewrap, so that the honest-green
    suite runs the real sandbox remotely too.
15. As a user of the offline build, I want zero new Rust/Python
    dependencies, so that every gate keeps working unchanged.

## Implementation Decisions

- **Module layout:** `sandbox/` with `provider` (trait + errors + spec +
  attestation), `bwrap` (implementation + supervisor + prlimit wrapper),
  `executor` (TaskExecutor adapter); payload in `python/hephaestus_workers`.
- **Isolation argv** exactly per grill Q3 (minimal binds, unshare-all,
  clearenv+allowlist, tmpfs /work, die-with-parent, new-session); network =
  Off only → any other policy is `Unsupported`.
- **Limits/prlimit** per grill Q4; **declared tools** per Q5 (vector argv,
  never a shell); **outputs** per Q6; **attestation** per Q7 (static checks
  before run; effective-boundary probes live in the test matrix).
- **Worker protocol** per Q8; **SandboxExecutor mapping** per Q9; **CI
  bubblewrap install** per Q10 (gate edit sealed; ADR citation deferred to
  the eventual commit); **tests** the 13-case matrix per Q11; **docs**
  per Q12.
- **Errors:** one `SandboxError` enum (AttestationFailed, UndeclaredTool,
  Unsupported, Timeout, OutputRefused, OutputTruncated{..}, WorkerFailed{..},
  Io{..}) — every refusal maps to a typed outcome, never a panic.
- **No ADR; no requirements/schema/generated edits; zero new deps.**

## Testing Decisions

- Seam: `SandboxProvider::{attest, run}` + `SandboxExecutor` behaviors;
  tests observe only public outputs (exit/stdout/files/attestation/errors).
- Prior art: `tests/scheduler_run.rs` (deny-first + mock discipline),
  `tests/event_ledger.rs` (temp-dir helpers).
- The :389 matrix (grill Q11's 13 cases) lives in
  `tests/sandbox_deny.rs` (refusals) and `tests/sandbox_run.rs`
  (boundaries + e2e + scheduler integration). Tests hard-require bwrap —
  a missing tool fails the suite (never skips).
- Suite: fmt, clippy `-D warnings`, `cargo test --workspace`,
  `make doc-check`, `make ci` — evidence lines to the state LOG; new files
  staged + allowlisted (standing procedure).

## Out of Scope

- Network brokering / allowed routes (AT-110's brokered half — later);
  seccomp-bpf profiles; cgroup v2 quotas; GPU; multi-tenant hardening;
  persistence/caching of workdirs; mission-compiler production of
  IsolationSpecs; artifact-store integration of collected outputs (callers
  hold them).

## Further Notes

- Limitations (to be verbatim in module docs, :389): depends on
  unprivileged user namespaces (kernel/policy-dependent); no seccomp layer
  yet; no brokered egress — network is all-or-nothing off; host-kernel
  attack surface remains; NOT a multi-tenant boundary; rlimits are advisory
  against kernel OOM behavior at the edges.
- Limitation: static attestation cannot detect a host mount namespace
  mutated after startup — effective probes are tests, re-attestation per
  dispatch covers spec/tool drift only.
- `SandboxExecutor::run_batch` loops per task (batching efficiency inside
  the sandbox is future worker-side work).
