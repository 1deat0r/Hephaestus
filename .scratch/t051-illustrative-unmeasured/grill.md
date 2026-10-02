# T-051 grill - R-082: label illustrative mechanisms and target values as unmeasured

Goal: R-082/AT-082 (Protected context oracle and evaluator; contract
M0, runtime M3, MASTER_SPEC §28) — negative: "Load and export the
packaged context-assembly example." Required: "No synthetic fixture is
rendered as real experimental evidence."

Existing surfaces (facts):
- Packaged example: `examples/software-mission*.json` carry
  `"data_origin": "synthetic_fixture"` (42 markers) — mission-level
  labeling EXISTS (MissionDataOrigin enum, trust synthetic mode).
- `Dossier` has NO origin/label field — export serializes whatever it
  is given, so a dossier built from the example exports with no
  synthetic/unmeasured marker at all: THE gap.
- Dossier content includes `mechanism_explanation` (the statement's
  "mechanisms") and `results_json`/`analysis` (its "target values");
  the target-values half in release was T-049/R-076 — this ticket's
  half is the dossier render path.
- Tests home: tests/dossier_export.rs with a `dossier()` helper
  (extend, no new test file → allowlist +3 only).

## Q1 - What is the label?
**A:** Non-optional `Dossier.evidence_label: EvidenceLabel` with
`EvidenceLabel::Measured { run_receipt }` vs
`EvidenceLabel::Unmeasured { reason, source }` (`UnmeasuredReason`:
SyntheticFixture | IllustrativeExample). Non-optional = no unlabeled
dossier can exist (fail-closed labeling, §15 field list).
(agent-default)

## Q2 - Where is the gate?
**A:** `dossier::export`: a `Measured` label with an empty receipt →
new `ExportError::MeasuredWithoutReceipt` (a measured claim must
carry its version-bound receipt); `Unmeasured` labels export fine —
and the serialized JSON then ALWAYS carries the label, so no consumer
can read the export as real evidence without seeing why it is not.
(agent-default)

## Q3 - How does the negative case get pinned?
**A:** Test loads the packaged example (`examples/software-mission
.json`), asserts its `data_origin` is `synthetic_fixture`, builds the
dossier with `Unmeasured { SyntheticFixture, ... }`, exports, and
asserts the output JSON contains both the unmeasured label and the
synthetic-fixture reason; a forged `Measured { run_receipt: "" }`
dossier is refused by name. (agent-default)

## Q4 - Statement halves?
**A:** The dossier's `mechanism_explanation` and result/target fields
are exactly the statement's two nouns — ONE dossier-level label
covers both; release-side targets stay T-049's. No genesis/
MechanismRecord churn. (agent-default)

## Q5 - Files/gates?
**A:** dossier/record.rs (enum + field), dossier/mod.rs (export check
+ ExportError variant), tests/dossier_export.rs (helper update + 2
tests); allowlist +3 (.scratch/t051); reseal; glossary rows:
Evidence label, Synthetic fixture; citations R-082/AT-082. Red-first
at export. No commit (rule 5). (agent-default)
