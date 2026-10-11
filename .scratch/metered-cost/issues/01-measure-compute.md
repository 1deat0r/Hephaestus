# 01: Measure wall and CPU time per comparison run

**Status:** done
**Verify:** cargo test -p hephaestus --test acceleration_eval metered_compute
**Covers:** 1, 2
**Blocked by:** None (can start immediately)
**Requirements:** R-103, R-104

## Scope

Clock each deterministic comparison run with wall and CPU
timers around the chain call. Carry per-arm totals into the
baseline report compute section. Degrade to explicit gaps
where clocks are unreadable.

**Small tasks:**

1. [x] **S1** Clock runs and report per-arm compute
   **Status:** done
   **Verify:** cargo test -p hephaestus --test acceleration_eval metered_compute
   Add the measurement, render the section, prove replay scope.
   **Micro-tasks:**
   1. [x] **M1** Red-to-green measurement cycle
      **Verify:** cargo test -p hephaestus --test acceleration_eval metered_compute
      atomic

## Comments

Closes the compute half of the dossier unmetered finding.
Currency metering stays out: no provider exists.
