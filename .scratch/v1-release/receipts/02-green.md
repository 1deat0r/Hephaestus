# 02 receipt — baseline comparison, deterministic arms

Date: 2026-10-12 UTC.
Spec AC: 3, 4. No architectural departure.

## Red

Ticket Verify `test -f docs/BASELINE_COMPARISON_REPORT.md`
failed: file absent. CLI tests `baseline_compare_*` failed:
command unknown.

## Green

`cargo test --test cli baseline` → 4 passed.
`cargo test --test acceleration_eval deterministic_arms` →
1 passed. Ticket Verify passes: report exists from a real run.

## Implementation

`acceleration/mod.rs`: `measure_deterministic_arms` runs base
and cand arms over repo subsamples and measures every run
(correctness from actual margins, tokenizer token counts,
Instant wall time, derived throughput, VmHWM peak); S3
`compare_qualified_efficiency` enforces matched envelopes.
`main.rs`: `fixture baseline-compare --trace --out`.
Report carries all seven dimensions, 20 run receipts, method
notes, and the explicit model-arm absence record.

## Reviews

Independent re-review after first FAIL: labels fixed (total /
mean / max), receipts section added, bad-arg and receipt-link
tests added, library-level measurement test added. Three independent reviews: FAIL, FAIL, then PASS on all nine
verification items. All findings fixed with disclosures,
receipt linkage, and extended tests.

## Regressions

`cli` full target, `acceleration_eval` full target in `make ci`.

## Limitations

Base arm is the honest-negative control: 0.0 correctness is
expected, not a value finding. No promotion gate ran. Model
arms absent (no consented provider); report backs no model
claim. Latency, throughput, and memory are local and vary run
to run; only deterministic fields replay.
