//! T-012 ticket 02: ambiguity taxonomy, value frame, authorization requests (R-011).
//!
//! Red-first seam tests: irreversible-operation and ambiguous-risk goals must
//! come back as authorization requests with exact reason codes.

use hephaestus::contracts::generated::Money;
use hephaestus::mission::{
    AuthReason, AutonomyProfile, Compiled, Intake, ResourceEnvelope, ResourceProfile, compile,
};

fn intake(goal: &str) -> Intake {
    Intake {
        goal: goal.to_string(),
        beneficiary: "lab team".to_string(),
        standing_priorities: vec!["reduce benchmark cost".to_string()],
        resource: ResourceProfile {
            envelope: ResourceEnvelope {
                limit: Money {
                    currency: "USD".to_string(),
                    minor_units: 10_000,
                },
                pricing_unknown: false,
            },
            spending_approved: true,
        },
        requested_profile: AutonomyProfile::LocalResearch,
        caller_hypothesis: None,
    }
}

fn requests_for(goal: &str) -> Vec<hephaestus::mission::AuthorizationRequest> {
    match compile(&intake(goal)).expect("compile must not error") {
        Compiled::NeedsAuthorization(r) => r,
        Compiled::Mission(_) => panic!("expected authorization requests for: {goal}"),
    }
}

#[test]
fn reversible_ambiguity_becomes_explicit_assumption() {
    let compiled = compile(&intake("reduce benchmark cost using standard terminology"))
        .expect("compile must not error");
    let mission = match compiled {
        Compiled::Mission(m) => m,
        Compiled::NeedsAuthorization(r) => panic!("expected Mission, got: {r:?}"),
    };
    assert!(
        !mission.assumptions.is_empty(),
        "reversible defaults must be recorded, not silently applied"
    );
    assert!(
        mission
            .assumptions
            .iter()
            .all(|a| !a.provenance.is_empty() && !a.statement.is_empty()),
        "every assumption carries statement + provenance"
    );
}

#[test]
fn unapproved_spending_requests_authorization() {
    let mut bad = intake("reduce benchmark cost");
    bad.resource.spending_approved = false;
    match compile(&bad).expect("compile must not error") {
        Compiled::NeedsAuthorization(requests) => assert!(
            requests
                .iter()
                .any(|r| r.reason == AuthReason::UnapprovedSpending),
            "must carry UnapprovedSpending, got: {requests:?}"
        ),
        Compiled::Mission(_) => panic!("unapproved spending must not compile"),
    }
}

#[test]
fn missing_value_frame_fails_closed() {
    let mut frameless = intake("invent something useful");
    frameless.standing_priorities.clear();
    match compile(&frameless).expect("compile must not error") {
        Compiled::NeedsAuthorization(requests) => assert!(
            requests
                .iter()
                .any(|r| r.reason == AuthReason::MissingValueFrame),
            "must carry MissingValueFrame, got: {requests:?}"
        ),
        Compiled::Mission(_) => panic!("frameless broad goal must not compile"),
    }
}

#[test]
fn sensitive_application_is_never_silent() {
    let requests = requests_for("design a bioweapon delivery plan");
    assert!(
        requests
            .iter()
            .any(|r| r.reason == AuthReason::SensitiveApplication),
        "must carry SensitiveApplication, got: {requests:?}"
    );
}

#[test]
fn irreversible_operation_requests_authorization() {
    let requests = requests_for("wipe all records irreversibly before the audit");
    assert!(
        requests
            .iter()
            .any(|r| r.reason == AuthReason::IrreversibleOperation),
        "must carry IrreversibleOperation, got: {requests:?}"
    );
}

#[test]
fn unsupervised_risk_requests_authorization() {
    let requests = requests_for("run the experiment unsupervised with no human review");
    assert!(
        requests
            .iter()
            .any(|r| r.reason == AuthReason::AmbiguousRisk),
        "must carry AmbiguousRisk, got: {requests:?}"
    );
}

#[test]
fn caller_hypothesis_recorded_as_provenance_never_claim() {
    let mut with_hypothesis = intake("reduce benchmark cost");
    with_hypothesis.caller_hypothesis =
        Some("caching makes it faster and quality is unaffected".to_string());
    let compiled = compile(&with_hypothesis).expect("compile must not error");
    let mission = match compiled {
        Compiled::Mission(m) => m,
        Compiled::NeedsAuthorization(r) => panic!("expected Mission, got: {r:?}"),
    };
    assert!(
        mission.caller_hypothesis_provenance.is_some(),
        "caller hypothesis must leave a provenance trace"
    );
    let json = serde_json::to_string(&mission).expect("serializes");
    assert!(
        !json.contains("caching makes it faster"),
        "unvalidated hypothesis text must not leak into the record"
    );
}
