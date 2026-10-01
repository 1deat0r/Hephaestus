# 04: Scheduler adapter, glossary, CI, and gate evidence

**What to build:** The integration that T-011 will stand on —
`SandboxExecutor` mapping sandbox runs onto T-009's outcomes — plus the
domain vocabulary, CI's bubblewrap install (honest green), documented
limitations, and a green gate suite.

**Blocked by:** 01 (sandbox provider core).

**Status:** ready-for-agent

- [x] `SandboxExecutor: TaskExecutor` — exit 0 → `Succeeded{cost}`,
      nonzero → `Failed{reason}`, wall-kill → `TimedOut`, attest/undeclared
      denial → `Failed{Sandbox:…}`; `run_batch` loops
- [x] E2E: a T-009 DAG (2 tasks, priced) runs through `SandboxExecutor`
      with budget reserve→commit intact; a timed-out task maps to
      `TimedOut`
- [x] GLOSSARY gains: sandbox provider, isolation spec, attestation
      (decision row written **first**)
- [x] Sandbox module docs list the real limitations verbatim per spec
      (MASTER_SPEC:389 demand)
- [x] `ci.yml` gates job installs `bubblewrap` (gate edit; seal
      regenerated; ADR citation required at eventual commit)
- [x] fmt / clippy `-D warnings` / `cargo test --workspace` /
      `make doc-check` / `make ci` green with evidence lines; new files
      staged + allowlisted; tests hard-require bwrap (no skips)

## Comments

2026-09-30T22:53:24Z — Done; all ACs verified by the suites (sandbox_deny 13, sandbox_run 9, launch unit 2) and green gates (255 workspace tests, doc-check 0, make ci 0 with the new files allowlisted).
