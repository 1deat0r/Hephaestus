# 02: Carry compute into the dossier and resolve the finding

**Status:** done
**Verify:** grep -q "compute-wall-time-metered" docs/RELEASE_DOSSIER_V1.md
**Covers:** 3, 4
**Blocked by:** 01 (needs measured compute)

## Scope

Regenerate the comparison and dossier with the compute
section. Replace the unmetered finding with a resolved entry
naming metered local compute. Keep currency absence explicit.

**Small tasks:**

1. [x] **S1** Regenerate reports and resolve the finding
   **Status:** done
   **Verify:** grep -q "compute-wall-time-metered" docs/RELEASE_DOSSIER_V1.md
   Regenerate both reports; resolve with successor named.
   **Micro-tasks:**
   1. [x] **M1** Regenerate and verify the marker
      **Verify:** grep -q "compute-wall-time-metered" docs/RELEASE_DOSSIER_V1.md
      atomic

## Comments

Blocked until 01 meters the runs.
