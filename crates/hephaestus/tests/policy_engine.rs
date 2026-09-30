//! T-007 ticket 03 — deterministic policy engine (seam: `hephaestus::policy`).
//!
//! Obligations cited in tests: R-052/AT-052 (deterministic exact gates),
//! R-060/AT-060 (bind approval to exact operation/artifact/destination/policy),
//! R-095 (trust from protected context), R-099/AT-099 (recheck before dispatch).

use hephaestus::contracts::generated::{
    MissionDataOrigin, Money, Provenance, RecordRef, TrustOrigin,
};
use hephaestus::policy::{
    CapabilityGrant, GrantSpec, MissionState, PolicyEngine, PolicyRequest, ReasonCode,
};
use hephaestus::security::digest::record_digest;
use hephaestus::security::trust::TrustContext;
use time::OffsetDateTime;

const T0: &str = "2026-09-30T00:00:00+00:00";
const T1: &str = "2026-10-01T00:00:00+00:00";

fn clock() -> OffsetDateTime {
    OffsetDateTime::parse(T0, &time::format_description::well_known::Rfc3339).unwrap()
}

fn spec() -> GrantSpec {
    GrantSpec {
        id: "GRANT-1".to_string(),
        mission_ref: RecordRef {
            id: "MIS-1".to_string(),
            version: 1,
        },
        operation_id: "OP-1".to_string(),
        capabilities: vec!["run_sandboxed".to_string()],
        destination: Some("local".to_string()),
        artifact_sha256: Some("a".repeat(64)),
        policy_version: "PV-1".to_string(),
        issued_at: T0.to_string(),
        expires_at: T1.to_string(),
        max_cost: Money {
            currency: "USD".to_string(),
            minor_units: 500,
        },
        issuer_id: "AUTH-LOCAL".to_string(),
        created_at: T0.to_string(),
        data_origin: MissionDataOrigin::SyntheticFixture,
        provenance: Provenance {
            actor_id: "AUTH-LOCAL".to_string(),
            method: "standing_local_authority".to_string(),
            input_refs: vec![],
            artifact_hashes: vec![],
            trust_origin: TrustOrigin::Owner,
        },
        record_version: 1,
    }
}

fn mission() -> MissionState {
    MissionState {
        id: "MIS-1".to_string(),
        version: 1,
        authorization_state: Some("approved".to_string()),
        policy_version: Some("PV-1".to_string()),
        allowed_capabilities: Some(vec!["run_sandboxed".to_string()]),
        allowed_destinations: Some(vec!["local".to_string()]),
    }
}

fn request() -> PolicyRequest {
    PolicyRequest {
        operation_id: "OP-1".to_string(),
        required_capabilities: vec!["run_sandboxed".to_string()],
        destination: Some("local".to_string()),
        artifact_sha256: Some("a".repeat(64)),
        budget: Some(Money {
            currency: "USD".to_string(),
            minor_units: 400,
        }),
    }
}

/// Protected context: grant allowlisted, digest authenticated, policy current,
/// clock injected — exactly the T-004 fixture shape.
fn trusted_ctx(grant: &CapabilityGrant) -> TrustContext {
    let mut ctx = TrustContext::empty();
    let value = serde_json::to_value(grant.to_contract()).expect("grant json");
    ctx.trust_grant(&grant.id, grant.record_version);
    ctx.authenticate_record_hash(
        &grant.id,
        grant.record_version,
        record_digest(&value).expect("digest"),
    );
    ctx.set_current_policy("MIS-1", "PV-1".to_string());
    ctx.set_evaluated_at(clock());
    ctx
}

#[test]
fn exact_trusted_unexpired_grant_is_allowed() {
    // Positive control for AT-060/AT-099: valid unchanged grants still work.
    let grant = CapabilityGrant::mint(spec()).expect("mint");
    let ctx = trusted_ctx(&grant);
    let decision = PolicyEngine::evaluate(&request(), &mission(), Some(&grant), &ctx);
    assert!(
        decision.allowed,
        "expected allow, got {:?}",
        decision.reasons
    );
    assert!(decision.reasons.is_empty());
}

#[test]
fn single_facet_mismatch_reports_exactly_that_reason() {
    // AT-060: parameterize a change after approval -> specific reason.
    let grant = CapabilityGrant::mint(spec()).expect("mint");
    let ctx = trusted_ctx(&grant);
    let mut req = request();
    req.operation_id = "OP-EVIL".to_string();
    let decision = PolicyEngine::evaluate(&req, &mission(), Some(&grant), &ctx);
    assert!(!decision.allowed);
    assert_eq!(decision.reasons, vec![ReasonCode::GrantOperationMismatch]);

    let mut req = request();
    req.artifact_sha256 = Some("b".repeat(64));
    let decision = PolicyEngine::evaluate(&req, &mission(), Some(&grant), &ctx);
    assert_eq!(decision.reasons, vec![ReasonCode::GrantArtifactMismatch]);

    // Destination binding: request moved off the granted destination while
    // the mission would have allowed the new one.
    let mut req = request();
    req.destination = Some("remote".to_string());
    let mut permissive_mission = mission();
    permissive_mission.allowed_destinations = Some(vec!["local".to_string(), "remote".to_string()]);
    let decision = PolicyEngine::evaluate(&req, &permissive_mission, Some(&grant), &ctx);
    assert_eq!(decision.reasons, vec![ReasonCode::GrantDestinationMismatch]);

    // Destination allow-list: grant and request agree on a destination the
    // mission never allowed.
    let mut spec = spec();
    spec.destination = Some("remote".to_string());
    let grant2 = CapabilityGrant::mint(spec).expect("mint");
    let ctx2 = trusted_ctx(&grant2);
    let mut req2 = request();
    req2.destination = Some("remote".to_string());
    let decision = PolicyEngine::evaluate(&req2, &mission(), Some(&grant2), &ctx2);
    assert_eq!(
        decision.reasons,
        vec![ReasonCode::GrantDestinationNotAllowed]
    );
}

#[test]
fn multi_fault_reasons_come_out_in_fixed_declaration_order() {
    // R-052: identical inputs -> identical decision, including reason order.
    let mut spec = spec();
    spec.operation_id = "OP-OTHER".to_string();
    spec.destination = Some("remote".to_string());
    spec.policy_version = "PV-EVIL".to_string();
    let grant = CapabilityGrant::mint(spec).expect("mint");
    let mut ctx = trusted_ctx(&grant);
    let after = OffsetDateTime::parse(
        "2026-11-01T00:00:00+00:00",
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    ctx.set_evaluated_at(after);
    let mut mission = mission();
    mission.authorization_state = Some("pending".to_string());
    let mut req = request();
    req.budget = Some(Money {
        currency: "USD".to_string(),
        minor_units: 9999,
    });

    let decision = PolicyEngine::evaluate(&req, &mission, Some(&grant), &ctx);
    assert!(!decision.allowed);
    assert_eq!(
        decision.reasons,
        vec![
            ReasonCode::MissionAuthNotApproved,
            ReasonCode::GrantPolicyMismatch,
            ReasonCode::GrantOperationMismatch,
            ReasonCode::GrantDestinationMismatch,
            ReasonCode::GrantCostExceeded,
            ReasonCode::GrantExpiredOrTimeUnknown,
        ],
        "exact fixed declaration order"
    );
    // Re-evaluating identical inputs yields the identical decision (Eq).
    let again = PolicyEngine::evaluate(&req, &mission, Some(&grant), &ctx);
    assert_eq!(decision, again, "decisions must be byte-identical");
}

#[test]
fn fail_closed_on_every_missing_fact() {
    // No grant at all.
    let ctx = TrustContext::empty();
    let decision = PolicyEngine::evaluate(&request(), &mission(), None, &ctx);
    assert_eq!(decision.reasons, vec![ReasonCode::GrantMissing]);

    // Untrusted: allowlist entry exists but the digest was never authenticated.
    let grant = CapabilityGrant::mint(spec()).expect("mint");
    let mut ctx = TrustContext::empty();
    ctx.trust_grant(&grant.id, grant.record_version);
    ctx.set_current_policy("MIS-1", "PV-1".to_string());
    ctx.set_evaluated_at(clock());
    let decision = PolicyEngine::evaluate(&request(), &mission(), Some(&grant), &ctx);
    assert_eq!(decision.reasons, vec![ReasonCode::GrantNotTrusted]);

    // Trusted but never marked current policy for the mission.
    let mut ctx = trusted_ctx(&grant);
    ctx.set_current_policy("MIS-1", "PV-OTHER".to_string());
    let decision = PolicyEngine::evaluate(&request(), &mission(), Some(&grant), &ctx);
    assert_eq!(decision.reasons, vec![ReasonCode::GrantPolicyMismatch]);

    // Missing mission authorization state denies.
    let ctx = trusted_ctx(&grant);
    let mut bad_mission = mission();
    bad_mission.authorization_state = None;
    let decision = PolicyEngine::evaluate(&request(), &bad_mission, Some(&grant), &ctx);
    assert!(
        decision
            .reasons
            .contains(&ReasonCode::MissionAuthNotApproved),
        "missing mission auth denies: {:?}",
        decision.reasons
    );

    // Revocation is immediate.
    let mut revoked = CapabilityGrant::mint(spec()).expect("mint");
    revoked.revoke();
    let ctx = trusted_ctx(&revoked);
    let decision = PolicyEngine::evaluate(&request(), &mission(), Some(&revoked), &ctx);
    assert_eq!(decision.reasons, vec![ReasonCode::GrantRevoked]);
}

#[test]
fn engine_and_contract_layer_agree_on_every_shared_facet() {
    // Consistency invariant: where the engine denies, the T-004 contract
    // check errors, and vice versa — extraction changed granularity, never
    // outcomes. The engine may add reasons (specificity); the parity is at
    // the allow/deny line.
    for facet in facets() {
        let scenario = (facet.mutate)(Scenario::base());
        let engine = scenario.engine_decision();
        let contract_errors = scenario.contract_errors();
        assert_eq!(
            engine.allowed,
            contract_errors.is_empty(),
            "scenario {}: engine allowed={} reasons={:?} vs contract errors={:?}",
            facet.label,
            engine.allowed,
            engine.reasons,
            contract_errors
        );
    }
}

struct Scenario {
    spec: GrantSpec,
    mission: MissionState,
    request: PolicyRequest,
    trusted: bool,
    current_policy: Option<&'static str>,
    clock: Option<&'static str>,
    mission_version_override: Option<i64>,
    revoke: bool,
}

impl Scenario {
    fn base() -> Self {
        Scenario {
            spec: spec(),
            mission: mission(),
            request: request(),
            trusted: true,
            current_policy: Some("PV-1"),
            clock: Some(T0),
            mission_version_override: None,
            revoke: false,
        }
    }

    fn engine_decision(&self) -> hephaestus::policy::PolicyDecision {
        let mut grant = CapabilityGrant::mint(self.spec.clone()).expect("mint");
        if self.revoke {
            grant.revoke();
        }
        let ctx = self.context(&grant);
        let mut mission = self.mission.clone();
        if let Some(version) = self.mission_version_override {
            mission.version = version;
        }
        PolicyEngine::evaluate(&self.request, &mission, Some(&grant), &ctx)
    }

    fn contract_errors(&self) -> Vec<String> {
        use hephaestus::security::grant::{GrantCheck, authorize_grant};

        let mut grant = CapabilityGrant::mint(self.spec.clone()).expect("mint");
        if self.revoke {
            grant.revoke();
        }
        let ctx = self.context(&grant);
        let grant_value = serde_json::to_value(grant.to_contract()).expect("grant json");
        let mission_json = serde_json::json!({
            "id": self.mission.id,
            "record_version": self.mission_version_override.unwrap_or(self.mission.version),
            "authorization": {
                "state": self.mission.authorization_state,
                "policy_version": self.mission.policy_version,
                "allowed_capabilities": self.mission.allowed_capabilities,
                "allowed_destinations": self.mission.allowed_destinations,
            }
        });
        let record_json = serde_json::json!({
            "id": "TASK-1",
            "kind": "task",
            "record_version": 1,
            "grant_ref": {"id": grant.id, "version": grant.record_version},
            "budget": self.request.budget,
            "required_capabilities": self.request.required_capabilities,
            "operation_id": self.request.operation_id,
            "destination": self.request.destination,
        });
        let capabilities: Vec<&str> = self
            .request
            .required_capabilities
            .iter()
            .map(String::as_str)
            .collect();
        let check = GrantCheck {
            record: &record_json,
            mission: &mission_json,
            grant: &grant_value,
            artifact: self.request.artifact_sha256.as_deref(),
            capabilities: &capabilities,
            operation: &self.request.operation_id,
            destination: self.request.destination.as_deref(),
        };
        authorize_grant(&check, &ctx)
    }

    fn context(&self, grant: &CapabilityGrant) -> TrustContext {
        let mut ctx = TrustContext::empty();
        let value = serde_json::to_value(grant.to_contract()).expect("grant json");
        if self.trusted {
            ctx.trust_grant(&grant.id, grant.record_version);
            ctx.authenticate_record_hash(
                &grant.id,
                grant.record_version,
                record_digest(&value).expect("digest"),
            );
        }
        if let Some(policy) = self.current_policy {
            ctx.set_current_policy(&self.mission.id, policy.to_string());
        }
        if let Some(clock) = self.clock {
            ctx.set_evaluated_at(
                OffsetDateTime::parse(clock, &time::format_description::well_known::Rfc3339)
                    .unwrap(),
            );
        }
        ctx
    }
}

#[test]
fn deny_matrix_reports_one_specific_reason_per_single_fault() {
    // AT-060: every parameterized change after approval denies with its own
    // specific reason; AT-099: recheck covers each facet around dispatch.
    for facet in facets() {
        let Some(expected) = facet.expected else {
            continue;
        };
        let scenario = (facet.mutate)(Scenario::base());
        let grant = {
            let mut g = CapabilityGrant::mint(scenario.spec.clone()).expect("mint");
            if scenario.revoke {
                g.revoke();
            }
            g
        };
        let ctx = scenario.context(&grant);
        let mut mission = scenario.mission.clone();
        if let Some(version) = scenario.mission_version_override {
            mission.version = version;
        }
        let decision = PolicyEngine::evaluate(&scenario.request, &mission, Some(&grant), &ctx);
        assert!(!decision.allowed, "scenario {} must deny", facet.label);
        assert_eq!(
            decision.reasons,
            vec![expected],
            "scenario {}: exactly one specific reason",
            facet.label
        );
    }
    // And with no grant at all.
    let decision = PolicyEngine::evaluate(&request(), &mission(), None, &TrustContext::empty());
    assert_eq!(decision.reasons, vec![ReasonCode::GrantMissing]);
}

/// One facet of the shared table: a single mutation of the base scenario,
/// its expected engine reason (`None` = the positive control), used by the
/// parity, single-reason, and determinism tests alike (one table, not two
/// that must stay in sync — T-007 review pass 1).
struct Facet {
    label: &'static str,
    mutate: fn(Scenario) -> Scenario,
    expected: Option<ReasonCode>,
}

fn facets() -> Vec<Facet> {
    vec![
        Facet {
            label: "positive control",
            mutate: |s| s,
            expected: None,
        },
        Facet {
            label: "operation",
            mutate: |mut s| {
                s.request.operation_id = "OP-EVIL".to_string();
                s
            },
            expected: Some(ReasonCode::GrantOperationMismatch),
        },
        Facet {
            label: "artifact",
            mutate: |mut s| {
                s.request.artifact_sha256 = Some("b".repeat(64));
                s
            },
            expected: Some(ReasonCode::GrantArtifactMismatch),
        },
        Facet {
            label: "destination binding",
            mutate: |mut s| {
                s.request.destination = Some("remote".to_string());
                s.mission.allowed_destinations =
                    Some(vec!["local".to_string(), "remote".to_string()]);
                s
            },
            expected: Some(ReasonCode::GrantDestinationMismatch),
        },
        Facet {
            label: "destination allow-list",
            mutate: |mut s| {
                s.spec.destination = Some("remote".to_string());
                s.request.destination = Some("remote".to_string());
                s
            },
            expected: Some(ReasonCode::GrantDestinationNotAllowed),
        },
        Facet {
            label: "capability",
            mutate: |mut s| {
                s.request.required_capabilities = vec!["network".to_string()];
                s
            },
            expected: Some(ReasonCode::GrantCapabilityNotGranted),
        },
        Facet {
            label: "mission auth pending",
            mutate: |mut s| {
                s.mission.authorization_state = Some("pending".to_string());
                s
            },
            expected: Some(ReasonCode::MissionAuthNotApproved),
        },
        Facet {
            label: "mission auth missing",
            mutate: |mut s| {
                s.mission.authorization_state = None;
                s
            },
            expected: Some(ReasonCode::MissionAuthNotApproved),
        },
        Facet {
            label: "trust",
            mutate: |mut s| {
                s.trusted = false;
                s
            },
            expected: Some(ReasonCode::GrantNotTrusted),
        },
        Facet {
            label: "policy at context",
            mutate: |mut s| {
                s.current_policy = Some("PV-OTHER");
                s
            },
            expected: Some(ReasonCode::GrantPolicyMismatch),
        },
        Facet {
            label: "policy at grant",
            mutate: |mut s| {
                s.spec.policy_version = "PV-EVIL".to_string();
                s
            },
            expected: Some(ReasonCode::GrantPolicyMismatch),
        },
        Facet {
            label: "mission ref",
            mutate: |mut s| {
                s.mission_version_override = Some(2);
                s
            },
            expected: Some(ReasonCode::MissionRefMismatch),
        },
        Facet {
            label: "revocation",
            mutate: |mut s| {
                s.revoke = true;
                s
            },
            expected: Some(ReasonCode::GrantRevoked),
        },
        Facet {
            label: "cost over cap",
            mutate: |mut s| {
                s.request.budget = Some(Money {
                    currency: "USD".to_string(),
                    minor_units: 9999,
                });
                s
            },
            expected: Some(ReasonCode::GrantCostExceeded),
        },
        Facet {
            label: "currency",
            mutate: |mut s| {
                s.request.budget = Some(Money {
                    currency: "EUR".to_string(),
                    minor_units: 1,
                });
                s
            },
            expected: Some(ReasonCode::GrantCostExceeded),
        },
        Facet {
            label: "budget missing",
            mutate: |mut s| {
                s.request.budget = None;
                s
            },
            expected: Some(ReasonCode::RequestBudgetMissing),
        },
        Facet {
            label: "expired",
            mutate: |mut s| {
                s.clock = Some("2026-11-01T00:00:00+00:00");
                s
            },
            expected: Some(ReasonCode::GrantExpiredOrTimeUnknown),
        },
        Facet {
            label: "clock unknown",
            mutate: |mut s| {
                s.clock = None;
                s
            },
            expected: Some(ReasonCode::GrantExpiredOrTimeUnknown),
        },
    ]
}

#[test]
fn decisions_are_reproducible_across_the_whole_matrix() {
    // R-052 nondeterminism regression guard: the same inputs must keep
    // yielding the same decision (goldens elsewhere pin the actual bytes);
    // this fails if time/randomness/iteration order ever leaks in.
    for facet in facets() {
        let build = || {
            let scenario = (facet.mutate)(Scenario::base());
            let mut grant = CapabilityGrant::mint(scenario.spec.clone()).expect("mint");
            if scenario.revoke {
                grant.revoke();
            }
            let ctx = scenario.context(&grant);
            let mut mission = scenario.mission.clone();
            if let Some(version) = scenario.mission_version_override {
                mission.version = version;
            }
            let decision = PolicyEngine::evaluate(&scenario.request, &mission, Some(&grant), &ctx);
            serde_json::to_string(&decision).expect("serialize decision")
        };
        assert_eq!(
            build(),
            build(),
            "scenario {}: decisions must be reproducible",
            facet.label
        );
    }
}

#[test]
fn request_and_decision_key_sets_are_pinned() {
    // Provider credentials stay outside model-visible context: the shapes
    // carry only enumerated facts — a credential/provider slot cannot appear
    // without this test failing.
    let request_json = serde_json::to_value(request()).expect("serialize request");
    let request_keys: Vec<&str> = request_json
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        request_keys,
        vec![
            "artifact_sha256",
            "budget",
            "destination",
            "operation_id",
            "required_capabilities",
        ]
    );

    // The exact key sets ARE the guarantee: any credential/provider slot on
    // any engine shape changes a pinned list above and fails here — the
    // forbidden-word scan of review pass 1 was dead weight after the pins.
    let mission_json = serde_json::to_value(mission()).expect("serialize mission");
    let mission_keys: Vec<&str> = mission_json
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        mission_keys,
        vec![
            "allowed_capabilities",
            "allowed_destinations",
            "authorization_state",
            "id",
            "policy_version",
            "version",
        ]
    );

    let decision = {
        let grant = CapabilityGrant::mint(spec()).expect("mint");
        let ctx = trusted_ctx(&grant);
        PolicyEngine::evaluate(&request(), &mission(), Some(&grant), &ctx)
    };
    let decision_json = serde_json::to_value(&decision).expect("serialize decision");
    let decision_keys: Vec<&str> = decision_json
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(decision_keys, vec!["allowed", "reasons"]);
}

#[test]
fn golden_decision_bytes_are_stable() {
    // R-052 "byte-identical decisions": expected values are literal strings
    // — an independent source of truth, not a recomputation of the code.
    let allow = {
        let grant = CapabilityGrant::mint(spec()).expect("mint");
        let ctx = trusted_ctx(&grant);
        PolicyEngine::evaluate(&request(), &mission(), Some(&grant), &ctx)
    };
    assert_eq!(
        serde_json::to_string(&allow).expect("serialize"),
        r#"{"allowed":true,"reasons":[]}"#
    );

    let operation = {
        let mut req = request();
        req.operation_id = "OP-EVIL".to_string();
        let grant = CapabilityGrant::mint(spec()).expect("mint");
        let ctx = trusted_ctx(&grant);
        PolicyEngine::evaluate(&req, &mission(), Some(&grant), &ctx)
    };
    assert_eq!(
        serde_json::to_string(&operation).expect("serialize"),
        r#"{"allowed":false,"reasons":["GRANT_OPERATION_MISMATCH"]}"#
    );

    let missing = PolicyEngine::evaluate(&request(), &mission(), None, &TrustContext::empty());
    assert_eq!(
        serde_json::to_string(&missing).expect("serialize"),
        r#"{"allowed":false,"reasons":["GRANT_MISSING"]}"#
    );
}

#[test]
fn invalid_interval_denies_after_construction_time() {
    // Pub fields mean a grant can be mutated post-mint; the engine must not
    // trust construction-time validation alone (fail-closed recheck).
    let mut grant = CapabilityGrant::mint(spec()).expect("mint");
    grant.issued_at = grant.expires_at.clone(); // empty window
    let ctx = trusted_ctx(&grant);
    let decision = PolicyEngine::evaluate(&request(), &mission(), Some(&grant), &ctx);
    assert_eq!(decision.reasons, vec![ReasonCode::GrantInvalidInterval]);

    // Unparseable timestamps: same fail-closed refusal at evaluation.
    let mut grant = CapabilityGrant::mint(spec()).expect("mint");
    grant.expires_at = "not-a-timestamp".to_string();
    let ctx = trusted_ctx(&grant);
    let decision = PolicyEngine::evaluate(&request(), &mission(), Some(&grant), &ctx);
    assert_eq!(decision.reasons, vec![ReasonCode::GrantInvalidInterval]);
}
