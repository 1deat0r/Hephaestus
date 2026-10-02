# T-052 spec - R-073: evaluate domain-only discovery

Status: ready-for-agent

## Problem Statement

R-073 (M4): the campaign benchmark must evaluate the generator on
independently originated opportunities and mechanisms — running the
full benchmark without supplying hypotheses is refused (AT-073
negative). The eval suite has four comparison arms and no admission
gate at all (grep: no discovery path, no R-073 citation), so a
benchmark can start with zero originated inputs today.

## Requirements trace

- R-073/AT-073 (OBLIGATIONS; enforcement: Protected campaign
  evaluator; contract M0, runtime M4). Source: MASTER_SPEC §25.
- Continuity: ArmKind / define_baseline / check_matched / ablation
  (T-026, R-074/R-075) untouched — this gate runs BEFORE arms.

## Design

Extend `evalsuite`:

- `OriginatedHypothesis { hypothesis_id, opportunity_id,
  mechanism_id }` — lineage proving independent origin.
- `BenchmarkError::{ NoSuppliedHypotheses, NotIndependentlyOriginated
  { hypothesis_id } }`.
- `begin_benchmark(&[OriginatedHypothesis]) ->
  Result<BenchmarkSession, BenchmarkError>`: empty slice →
  `NoSuppliedHypotheses`; any empty opportunity/mechanism id →
  `NotIndependentlyOriginated`; else
  `BenchmarkSession { hypothesis_ids }` (admitted cohort).

## Acceptance Criteria

1. **AT-073 negative**: `begin_benchmark(&[])` →
   `NoSuppliedHypotheses`.
2. **Required outcome**: fully-lineaged hypotheses → `Ok` session
   listing them; a hypothesis with an empty opportunity_id OR
   mechanism_id → `NotIndependentlyOriginated` naming it.
3. Session carries the admitted ids (observable admission evidence).
4. Glossary rows: Domain-only discovery, Independently originated.
5. Gates green (`make ci` + `make doc-check`), allowlist +3 + reseal,
   tests cite R-073/AT-073.

## Out of Scope

- New ArmKind values or arm machinery (T-026's).
- Campaign execution/measurement (admission gate only).
- Committing or pushing (rule 5).

## Further Notes

Grill: `.scratch/t052-domain-only-discovery/grill.md`.
