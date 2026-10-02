# 01: Sandbox provider core — isolation, attestation, supervision

**What to build:** The bwrap-backed `SandboxProvider`: minimal-bind,
network-off, env-scrubbed isolation with byte-capped stdio, a Rust
wall-timeout supervisor, and `attest()` before every run that denies on a
missing tool or an invalid spec — so R-059/R-110's static half has a home
and the first boundary tests can go red-then-green.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] `IsolationSpec` (declared tools, env allowlist, binds, wall/cpu/
      memory/nproc/fsize limits, network Off-only, output caps) with
      positive-limit validation; any non-Off network policy → `Unsupported`
- [x] bwrap argv per spec Q3: `--unshare-all`, `--die-with-parent`,
      `--new-session`, explicit minimal binds (no whole-root bind, home and
      /tmp never bound), tmpfs `/work`, `--clearenv` + allowlist
- [x] `attest()` runs before every run: bwrap `--version` executable,
      spec self-consistent; failure → `AttestationFailed`/`InvalidSpec`
      and **no spawn** (AT-110 static half)
- [x] Wall timeout: spawn, deadline poll, kill + reap; `Timeout` error;
      resource limits via `prlimit` wrapper
- [x] stdout/stderr captured with byte caps (truncation flagged)
- [x] Boundary tests green: TCP+DNS fail inside; host secret env var
      invisible; `/usr` read-only, `/work` writable; `../` traversal cannot
      write host; undeclared argv refused pre-spawn (deny) — test headers
      cite R-059/AT-059 (+R-110 where attestation)
- [x] Zero new dependencies

## Comments

2026-09-30T22:53:24Z — Done; all ACs verified by the suites (sandbox_deny 13, sandbox_run 9, launch unit 2) and green gates (255 workspace tests, doc-check 0, make ci 0 with the new files allowlisted).

Closed 2026-10-02 — work landed earlier; verified green: sandbox_deny 17/17 and sandbox_run green; bwrap gates verified in CI.
