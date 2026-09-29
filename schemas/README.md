# Contract notes

The 1.2 schema contains 18 principal record types. An immutable reference is `{ "id": "HYP-CONTEXT", "version": 1 }`. The logical ID stays stable across revisions; each ID/version pair is immutable. Every live write must reject an attempt to overwrite an existing pair. Schema version and record version are separate. The archived 1.0 schema and fixture preserve historical interpretation; no runtime migration has been implemented.

The root schema validates one domain record. Example bundles contain a `records` array and a prominent fixture notice; the package verifier validates each record and cross-record references.

All listed fields are required; fields that can be unknown explicitly permit null. Units, confidence methods, data origin, and source provenance are explicit. Empty fields are permitted only where a phase legitimately has no result yet. Semantic gates restrict what can be promoted.

A structural pass is not a scientific or security pass. The reference validator demonstrates selected invariants only. It does not implement actual sandboxing, transactions, cryptographic approval, statistical tests, lineage-wide evidence re-evaluation, or a production promotion service. Those require runtime acceptance tests.

Qualification additionally uses `reference/qualification.py`. `ValidationContext` must be supplied by a protected verifier, never parsed from the candidate bundle. Trusted record identities are checked against externally authenticated content hashes; artifact digests require verified captured bytes; freshness, policy and dispatch time are external facts. The default trust context is empty. Live promotion rejects synthetic dependencies. Tests explicitly enable synthetic fixture mode and simulate protected context; they establish contract behavior only.

Plans bind family, method qualification, grants, snapshot, guardrails and operation identity. Results bind execution/analysis receipts and endpoint/control/guardrail coverage. Dossiers bind reproduction/promotion assessments and verified novelty claim charts. A receipt binds both the plan and the exact result/dossier assessment payload, preventing result-field changes from inheriting a receipt. `subject_digest` excludes only receipt-reference fields to avoid hash cycles; `plan_digest` excludes only status and registration. The documented Python JSON hash profile is versioned and needs cross-language conformance vectors before runtime use.

The reference supports one hypothesis/qualifying run per promotion and the sealed-feedback fixed-family profile; unsupported composition fails closed. A `validated_solution` may be a known useful rediscovery. A `validated_candidate` additionally needs protected assessed differences. Neither is global novelty or a legal opinion.

The JSON root and record contracts are provider-neutral. Extra unrecognized fields are rejected to expose drift; extensions require a versioned contract update rather than silent pass-through.

The 1.2 `improvement_candidate` record binds incumbent/challenger, observed evidence, protected evaluation/family, scoped deployment and rollback. A trusted externally authenticated candidate assessment is required for qualification/deployment. No such authentication or actual learning runtime is implemented by the reference checks.
