//! T-012 ticket 03: revise/versioning, impact report, budget-envelope binding
//! (R-012/AT-012: a quality-margin change lands as a recorded constraint
//! with a new version — never a silent edit).
//!
//! Red-first seam tests against `hephaestus::mission::{revise, ...}`.

use hephaestus::budget::BudgetLedger;
use hephaestus::contracts::generated::Money;
use hephaestus::mission::{
    AutonomyProfile, Compiled, Intake, MissionChange, ResourceEnvelope, ResourceProfile, compile,
    revise,
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

fn money(minor: i64) -> Money {
    Money {
        currency: "USD".to_string(),
        minor_units: minor,
    }
}

#[test]
fn goal_change_mints_version_two() {
    let first = mission_for("reduce benchmark cost");
    let (second, impact) = revise(
        &first,
        &MissionChange::Goal("reduce benchmark memory".to_string()),
    )
    .expect("revise must succeed");
    assert_eq!(second.version, 2);
    assert_eq!(second.supersedes, Some(1));
    assert_eq!(second.objective, "reduce benchmark memory");
    assert!(
        impact.changed_sections.contains(&"objective".to_string()),
        "impact must name objective, got: {:?}",
        impact.changed_sections
    );
    // Untouched sections survive verbatim.
    assert_eq!(second.guardrails, first.guardrails);
    assert_eq!(second.envelope, first.envelope);
}

#[test]
fn cost_change_updates_envelope_only() {
    let first = mission_for("reduce benchmark cost");
    let (second, impact) =
        revise(&first, &MissionChange::CostLimit(money(25_000))).expect("revise must succeed");
    assert_eq!(second.version, 2);
    assert_eq!(second.envelope.limit.minor_units, 25_000);
    assert_eq!(impact.changed_sections, vec!["envelope".to_string()]);
}

#[test]
fn dataset_and_quality_changes_land_in_constraints() {
    let first = mission_for("reduce benchmark cost");
    let (second, impact) = revise(
        &first,
        &MissionChange::DatasetPolicy("use only licensed corpora".to_string()),
    )
    .expect("revise must succeed");
    assert!(
        second
            .constraints
            .iter()
            .any(|c| c.contains("licensed corpora")),
        "dataset policy must be recorded, got: {:?}",
        second.constraints
    );
    assert!(impact.changed_sections.contains(&"constraints".to_string()));

    let (third, _) = revise(
        &second,
        &MissionChange::QualityMargin("quality may not regress more than 1%".to_string()),
    )
    .expect("revise must succeed");
    assert_eq!(third.version, 3);
    assert_eq!(third.supersedes, Some(2));
    assert!(
        third
            .constraints
            .iter()
            .any(|c| c.contains("quality margin")),
        "quality margin must be recorded, got: {:?}",
        third.constraints
    );
}

#[test]
fn requirement_override_keeps_provenance() {
    let first = mission_for("reduce benchmark cost");
    let target = first.constraints[0].clone();
    let (second, impact) = revise(
        &first,
        &MissionChange::OverrideRequirement {
            requirement: target.clone(),
            replacement: "operate inside the approved workspace, extended hours".to_string(),
        },
    )
    .expect("revise must succeed");
    assert_eq!(second.version, 2);
    assert!(
        second
            .constraints
            .iter()
            .any(|c| c.contains("extended hours")),
        "override must land, got: {:?}",
        second.constraints
    );
    assert!(
        second
            .assumptions
            .iter()
            .any(|a| a.statement.contains("override")),
        "override must leave a provenance assumption, got: {:?}",
        second.assumptions
    );
    assert!(!impact.changed_sections.is_empty());
}

#[test]
fn unknown_requirement_is_an_error_not_a_guess() {
    let first = mission_for("reduce benchmark cost");
    let result = revise(
        &first,
        &MissionChange::OverrideRequirement {
            requirement: "no such requirement anywhere".to_string(),
            replacement: "whatever".to_string(),
        },
    );
    assert!(
        result.is_err(),
        "overriding a nonexistent requirement must fail, not guess"
    );
}

#[test]
fn revise_is_deterministic() {
    let first = mission_for("reduce benchmark cost");
    let change = MissionChange::CostLimit(money(25_000));
    let (a, _) = revise(&first, &change).expect("revise must succeed");
    let (b, _) = revise(&first, &change).expect("revise must succeed");
    let sa = serde_json::to_string(&a).expect("serializes");
    let sb = serde_json::to_string(&b).expect("serializes");
    assert_eq!(sa, sb, "twin revises must be byte-identical");
}

#[test]
fn envelope_binds_to_budget_ledger() {
    let mission = mission_for("reduce benchmark cost");
    assert!(
        !mission.envelope.pricing_unknown,
        "priced intake must stay priced through compile"
    );
    let ledger =
        BudgetLedger::new(mission.envelope.limit.clone()).expect("ledger accepts the envelope");
    assert_eq!(ledger.available().minor_units, 10_000);
    assert_eq!(ledger.limit().minor_units, 10_000);
}

#[test]
fn unknown_price_survives_compile_explicitly() {
    let mut priced = intake("reduce benchmark cost");
    priced.resource.envelope.pricing_unknown = true;
    let mission = match compile(&priced).expect("compile must not error") {
        Compiled::Mission(m) => m,
        Compiled::NeedsAuthorization(r) => panic!("expected Mission, got: {r:?}"),
    };
    assert!(
        mission.envelope.pricing_unknown,
        "unknown price must stay explicit, never normalized away"
    );
}
