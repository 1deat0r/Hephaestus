# 01: Characterize the sandbox flake with data

**Status:** done
**Verify:** test -f docs/SANDBOX_FLAKE_NOTE.md
**Covers:** 1, 2, 3, 4
**Blocked by:** None (can start immediately)

## Scope

Run sandbox_deny solo five times and under load three times.
Record load averages alongside. Write the note with a
data-bounded verdict. Persist raw outputs.

**Small tasks:**

1. [x] **S1** Measure and write the verdict
   **Status:** done
   **Verify:** test -f docs/SANDBOX_FLAKE_NOTE.md
   Run both series, log outputs, write the bounded verdict.
   **Micro-tasks:**
   1. [x] **M1** Measure, log, and write
      **Verify:** test -f docs/SANDBOX_FLAKE_NOTE.md
      atomic

## Comments

No source fix in this ticket. Data first, verdict bounded.
