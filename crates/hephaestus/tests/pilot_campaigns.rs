//! Pilot campaigns (T-027, R-103).
//!
//! Integration tests at the public seam: `plan_pilot`, `record_outcome`,
//! `ready_for_confirmation`.

use hephaestus::pilot::record::{
    ConfirmationBlock, MissionOutcome, Partition, PilotPlan, PlanError, RepositoryAssignment,
    Stratification,
};
use hephaestus::pilot::{plan_pilot, ready_for_confirmation, record_outcome};

fn s(v: &str) -> String {
    v.to_string()
}

fn assignment(repo: &str, partition: Partition) -> RepositoryAssignment {
    RepositoryAssignment {
        repo_id: s(repo),
        partition,
        episode_ids: vec![s("ep-1"), s("ep-2")],
        size_band: s("medium"),
        topology_class: s("flat"),
    }
}

fn plan() -> PilotPlan {
    PilotPlan {
        preregistration_id: s("prereg-1"),
        stratification: Stratification {
            size_bands: vec![s("small"), s("medium")],
            topology_classes: vec![s("flat"), s("deep")],
        },
        assignments: vec![
            assignment("repo-a", Partition::TrainTune),
            assignment("repo-b", Partition::Pilot),
        ],
        batch_size: 12, // FREE — twenty is not a threshold
        intent: s("debugging batch"),
        budget: 50,
    }
}

fn preregs() -> Vec<String> {
    vec![s("prereg-1")]
}

// ---- Ticket 01: plan + partitions ----

#[test]
fn plan_registers_with_preregistration_and_partitions() {
    let p = plan_pilot(plan(), &preregs()).expect("planned");
    assert_eq!(p.batch_size, 12, "batch size free — no twenty rule");
    assert_eq!(p.intent, "debugging batch");
    // Unregistered preregistration refused.
    assert_eq!(
        plan_pilot(plan(), &[]),
        Err(PlanError::UnregisteredPreregistration)
    );
    // Empty batch refused.
    let mut empty = plan();
    empty.assignments.clear();
    assert_eq!(plan_pilot(empty, &preregs()), Err(PlanError::EmptyBatch));
}

#[test]
fn partition_conflicts_named() {
    // Same repository in two partitions.
    let mut p = plan();
    p.assignments
        .push(assignment("repo-b", Partition::Confirmatory));
    assert_eq!(
        plan_pilot(p, &preregs()),
        Err(PlanError::PartitionConflict(s("repo-b")))
    );
    // Confirmatory partition requires pre-sampled episodes (R-103).
    let mut p = plan();
    let mut c = assignment("repo-c", Partition::Confirmatory);
    c.episode_ids.clear();
    p.assignments.push(c);
    assert_eq!(
        plan_pilot(p, &preregs()),
        Err(PlanError::EpisodesNotPreSampled)
    );
}

// ---- Ticket 02: variance + gate ----

#[test]
fn variance_estimated_with_failures_retained() {
    let outcomes = vec![
        MissionOutcome {
            repo_id: s("r1"),
            arm_id: s("cand"),
            value: Some(0.10),
            failed_or_blocked: false,
        },
        MissionOutcome {
            repo_id: s("r1"),
            arm_id: s("base"),
            value: Some(0.02),
            failed_or_blocked: false,
        },
        MissionOutcome {
            repo_id: s("r2"),
            arm_id: s("cand"),
            value: Some(0.30),
            failed_or_blocked: false,
        },
        MissionOutcome {
            repo_id: s("r2"),
            arm_id: s("base"),
            value: Some(0.04),
            failed_or_blocked: false,
        },
        MissionOutcome {
            repo_id: s("r3"),
            arm_id: s("cand"),
            value: None,
            failed_or_blocked: true,
        },
        MissionOutcome {
            repo_id: s("r3"),
            arm_id: s("base"),
            value: None,
            failed_or_blocked: true,
        },
    ];
    let est = record_outcome(&outcomes);
    assert_eq!(est.total_missions, 6);
    // Failures retained in the denominator (R-103).
    assert_eq!(est.failures_retained, 2);
    // cand mean = (0.10+0.30)/2 = 0.20; base mean = 0.03.
    let cand = est.per_arm.iter().find(|(a, _, _)| a == "cand").unwrap();
    assert!((cand.1.unwrap() - 0.20).abs() < 1e-12);
    // Paired differences: r1: 0.02-0.10=-0.08; r2: 0.04-0.30=-0.26.
    let pv = est.paired_difference_variance.expect("paired variance");
    assert!(pv > 0.0);
    // Empty outcomes -> explicit None, never zeros.
    let empty = record_outcome(&[]);
    assert!(empty.per_arm.is_empty());
    assert!(empty.paired_difference_variance.is_none());
}

#[test]
fn confirmation_gate_named_blocks() {
    let est = record_outcome(&[MissionOutcome {
        repo_id: s("r1"),
        arm_id: s("cand"),
        value: Some(0.1),
        failed_or_blocked: false,
    }]);
    // Happy path.
    assert_eq!(ready_for_confirmation(Some(&est), true, true, true), Ok(()));
    // Missing variance.
    assert_eq!(
        ready_for_confirmation(None, true, true, true),
        Err(ConfirmationBlock::MissingVarianceEstimate)
    );
    // Unresolved oracle.
    assert_eq!(
        ready_for_confirmation(Some(&est), false, true, true),
        Err(ConfirmationBlock::UnresolvedOracle)
    );
    // Unqualified analysis.
    assert_eq!(
        ready_for_confirmation(Some(&est), true, false, true),
        Err(ConfirmationBlock::UnqualifiedAnalysis)
    );
    // Confirmatory-partition contamination.
    assert_eq!(
        ready_for_confirmation(Some(&est), true, true, false),
        Err(ConfirmationBlock::PartitionContamination)
    );
}

#[test]
fn twin_run_byte_identical() {
    let a = plan_pilot(plan(), &preregs()).unwrap();
    let b = plan_pilot(plan(), &preregs()).unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
