# Qualification, lineage, and trust contract

Version 1.2 · 30 September 2026 · Normative supplement to master sections 13–16, 20–21, and 26. Requirements R-094–R-101 and R-112–R-114 apply. This defines future runtime obligations; included Python checks exercise selected metadata only.

## Protected trust context [R-094, R-095]

Candidate-supplied `valid`, `supported`, `qualified`, `approved`, and `independent_pass` fields MUST NOT establish authority. A protected service MUST authenticate receipt/grant/qualification issuers, validate their immutable signed or authenticated contents and revocation state, retrieve artifacts and verify their bytes, and establish current dependency and policy versions. Importing a record or setting its `trust_origin` is not authentication. Receipts MUST be bound to exact record versions and candidate, evaluator, analysis, evidence-snapshot, and policy identities. A digest without accessible verified bytes is insufficient. Each receipt also binds the exact assessed result/dossier payload and plan registration digest. Assessment payload hashing excludes only receipt-reference fields to prevent hash cycles; altering findings, conclusions or labels requires a new assessment.

`ValidationContext` in the reference code is an externally supplied test seam. It MUST NOT be populated from model output, the candidate bundle, or a user-editable cache. Trusted identity sets also require matching externally authenticated record-content hashes; freshness binds both version and immutable content. Passing synthetic context demonstrates metadata behavior, not real authentication or qualification. The runtime MUST reject fixture trust contexts in live promotion. The reference default is empty trust and therefore cannot promote a candidate. The schema includes five additional records: authorization grant, method qualification, experiment family, holdout access, and verification receipt. Each is an immutable domain record; only the protected authority can confer trust on it.

## Complete qualification path [R-094, R-095, R-096]

For every claim and mission guardrail needed for a label, the promotion service MUST resolve the exact hypothesis and executable plan, ensure blockers are resolved, and verify the applicable method qualification. It MUST check primary endpoint coverage, units, failed or missing controls, guardrails, raw artifacts, analysis artifacts, deviations, independent reproduction, and the final promotion assessment. Execution success alone does not establish a supported claim. Failed controls invalidate the affected inference; failed engineering guardrails can leave a mechanism informative while blocking the solution label.

Fixed-sample empirical qualification requires its registered design and sample-size rationale. Sequential qualification requires an independently qualified stopping method and family policy. Formal qualification requires the proposition, assumptions, proof/counterexample artifacts, and a trusted checker; deterministic qualification requires a declared exhaustive or bounded test and oracle scope. Formal or deterministic work uses the `nonstatistical` family and MUST NOT invent statistical sample sizes, alpha or confidence intervals. Registration freezes its proof/test obligations and the access ledger records protected evaluator-input access. Every design has immutable method/evaluator identity and explicit limitations.

The v1.1 reference profile deliberately checks a single hypothesis and qualifying result for a dossier, with frozen confirmatory registration, trusted receipts, endpoint/control/guardrail coverage, and externally verified dependencies. Unsupported multi-hypothesis promotion fails closed. Runtime composition can support additional profiles only after their claim aggregation and independence rules pass contract tests. Independent reproduction requires a protected assessment of personnel, generator, measurement pipeline, artifacts, and data dependencies; merely changing an issuer name does not establish independence.

## Lineage and applicability [R-097, R-098]

| Relationship | Required binding | Change consequence |
|---|---|---|
| Opportunity → mission | Same immutable mission version | New mission requires applicability assessment |
| Mechanism → opportunity | Exact opportunity version | New mechanism gets new lineage |
| Hypothesis → mechanism/opportunity/mission | Coherent chain; no cross-mission substitution | Recompile and reassess applicable evidence |
| Plan → hypothesis | Exact hypothesis, predictions, candidate, comparator, evaluator, analysis, snapshot | Material change requires new registered version |
| Result → plan/hypothesis | Exact execution and analysis identities, endpoint and unit coverage | Cannot relabel old observations as a new run |
| Dossier → hypotheses/results | Every result addresses a listed hypothesis in the dossier mission | Cross-mission reuse requires explicit applicability assessment |
| Grant → operation | Exact mission, operation, destination, artifact, policy, cost and expiry | Changed identity or revoked grant blocks dispatch |
| Cache/summary → inputs | Semantic task, source, record, prompt, model/tool, policy and environment dependencies | Relevant change invalidates reuse |

The runtime MUST maintain dependency edges from evidence, sources, grants, qualification, evaluator, and artifacts to conclusions. Corrections, retractions, quarantine, changed policy, and changed scope MUST mark affected current labels stale and block reuse pending assessment. Historical records remain intact. New versions MUST NOT inherit old approval or support automatically. An explicit signed applicability assessment can approve historical evidence reuse within named conditions; absent that supported contract, the v1.1 reference rejects stale/unknown versions.

## Authorization timing [R-099]

Authorization MUST be checked at compilation, immediately before dispatch, and before externally consequential effects. Grants include issue/expiry time, revocation state, operation, destination, artifact, capabilities, policy, and maximum cost. Mission capabilities and destination allowlists also apply. A standing local profile may mint scoped grants deterministically without a new user prompt. Time must come from the control plane, not the worker's creation timestamp. Concurrent cost reservation remains a separate transactional prerequisite.

Revocation stops new dispatch and effects; already completed effects are reconciled rather than represented as undone. Cancellation, checkpoint, and reconciliation behavior MUST be specified per capability. Dependency writes and redirects cannot bypass destination scope. Policy changes invalidate outstanding grants unless an explicit approved compatibility rule applies.

## Registration and access [R-100, R-101]

Registration binds all plan fields except the operational `status` and the registration envelope itself. v1.1 defines the hash profile as UTF-8 JSON with sorted object keys, compact separators, unescaped Unicode, and finite numbers, as implemented by `plan_digest`. Implementations MUST use shared canonicalization vectors; different JSON encoders or numeric representations must not silently produce equivalent-looking payloads. The schema version binds this profile. Any hash-profile migration requires a new version and cannot rewrite historical registrations.

The future service MUST freeze the plan before results/holdout access, make registered records immutable, and authenticate the registration event. The reference recomputes the payload digest but cannot authenticate its history. A sealed-data broker MUST create immutable access events and track feedback queries, family membership, selection lineage and data partitions across missions. Worker-supplied timestamps do not establish absence of prior access.

The initial campaign uses `bonferroni_fixed_family`: no more than the registered number of confirmatory endpoint tests across all member plans, family-wise alpha divided across that fixed family, and no detailed feedback until campaign closure. Related claims and guardrails use the predeclared joint error policy. Independent fresh confirmation may address selection on exploratory data only when independence is established. Family resets, renamed missions, and dataset aliases MUST NOT reset exposure. Sequential methods and adaptive holdout reuse are unavailable until separately qualified. Retired/overexposed data cannot confer confirmation; the next campaign uses fresh independent workloads.

## Label taxonomy [R-112]

`completed_investigation` includes informative negative and inconclusive outcomes. `validated_solution` requires scoped scientific/engineering/guardrail/reproduction qualification and a resolved scoped prior-art assessment; `known` prior art is allowed and displayed prominently as rediscovery. `validated_candidate` adds a protected claim-level difference assessment and verified claim-chart artifact; it permits `near_match` or `no_match_within_search_scope` only within the stated coverage. Neither implies global novelty or a legal conclusion. `known` is not eligible for the invention-candidate label. Scientific conclusions stay identical when only the prior-art label changes.

## Explicit state transitions [R-114]

| Record | Allowed progression | Blocking/revision rule |
|---|---|---|
| Opportunity | discovered → grounded → prioritized → explored → parked/closed | Reopening creates a version and reason |
| Hypothesis | exploratory → compiled → reviewed → test_ready → testing → assessed | Blocked/archive transitions preserve evidence; material revision restarts affected gates |
| Experiment plan | draft → ready → running → completed | Blocked plans cannot run; frozen payload cannot change |
| Task | pending → ready → running → completed/failed/cancelled | Retry creates a new attempt under the same declared effect semantics; unknown effects reconcile |
| Dossier | exploratory/planned → completed_investigation → validated_solution or validated_candidate | Promotion requires current protected assessment; invalidation creates a stale assessment event |

Transitions are validated control-plane commands with expected prior version, event ID, actor, and reason. Each transition has authorization, reservation, evidence and output prerequisites. No transition overwrites a record version. Migration MUST reject unsupported schema versions, preserve raw old records and hashes, and emit a migration receipt with old/new identities and semantic differences. The archived v1.0 schema/fixture are compatibility inputs, not a live 1.1 bypass.
