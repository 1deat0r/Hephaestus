# 01: Write the v1 gaps register

**Status:** done
**Verify:** test -f docs/V1_GAPS.md
**Covers:** 1, 2, 3, 4
**Blocked by:** None (can start immediately)

## Scope

Sweep the dossier, posture audit, and release track into
`docs/V1_GAPS.md`. One row per gap with owner, consent flag,
unblocker, and gated claim. No gap closes without evidence.

**Small tasks:**

1. [x] **S1** Sweep and write the register
   **Status:** done
   **Verify:** test -f docs/V1_GAPS.md
   Collect every gap, flag consent, write the file.
   **Micro-tasks:**
   1. [x] **M1** Collect, flag, and write
      **Verify:** test -f docs/V1_GAPS.md
      atomic

## Comments

Register only. Closing gaps is later work with evidence.
