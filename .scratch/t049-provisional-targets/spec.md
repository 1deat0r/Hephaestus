# T-049 spec - R-076: label performance targets provisional until measured

Status: ready-for-agent

## Problem Statement

R-076 (M4): performance targets are provisional until measured —
MASTER_SPEC §26's budgets ("provisional engineering budgets ... not
achieved results") must never be displayable as achieved benchmarks.
AT-076 negative: rendering an unimplemented feature with a p95 target
must not come out of the dossier or UI as an achieved benchmark. No
target/claim/benchmark concept exists in src (grep-verified); the
release verifier that enforces R-077/078/080 has no R-076 check.

## Requirements trace

- R-076/AT-076 (OBLIGATIONS; enforcement: Protected release verifier;
  contract M0, runtime M4). Source: MASTER_SPEC §26.
- Continuity: quantity-only measurements stay R-103's; ScopeLabel
  stays R-092/093's; this gate adds claims, not numbers-in-place.

## Design

Extend `release` (the named verifier):

- `PerformanceClaim` enum:
  - `ProvisionalTarget { feature, metric_label, target_value }` — a
    §26 budget; by construction not an achieved result;
  - `MeasuredBenchmark { feature, metric_label, measured_value, unit,
    benchmark_receipt, reference_machine }` — §26's "measured on a
    recorded reference machine".
- `ReleasePacket.performance_claims: Vec<PerformanceClaim>`.
- `assemble_release` check: every `MeasuredBenchmark` must carry a
  non-empty receipt, measured value, and reference machine, else
  `ReleaseBlock::UnmeasuredTargetDisplayed { feature }`. Provisional
  targets may be recorded.
- Display funnel: `achieved_benchmarks(&packet)` returns only
  `MeasuredBenchmark` claims — provisional targets cannot pass through
  the single achieved-presentation path (dossier/UI consume this
  funnel; wiring callers is composition, out of scope).

## Acceptance Criteria

1. **AT-076 negative**: packet recording a provisional p95 target for
   an unimplemented feature → `assemble_release` Ok (legal record) and
   `achieved_benchmarks` does NOT contain it (cannot display as
   achieved).
2. **Forged achieved**: `MeasuredBenchmark` with an empty receipt (or
   empty reference machine) → `assemble_release` refuses with
   `UnmeasuredTargetDisplayed`.
3. **Positive**: a fully-pinned `MeasuredBenchmark` passes the
   verifier and appears in the achieved funnel.
4. Glossary rows: Provisional target, Achieved benchmark.
5. Gates green (`make ci` + `make doc-check`), allowlist +3 + reseal,
   tests cite R-076/AT-076.

## Out of Scope

- Workspace/dossier view wiring (composition contract — callers use
  the funnel).
- Measuring §26's numeric budgets (qualification work, not labeling).
- Changes to Measurements (R-103), ScopeLabel (R-092/093), dossier
  records.
- Committing or pushing (rule 5).

## Further Notes

Grill: `.scratch/t049-provisional-targets/grill.md`.
