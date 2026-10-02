# 02: Anomaly detection with the unit gate (R-020)

**What to build:** The anomaly operator with its contract: an anomaly is an
uncertainty-aware discrepancy between a prediction and an observation sharing
a unit and comparable conditions (MASTER_SPEC:144). A fake anomaly built from
incompatible units (R-020/AT-020 negative case) is rejected with a recorded
reason — zero opportunities emitted. A genuine in-unit discrepancy is emitted
with the discrepancy and its explicit uncertainty.

**Blocked by:** 01 (needs the seam, TraceRecord, Opportunity, RejectedCandidate).

**Status:** done

- [x] Prediction + observation, same unit, comparable conditions, discrepancy beyond stated uncertainty → anomaly opportunity with uncertainty stated
- [x] Unit mismatch → `rejected` with reason; nothing emitted (AT-020)
- [x] Missing prediction → `rejected` with reason (never "two isolated numbers from different conditions", MASTER_SPEC:144)
- [x] Incomparable conditions recorded as rejection reason

## Comments
Done 2026-10-01T06:19Z: 5 new tests in `discovery_pressure_points.rs` (red first: 4 failures), 9/9. Uncertainty band = documented fixed 50% relative gap; per-record uncertainty fields deferred as a later refinement.
