# 01: Pilot campaign — fly 20 missions, record variance and cost

**Status:** done
**Verify:** test -f docs/PILOT_CAMPAIGN_REPORT.md
**Covers:** 1, 2
**Blocked by:** None (can start immediately)
**Requirements:** R-102, R-103, R-104

## Scope

Fly a bounded pilot batch on fixtures first. Predeclare metrics
before the first mission. Record every run, failure, and cost
receipt. Prospective live missions wait on owner consent and stay
out of this ticket.

**Small tasks:**

1. [x] **S1** Fly the 20-mission fixture batch
   **Status:** done
   **Verify:** test -f docs/PILOT_CAMPAIGN_REPORT.md
   Predeclare metrics, run the batch end to end, keep raw receipts.
   **Micro-tasks:**
   1. [x] **M1** Predeclare metrics and run the batch
      **Verify:** test -f docs/PILOT_CAMPAIGN_REPORT.md
      atomic

2. [x] **S2** Record variance, cost, and failure modes
   **Status:** done
   **Verify:** test -f docs/PILOT_CAMPAIGN_REPORT.md
   Compute variance from actual runs; attach cost receipts and
   failure modes; note confirmatory readiness honestly.
   **Micro-tasks:**
   1. [x] **M1** Compute and record from run receipts
      **Verify:** test -f docs/PILOT_CAMPAIGN_REPORT.md
      atomic

## Comments

Plan track for the v0.0.1 to v1 road. T-027 machinery exists;
zero missions have flown.
