# T-057 spec - R-007: evidence kinds distinct

Status: ready-for-agent

## Problem Statement

R-007 (M2): observations, assumptions, model judgments, and
derivations stay distinct, and no repetition of an unsupported model
statement promotes it to empirical evidence (AT-007 negative →
required outcome). The six-value schema enum exists but has zero
R-named tests and no runtime gate anywhere (run-24 re-audit).

## Requirements trace

- R-007/AT-007 (OBLIGATIONS; enforcement: Verification service;
  contract M0, runtime M2). Source: MASTER_SPEC §3.
- Continuity: kinds come from `contracts::generated::
  EvidenceEvidenceType` (schema = single source of truth); ingestion/
  span machinery (T-013) untouched.

## Design

Extend `knowledge`:

- `StatementIngestion { text, ingestions: usize, grounded_span:
  Option<Span> }`.
- `ClassifyError::UngroundedObservation { ingestions }`.
- `attest_evidence_class(declared: EvidenceEvidenceType,
  &StatementIngestion) -> Result<(), ClassifyError>`: Observation
  without a grounded span → error carrying the (seen, rejected)
  repetition count; any other declared kind → Ok — repetitions never
  change the standing of a model judgment.

## Acceptance Criteria

1. **AT-007 negative**: unsupported model statement declared as
   Observation, ingested repeatedly, no span → `UngroundedObservation`
   naming the count; declared ModelJudgment with the same input → Ok
   (kind stands).
2. Grounded Observation (span present) → Ok.
3. Distinctness: the four R-007 kinds round-trip through the
   generated enum pairwise-unequal (serde round-trip preserves
   identity; identical text under different kinds stays different).
4. Tests cite R-007/AT-007 (this ticket's tests mirror the AT
   verbatim — the one honest AT claim in the retrofit era).
5. Gates green; allowlist +4 (scratch×3 + new test file) + reseal.

## Out of Scope

Schema/generator edits, ingest/edge changes, six-vs-four kind
accounting beyond citing the four named (the enum carries all six;
the gate only restricts Observation promotion).

## Further Notes

Grill: `.scratch/t057-evidence-kind-distinctness/grill.md`.
