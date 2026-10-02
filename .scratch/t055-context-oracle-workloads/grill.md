# T-055 grill - R-084: protect held-out workload manifests and retain the declared clustering unit

Goal: R-084/AT-084 (Protected context oracle and evaluator; contract
M0, runtime M3, MASTER_SPEC §28) — negative: "Attempt worker access
to held-out workload manifests." Required: "Access is denied and the
analysis retains the declared clustering unit."

Existing surfaces (facts):
- Zero workload-manifest or clustering code (grep: `workload` only in
  prose, `cluster` zero in code).
- pilot (T-027) is the analysis home: partitions BY REPOSITORY
  (TrainTune/Pilot/Confirmatory), `record_outcome` computes
  paired-difference variance "across repositories" — the clustering
  unit is EFFECTIVE but never declared or carried (no field anywhere).
- The confirmatory partition IS the held-out set (R-101
  contamination check: pilot outcomes drawn from confirmatory are
  already refused) — held-out workload manifest = a confirmatory
  repo's assignment/episodes.
- `reproduction::record_outcome` is a different, unrelated fn;
  pilot's is called from pilot_campaigns.rs only (3 sites) and
  `PilotPlan` is built in one helper — signature/field churn is
  contained.

## Q1 - What are the two seams?
**A:** (a) `request_workload_manifest(scope: AccessScope, plan,
repo_id)` with `AccessScope::{ProtectedEvaluator, Worker}`:
confirmatory-repo + Worker → `WorkloadAccessError::
HeldOutManifestDenied { repo_id }` (AT-084 negative); unknown repo →
`UnknownRepository`; otherwise Ok (the oracle side reads it).
(b) `PilotPlan.clustering_unit: String` (non-empty enforced by
`plan_pilot` → `PlanError::MissingClusteringUnit`) and
`record_outcome(&PilotPlan, outcomes)` returns `VarianceEstimate`
carrying `clustering_unit` verbatim — the analysis RETAINS the
declared unit (the existing repo-level arithmetic was already at that
unit; now it is declared and transported). (agent-default)

## Q2 - Why pilot and not a new oracle module?
**A:** The required outcome is about ANALYSIS retaining a declared
unit — the analysis lives in pilot; the enforcement note
("protected context oracle") is the evaluator-side composition
recorded in the spec, per the standing precedent (enforcement names
services that CALL the gate). No new module for one gate pair.
(agent-default)

## Q3 - Signature churn?
**A:** `record_outcome` gains the plan parameter (3 test call sites
updated; no other callers). `PilotPlan` literal: one helper.
VarianceEstimate gains `clustering_unit` (Default keeps serde/twin
tests stable). (agent-default)

## Q4 - Files/gates?
**A:** pilot/{mod,record}.rs (all allowlisted), extend
tests/pilot_campaigns.rs (allowlisted) → allowlist +3 (.scratch only);
reseal; glossary rows: Held-out workload manifest, Clustering unit.
Citations R-084/AT-084. Red-first at the access gate. No commit
(rule 5). (agent-default)
