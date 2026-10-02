# T-051 spec - R-082: label illustrative mechanisms and target values as unmeasured

Status: ready-for-agent

## Problem Statement

R-082 (M3): illustrative mechanisms and target values must be labeled
unmeasured — loading and exporting the packaged context-assembly
example must never render a synthetic fixture as real experimental
evidence (AT-082). Mission records carry `data_origin:
synthetic_fixture`, but `Dossier` has no label at all, so an export
built from the example ships with no unmeasured marker (grep-verified
gap).

## Requirements trace

- R-082/AT-082 (OBLIGATIONS; enforcement: Protected context oracle and
  evaluator; contract M0, runtime M3). Source: MASTER_SPEC §28.
- Continuity: mission-level `MissionDataOrigin` labeling unchanged;
  release-side target labels are T-049/R-076's; §15 dossier field list
  gains one required field.

## Design

Extend `dossier`:

- `UnmeasuredReason { SyntheticFixture, IllustrativeExample }`;
  `EvidenceLabel::{ Measured { run_receipt }, Unmeasured { reason,
  source } }`.
- `Dossier.evidence_label: EvidenceLabel` — required field (no
  unlabeled dossier exists).
- `export` gate: `Measured` with an empty receipt →
  `ExportError::MeasuredWithoutReceipt`; `Unmeasured` exports with
  the label serialized in the output.

## Acceptance Criteria

1. **AT-082 negative**: load `examples/software-mission.json`
   (asserts `data_origin == synthetic_fixture`), build the dossier as
   `Unmeasured { SyntheticFixture }`, export → output JSON contains
   both the unmeasured label and the synthetic-fixture reason (never
   renderable as real evidence).
2. Forged `Measured { run_receipt: "" }` → `export` refuses with
   `MeasuredWithoutReceipt`.
3. Honest `Measured` with a receipt exports and the receipt is in the
   output.
4. Glossary rows: Evidence label, Synthetic fixture.
5. Gates green (`make ci` + `make doc-check`), allowlist +3 + reseal,
   tests cite R-082/AT-082.

## Out of Scope

- Mission-level origin enums (exist), release-side targets (T-049),
  genesis MechanismRecord fields (covered by the dossier-level label
  over `mechanism_explanation`), oracle/evaluator wiring
  (composition).
- Committing or pushing (rule 5).

## Further Notes

Grill: `.scratch/t051-illustrative-unmeasured/grill.md`.
