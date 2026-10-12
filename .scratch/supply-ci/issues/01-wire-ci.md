# 01: Wire supply checks into CI

**Status:** done
**Verify:** make supply-check
**Covers:** 1, 2, 3, 4
**Blocked by:** None (can start immediately)

## Scope

Add the Makefile target and the CI job with pinned tool
installs. Prove the target green locally. Keep `make ci`
membership unchanged.

**Small tasks:**

1. [x] **S1** Target plus CI job
   **Status:** done
   **Verify:** make supply-check
   Add target, add job, green locally, no ci changes.
   **Micro-tasks:**
   1. [x] **M1** Red-to-green wiring cycle
      **Verify:** make supply-check
      atomic

## Comments

Install cost lands on CI runners, not local gates.
