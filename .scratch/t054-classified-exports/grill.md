# T-054 grill - R-092: classified exports (own status, evidence, scope, reproduction state)

Goal: R-092/AT-092 (contract M0, runtime M4, MASTER_SPEC §31
readiness context) — negative: "Export exploratory, test-ready, and
validated records together." Required: "Each carries its own status,
evidence, scope, and reproduction state."

Existing surfaces (facts):
- lifecycle (T-024): `HypothesisState` includes Exploratory, TestReady,
  Assessed (the negative's three, modulo naming); `CandidateFields`
  four orthogonal statuses (R-046) — record-level classification
  EXISTS.
- dossier (T-023): `export(d: &Dossier)` — single-dossier path; no
  bundle export, no per-record classification gate; no R-092 cite.
- `ReproductionOutcome` (Reproduced | EnvironmentMismatch | Disagrees)
  already lives in dossier — the reproduction-state half.
- Test home: tests/dossier_export.rs (extend; allowlist +3 only).

## Q1 - What is the seam?
**A:** dossier-side bundle export: `ClassifiedRecord { record_id,
status: HypothesisState, evidence_ids: Vec<String>, scope: String,
reproduction: ReproductionOutcome }` + `export_bundle(&[...])`.
Status and reproduction are typed struct fields (cannot be omitted);
evidence and scope are the two stringly facts the gate enforces
non-empty — that is the required outcome, field by field.
(agent-default)

## Q2 - What does the negative case refuse?
**A:** A mixed-state bundle (exploratory + test-ready + validated)
exported WITHOUT per-record classification: any record with empty
evidence_ids → `BundleExportError::MissingEvidence { record_id }`;
empty scope → `MissingScope { record_id }`. Mixed states are legal
ONLY when each record carries all four facts — the negative is the
unclassified togetherness, not the mixing itself (required outcome
says each must carry its own, not that bundles are banned).
(agent-default)

## Q3 - Cross-module dependency?
**A:** dossier imports `crate::lifecycle::HypothesisState` (already
re-exported); lifecycle gains nothing and stays untouched. One-way
dossier→lifecycle, matching the dossier's existing consumption of
hypothesis versions. (agent-default)

## Q4 - Files/gates?
**A:** dossier/record.rs (ClassifiedRecord + error), dossier/mod.rs
(export_bundle + exports); extend tests/dossier_export.rs (header +
2 tests: unclassified mixed bundle refused naming records; classified
mixed bundle exports with per-record status distinct in JSON);
allowlist +3; reseal; glossary rows: Classified record, Reproduction
state. Citations R-092/AT-092. Red-first at export_bundle. No commit
(rule 5). (agent-default)
