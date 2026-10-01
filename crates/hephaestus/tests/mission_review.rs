//! T-012 Phase 7 fix-cycle tests (pass 1 findings).
//!
//! F1: revise Goal must re-screen tripwires. F2: compile accumulates every
//! matching authorization request. F3: assumption variants constructed from
//! goal text. S1-S7: spec-partial closures (fields, destinations, codes,
//! impact exactness, guardrails, forbidden sets, serde round-trip).

use hephaestus::contracts::generated::Money;
use hephaestus::mission::{
    AssumptionKind, AuthReason, AutonomyProfile, Compiled, Intake, MissionChange, ResourceEnvelope,
    ResourceProfile, compile, revise,
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

fn mission_for(goal: &str) -> hephaestus::mission::Mission {
    match compile(&intake(goal)).expect("compile must not error") {
        Compiled::Mission(m) => *m,
        Compiled::NeedsAuthorization(r) => panic!("expected Mission, got: {r:?}"),
    }
}

#[test]
fn revise_goal_rescreens_tripwires() {
    let first = mission_for("reduce benchmark cost");
    let result = revise(
        &first,
        &MissionChange::Goal("actuate the robotic arm to sort samples".to_string()),
    );
    match result {
        Err(hephaestus::mission::ReviseError::NeedsAuthorization(requests)) => assert!(
            requests
                .iter()
                .any(|r| r.reason == AuthReason::ActuationRefused),
            "must carry ActuationRefused, got: {requests:?}"
        ),
        Err(other) => panic!("expected NeedsAuthorization, got: {other}"),
        Ok(_) => panic!("authorization-grade goal must not revise into a Mission"),
    }
}

#[test]
fn compile_accumulates_every_matching_request() {
    let mut multi = intake("publish the robotic arm actuation manual");
    multi.resource.spending_approved = false;
    match compile(&multi).expect("compile must not error") {
        Compiled::NeedsAuthorization(requests) => {
            for want in [
                AuthReason::ActuationRefused,
                AuthReason::ExternalDisclosure,
                AuthReason::UnapprovedSpending,
            ] {
                assert!(
                    requests.iter().any(|r| r.reason == want),
                    "must carry {want:?}, got: {requests:?}"
                );
            }
        }
        Compiled::Mission(_) => panic!("multi-category goal must not compile"),
    }
}

#[test]
fn assumption_variants_come_from_goal_text() {
    let mission = mission_for("survey the subdomain corpus for benchmark vocabulary");
    let kinds: Vec<AssumptionKind> = mission.assumptions.iter().map(|a| a.kind.clone()).collect();
    for want in [
        AssumptionKind::Terminology,
        AssumptionKind::SearchVocabulary,
        AssumptionKind::InitialCorpus,
        AssumptionKind::TentativeSubdomain,
    ] {
        assert!(
            kinds.contains(&want),
            "must construct {want:?}, got: {kinds:?}"
        );
    }
}

#[test]
fn plain_goal_gets_only_the_terminology_default() {
    let mission = mission_for("reduce benchmark cost");
    assert_eq!(
        mission.assumptions.len(),
        1,
        "no hint words means exactly the terminology default, got: {:?}",
        mission.assumptions
    );
    assert_eq!(mission.assumptions[0].kind, AssumptionKind::Terminology);
}

#[test]
fn mission_carries_the_full_section_4_field_set() {
    let mission = mission_for("reduce benchmark cost");
    assert!(!mission.beneficiary.is_empty());
    assert!(!mission.objective.is_empty());
    assert!(!mission.domain_boundaries.is_empty());
    assert!(!mission.constraints.is_empty());
    assert_eq!(mission.envelope.limit.minor_units, 10_000);
    assert!(!mission.permitted_tools.is_empty());
    assert!(!mission.permitted_destinations.is_empty());
    assert!(!mission.forbidden_actions.is_empty());
    assert!(!mission.confidentiality.is_empty());
    assert!(!mission.success_metrics.is_empty());
    assert!(!mission.guardrails.is_empty());
    assert!(!mission.evidence_standard.is_empty());
    assert!(!mission.stop_conditions.is_empty());
    assert!(!mission.assumptions.is_empty());
}

#[test]
fn guardrails_block_metric_gaming() {
    let mission = mission_for("reduce latency");
    assert_ne!(
        mission.objective, mission.success_metrics[0],
        "objective and metrics must be distinct records"
    );
    assert!(
        mission
            .guardrails
            .iter()
            .any(|g| g.contains("trade-offs") || g.contains("quality")),
        "anti-quality-destruction guardrail must be present, got: {:?}",
        mission.guardrails
    );
}

#[test]
fn forbidden_sets_hold_for_every_profile() {
    for profile in [
        AutonomyProfile::PlanOnly,
        AutonomyProfile::LocalResearch,
        AutonomyProfile::SandboxExperiments,
        AutonomyProfile::SupervisedExternal,
    ] {
        let mut full = intake("reduce benchmark cost");
        full.requested_profile = profile;
        let mission = match compile(&full).expect("compile") {
            Compiled::Mission(m) => *m,
            Compiled::NeedsAuthorization(r) => panic!("got: {r:?}"),
        };
        assert!(
            mission
                .forbidden_actions
                .iter()
                .any(|a| a.contains("actuation")),
            "forbidden set must name actuation"
        );
        assert!(
            mission
                .forbidden_actions
                .iter()
                .any(|a| a.contains("disclosure")),
            "forbidden set must name disclosure"
        );
        assert!(
            !mission
                .permitted_tools
                .iter()
                .any(|t| t.contains("actuat") || t.contains("disclos")),
            "no profile may permit actuation/disclosure tools, got: {:?}",
            mission.permitted_tools
        );
    }
}

#[test]
fn destinations_widen_strictly_by_profile() {
    let counts: Vec<usize> = [
        AutonomyProfile::PlanOnly,
        AutonomyProfile::LocalResearch,
        AutonomyProfile::SandboxExperiments,
        AutonomyProfile::SupervisedExternal,
    ]
    .iter()
    .map(|p| {
        let mut full = intake("reduce benchmark cost");
        full.requested_profile = *p;
        match compile(&full).expect("compile") {
            Compiled::Mission(m) => m.permitted_destinations.len(),
            Compiled::NeedsAuthorization(r) => panic!("got: {r:?}"),
        }
    })
    .collect();
    assert!(
        counts[0] < counts[1] && counts[1] < counts[2] && counts[2] < counts[3],
        "destinations must widen strictly: {counts:?}"
    );
}

#[test]
fn mission_serde_round_trips() {
    let mission = mission_for("reduce benchmark cost");
    let json = serde_json::to_string(&mission).expect("serializes");
    let back: hephaestus::mission::Mission = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(mission, back, "record must survive a serde round-trip");
}
