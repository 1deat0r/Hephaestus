# T-055 spec - R-084: held-out workload manifests + declared clustering unit

Status: ready-for-agent

## Problem Statement

R-084 (M3): workers must be denied access to held-out workload
manifests, and the analysis must retain the declared clustering unit
(AT-084 negative → required outcome). Neither exists: no workload
access gate and no clustering-unit field anywhere (grep-verified),
even though pilot variance is effectively computed across
repositories — the unit is undeclared and untransported.

## Requirements trace

- R-084/AT-084 (OBLIGATIONS; enforcement: Protected context oracle
  and evaluator; contract M0, runtime M3). Source: MASTER_SPEC §28.
- Continuity: confirmatory partition = held-out (R-101 contamination
  check already treats it as protected); pilot's repo-level
  arithmetic unchanged — only declared and carried.

## Design

Extend `pilot`:

- `AccessScope::{ ProtectedEvaluator, Worker }`;
  `WorkloadAccessError::{ HeldOutManifestDenied { repo_id },
  UnknownRepository { repo_id } }`;
  `request_workload_manifest(scope, &PilotPlan, repo_id)` —
  confirmatory assignment + Worker → denied; evaluator or
  non-held-out partition → Ok.
- `PilotPlan.clustering_unit: String` + `plan_pilot` refuses empty →
  `PlanError::MissingClusteringUnit`.
- `record_outcome(&PilotPlan, &[MissionOutcome])` →
  `VarianceEstimate.clustering_unit` copied verbatim from the plan
  (analysis retains the declared unit).

## Acceptance Criteria

1. **AT-084 negative**: Worker requests a confirmatory (held-out)
   repo manifest → `HeldOutManifestDenied` naming it;
   ProtectedEvaluator → Ok; Worker on a pilot repo → Ok.
2. Empty `clustering_unit` → `plan_pilot` refuses
   (`MissingClusteringUnit`).
3. `record_outcome` output carries the plan's clustering unit
   verbatim (retained, not inferred/zeroed).
4. Glossary rows: Held-out workload manifest, Clustering unit.
5. Gates green (`make ci` + `make doc-check`), allowlist +3 + reseal,
   tests cite R-084/AT-084.

## Out of Scope

- New oracle module (pilot hosts the gate; evaluator-side wiring is
  composition, recorded here).
- Changing variance arithmetic (repo-level math already matches the
  declared unit).
- Committing or pushing (rule 5).

## Further Notes

Grill: `.scratch/t055-context-oracle-workloads/grill.md`.
