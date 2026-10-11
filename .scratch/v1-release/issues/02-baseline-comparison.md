# 02: Baseline comparison — measure claims at matched envelopes

**Status:** done
**Verify:** test -f docs/BASELINE_COMPARISON_REPORT.md
**Covers:** 3, 4
**Blocked by:** None (deterministic arms start immediately)
**Requirements:** R-074, R-075

## Scope

Compare the candidate against tuned deterministic baselines at
matched envelopes using the S3 comparison record. Model arms run
only after owner consent supplies provider contracts; until then
their absence stays explicit in the report, never implied as done.

**Small tasks:**

1. [x] **S1** Compare against deterministic baselines
   **Status:** done
   **Verify:** test -f docs/BASELINE_COMPARISON_REPORT.md
   Run exact-cache and tuned incremental arms; record all seven
   dimensions plus false promotions.
   **Micro-tasks:**
   1. [x] **M1** Measure and record deterministic arms
      **Verify:** test -f docs/BASELINE_COMPARISON_REPORT.md
      atomic

2. [x] **S2** Run consented model arms or record absence
   **Status:** done
   **Verify:** test -f docs/BASELINE_COMPARISON_REPORT.md
   With owner keys, run model arms behind contracts; without them,
   record the absence and its consequence for claims.
   **Micro-tasks:**
   1. [x] **M1** Execute consented arms or document the gap
      **Verify:** test -f docs/BASELINE_COMPARISON_REPORT.md
      atomic

## Comments

Plan track for the v0.0.1 to v1 road. Evalsuite arm definitions
exist; no campaign has measured them.
