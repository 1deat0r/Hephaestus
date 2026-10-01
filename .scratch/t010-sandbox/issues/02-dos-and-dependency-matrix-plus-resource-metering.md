# 02: DoS/dependency matrix and resource metering

**What to build:** The rest of MASTER_SPEC:389's mandated failure classes as
enforced behavior: memory bombs, fork bombs, infinite loops, and huge writes
die at their declared bounds; dependency installation fails because egress
does not exist — with metering evidence (kill reason, wall time) in every
outcome.

**Blocked by:** 01 (sandbox provider core).

**Status:** ready-for-agent

- [x] `prlimit` bounds enforced: memory bomb dies (rlimit-as or kill),
      fork bomb fails at nproc, huge write fails at fsize, infinite loop
      dies at cpu/wall with `Timeout` — each test asserts a BOUNDED death
      (limits honored), not just failure
- [x] Dependency install: `pip install` inside the sandbox fails with a
      PINNED cause (test probes pip first: absent-interpreter module error,
      or pip-present => network-caused failure) and no host site-packages
      are visible to mutate
- [x] Outcomes carry metering: wall duration + kill/limit reason where
      applicable
- [x] DoS tests cite R-059/AT-059; resource limits validated positive at
      spec construction

## Comments

2026-09-30T22:53:24Z — Done; all ACs verified by the suites (sandbox_deny 13, sandbox_run 9, launch unit 2) and green gates (255 workspace tests, doc-check 0, make ci 0 with the new files allowlisted).
