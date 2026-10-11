# AE-07 S3 receipt — measure qualified efficiency

Date: 2026-10-12 UTC.
Requirement IDs: R-018, R-074, R-106, R-107, R-111. Spec AC: 21.
No architectural departure: no RUNTIME_DECISIONS entry needed.

## Red (missing behavior)

Command:
cargo test -p hephaestus --test architecture_independent_qualification s3_measurement

Result: compile failure (E0432): `RunMeasurement`,
`ArmMeasurement`, `ComparisonError`, and
`compare_qualified_efficiency` are absent from
`hephaestus::evalsuite`.

## Green (after implementation)

Command: same as above.
Result: 3 passed, 0 failed —
s3_measurement_compares_matched_arms,
s3_measurement_refuses_unmatched_and_invalid,
s3_measurement_ignores_run_order.
Full target: 9 passed (S1 and S2 stay green).
Runner: `python3 .scratch/architecture-efficiency/workflow.py verify
AE-07 S3` → PASS, exit 0.
Receipt: ae07-s3-workflow-verify.log.

## Implementation

`crates/hephaestus/src/evalsuite/mod.rs` — new efficiency
comparison: `RunMeasurement` carries correctness, promotion,
tokens, cost, latency, throughput, and peak memory;
`ArmMeasurement` pairs an envelope with runs;
`compare_qualified_efficiency` requires matched budget, tool
access, and model access, requires nonempty runs with finite
non-negative cost and throughput, then totals tokens, cost,
and latency, means throughput, maxes peak memory, fractions
correctness, and counts candidate runs promoted while
incorrect as false promotions. Run order never affects output.

## Regressions

`eval_suite` 7 passed; `acceleration_eval` 8 passed.
`make ci` and `make doc-check` run before landing.

## Limitations

Measures are caller-supplied; the comparison computes, never
observes. Tuning-record parity stays in `check_matched`, not
here. This completes AE-07 S1–S3 and the seven-TASK plan.
