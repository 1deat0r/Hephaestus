# 01: Guard committed reports against drift

**Status:** done
**Verify:** cargo test --test cli reproduce_v1_committed
**Covers:** 1, 2, 3, 4
**Blocked by:** None (can start immediately)

## Scope

Add one CLI test that regenerates pilot, baseline, and dossier
reports into temp dirs and compares deterministic sections
with the committed files. Byte identity for the pilot;
scoped replay for baseline and dossier.

**Small tasks:**

1. [x] **S1** Add the reproduction guard test
   **Status:** done
   **Verify:** cargo test --test cli reproduce_v1_committed
   Write the guard; it passes only when artifacts track code.
   **Micro-tasks:**
   1. [x] **M1** Write and green the guard
      **Verify:** cargo test --test cli reproduce_v1_committed
      atomic

## Comments

Guard addition, not behavior fix: reproduction already holds.
The test turns that fact into a checked gate.
