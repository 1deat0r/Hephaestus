//! T-012 ticket 01: Mission record, autonomy profiles, compile happy path
//! (R-001, R-010/AT-010: the goal compiles under an existing authorized
//! profile).
//!
//! Red-first seam tests against `hephaestus::mission::{compile, ...}`.

use hephaestus::contracts::generated::Money;
use hephaestus::mission::{
    AutonomyProfile, Compiled, Intake, ResourceEnvelope, ResourceProfile, compile,
};

fn intake(goal: &str, profile: AutonomyProfile) -> Intake {
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
        requested_profile: profile,
        caller_hypothesis: None,
    }
}

#[test]
fn broad_goal_compiles_without_hypothesis() {
    let compiled = compile(&intake(
        "reduce benchmark cost",
        AutonomyProfile::LocalResearch,
    ))
    .expect("compile must not error on a well-formed intake");
    let mission = match compiled {
        Compiled::Mission(m) => m,
        Compiled::NeedsAuthorization(r) => panic!("expected Mission, got requests: {r:?}"),
    };
    assert_eq!(mission.version, 1);
    assert_eq!(mission.supersedes, None);
    assert!(!mission.objective.is_empty());
    assert!(!mission.guardrails.is_empty());
    // R-001 is structural: the serialized Mission has no hypothesis field.
    let json = serde_json::to_value(&mission).expect("Mission serializes");
    assert!(
        json.get("hypothesis").is_none(),
        "Mission must carry no hypothesis field"
    );
}

#[test]
fn four_profiles_bind_distinct_permissions() {
    let profiles = [
        AutonomyProfile::PlanOnly,
        AutonomyProfile::LocalResearch,
        AutonomyProfile::SandboxExperiments,
        AutonomyProfile::SupervisedExternal,
    ];
    let mut tool_counts = Vec::new();
    for profile in profiles {
        let compiled =
            compile(&intake("reduce benchmark cost", profile)).expect("compile must not error");
        match compiled {
            Compiled::Mission(m) => tool_counts.push(m.permitted_tools.len()),
            Compiled::NeedsAuthorization(r) => {
                panic!("expected Mission, got requests: {r:?}")
            }
        }
    }
    // Plan-only permits nothing executable; each wider profile permits more.
    assert!(
        tool_counts[0] < tool_counts[1]
            && tool_counts[1] < tool_counts[2]
            && tool_counts[2] < tool_counts[3],
        "profiles must widen strictly: {tool_counts:?}"
    );
}

#[test]
fn actuation_goal_never_compiles_to_mission() {
    let compiled = compile(&intake(
        "actuate the robotic arm to sort samples",
        AutonomyProfile::SupervisedExternal,
    ))
    .expect("compile must not error");
    match compiled {
        Compiled::Mission(_) => panic!("actuation goal must not become a Mission"),
        Compiled::NeedsAuthorization(requests) => assert!(
            !requests.is_empty(),
            "actuation goal must carry authorization requests"
        ),
    }
}

#[test]
fn compile_is_deterministic() {
    let first = compile(&intake("reduce benchmark cost", AutonomyProfile::PlanOnly))
        .expect("compile must not error");
    let second = compile(&intake("reduce benchmark cost", AutonomyProfile::PlanOnly))
        .expect("compile must not error");
    let a = serde_json::to_string(&first).expect("serializes");
    let b = serde_json::to_string(&second).expect("serializes");
    assert_eq!(a, b, "twin runs must be byte-identical");
}
