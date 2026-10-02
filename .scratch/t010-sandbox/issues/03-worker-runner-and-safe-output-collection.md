# 03: Worker runner and safe output collection

**What to build:** The Python payload and the collector: `runner` reads a
job, validates the tool against the declared list in-package (defense in
depth), invokes it, writes `/work/out/result.json`; the provider collects
only `/work/out` — refusing symlinks, capping count/per-file/total bytes —
so the AT-059 positive case (bounded worker + collected outputs) and the
symlink exfil class are real.

**Blocked by:** 01 (sandbox provider core).

**Status:** done

- [x] `hephaestus_workers.runner` protocol per spec Q8 with two in-package
      demo tools (echo, fail) and no third-party imports
- [x] Collection only from `/work/out`: symlink present → `OutputRefused`
      (nothing from it copied); oversize file/count/total → truncation
      flagged, never OOM/crash
- [x] E2E: sandbox runs the worker, `result.json` collected and parses;
      worker tool not on the declared list → worker exits nonzero
      (in-package validation), provider reports `WorkerFailed`
- [x] Traversal + symlink tests cite R-059/AT-059

## Comments

2026-09-30T22:53:24Z — Done; all ACs verified by the suites (sandbox_deny 13, sandbox_run 9, launch unit 2) and green gates (255 workspace tests, doc-check 0, make ci 0 with the new files allowlisted).

Closed 2026-10-02 — work landed earlier; verified green: sandbox_deny 17/17 and sandbox_run green; bwrap gates verified in CI.
