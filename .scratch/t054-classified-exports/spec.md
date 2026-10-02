# T-054 spec - R-092: classified exports

Status: ready-for-agent

## Problem Statement

R-092 (M4): exporting exploratory, test-ready, and validated records
together is only legitimate when each record carries its OWN status,
evidence, scope, and reproduction state (AT-092 negative → required
outcome). Lifecycle states and orthogonal candidate fields exist
(T-024/T-046), but nothing gates a mixed bundle export — no
classification check exists on any export path (grep-verified).

## Requirements trace

- R-092/AT-092 (OBLIGATIONS; contract M0, runtime M4). Source:
  MASTER_SPEC §31 readiness context.
- Continuity: `HypothesisState` (lifecycle, T-024), `CandidateFields`
  (R-046), `ReproductionOutcome` (dossier, T-023) reused unchanged.

## Design

Extend `dossier`:

- `ClassifiedRecord { record_id, status: HypothesisState,
  evidence_ids: Vec<String>, scope: String, reproduction:
  ReproductionOutcome }` — status and reproduction are typed fields
  (structurally always present).
- `BundleExportError::{ MissingEvidence { record_id }, MissingScope {
  record_id } }`.
- `export_bundle(&[ClassifiedRecord]) -> Result<String,
  BundleExportError>`: empty evidence_ids → `MissingEvidence`; empty
  scope → `MissingScope`; else serialize the bundle (per-record four
  facts in the output).

## Acceptance Criteria

1. **AT-092 negative**: mixed-state bundle (Exploratory, TestReady,
   Assessed) with records missing evidence or scope → refused naming
   the offending record(s).
2. The same three states WITH evidence/scope/reproduction each →
   `Ok`, and the exported JSON shows distinct per-record statuses
   (mixed togetherness without collapse).
3. Scope cut honored: status/reproduction cannot be missing (typed
   fields — asserted by construction + a compile-visible test).
4. Glossary rows: Classified record, Reproduction state.
5. Gates green (`make ci` + `make doc-check`), allowlist +3 + reseal,
   tests cite R-092/AT-092.

## Out of Scope

- Lifecycle changes (states/fields reused as-is).
- Single-dossier `export` (unchanged — this is the bundle path).
- Committing or pushing (rule 5).

## Further Notes

Grill: `.scratch/t054-classified-exports/grill.md`.
