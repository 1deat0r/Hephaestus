//! Evidence lifecycle (T-024, R-017, R-047 (corrected/retracted
//! evidence propagates to dependents — invalidate traverses),
//! R-097, R-098/AT-098 (stale/retracted/quarantined dependencies
//! invalidate dependent labels, history intact), MASTER_SPEC §16).
//!
//! Integration tests at the public seam: `advance_*`, `invalidate`,
//! `queue`, `reactivate`.

use hephaestus::lifecycle::record::{
    Correction, CorrectionKind, HypothesisState, OpportunityState, QueueError,
};
use hephaestus::lifecycle::{
    EvidenceGraph, advance_hypothesis, advance_opportunity, queue, reactivate,
};

fn s(v: &str) -> String {
    v.to_string()
}

// ---- Ticket 01: state machines + invalidation ----

#[test]
fn legal_and_illegal_opportunity_transitions() {
    // Legal: discovered -> grounded.
    let ok = advance_opportunity(
        "o-1",
        OpportunityState::Discovered,
        OpportunityState::Grounded,
        "snap-1",
        "m-1",
        "h1",
    );
    assert!(ok.is_ok());
    // Illegal: discovered -> closed skipping grounding... (allowed: any ->
    // closed). Use a genuinely illegal hop: explored -> discovered.
    let err = advance_opportunity(
        "o-1",
        OpportunityState::Explored,
        OpportunityState::Discovered,
        "snap-1",
        "m-1",
        "h1",
    );
    assert!(err.is_err(), "named illegal transition");
    assert!(err.unwrap_err().reason.contains("illegal"));
}

#[test]
fn legal_and_illegal_hypothesis_transitions() {
    // Legal forward path.
    assert!(
        advance_hypothesis(
            "h-1",
            HypothesisState::Exploratory,
            HypothesisState::Compiled,
            "s",
            "m",
            "h1"
        )
        .is_ok()
    );
    assert!(
        advance_hypothesis(
            "h-1",
            HypothesisState::Compiled,
            HypothesisState::Reviewed,
            "s",
            "m",
            "h1"
        )
        .is_ok()
    );
    // Illegal: exploratory -> test-ready (skipping compiled/reviewed).
    let err = advance_hypothesis(
        "h-1",
        HypothesisState::Exploratory,
        HypothesisState::TestReady,
        "s",
        "m",
        "h1",
    );
    assert!(err.is_err());
}

#[test]
fn transition_binds_snapshot_and_lineage() {
    let rec = advance_hypothesis(
        "h-1",
        HypothesisState::Compiled,
        HypothesisState::Reviewed,
        "snap-7",
        "m-1",
        "h1.0.0",
    )
    .expect("legal");
    assert_eq!(rec.snapshot_id, "snap-7");
    assert_eq!(rec.lineage_mission, "m-1");
    assert_eq!(rec.lineage_hypothesis_version, "h1.0.0");
    assert_eq!(rec.from_state, "Compiled");
    assert_eq!(rec.to_state, "Reviewed");
}

#[test]
fn invalidation_traverses_dependents_append_only() {
    let mut graph = EvidenceGraph::default();
    // dossier depends on result depends on observation.
    graph.add_dependency("dossier-1", "result-1");
    graph.add_dependency("result-1", "observation-1");
    assert!(!graph.is_stale("dossier-1"));
    let correction = Correction {
        target_id: s("observation-1"),
        correction_id: s("corr-1"),
        kind: CorrectionKind::CorrectedObservation,
        reason: s("sensor miscalibration discovered"),
        snapshot_id: s("snap-8"),
    };
    let report = graph.invalidate(correction.clone());
    // Traversal reached the dossier through result-1.
    assert!(report.stale.iter().any(|m| m.dependent_id == "dossier-1"));
    assert!(
        report
            .stale
            .iter()
            .any(|m| m.dependent_id == "observation-1")
    );
    // Historical result intact while current label stale (§16:317).
    assert!(report.stale.iter().all(|m| m.historical_result_intact));
    // Re-evaluation awaits budget+permission (§16:314).
    assert!(
        report
            .reevaluation
            .iter()
            .all(|e| e.requires_budget_and_permission)
    );
    // Append-only: the original correction is retained verbatim.
    assert_eq!(graph.corrections().len(), 1);
    assert_eq!(graph.corrections()[0], correction);
    assert!(graph.is_stale("dossier-1"));
}

// ---- Ticket 02: queue + reactivation ----

#[test]
fn queue_refuses_without_budget() {
    // Restarting research still requires available budget.
    assert_eq!(
        queue("c-1", 0, Some("snap-1")),
        Err(QueueError::BudgetUnavailable)
    );
    // Missing snapshot refused (§16:318).
    assert_eq!(queue("c-1", 5, None), Err(QueueError::MissingSnapshot));
    // With budget + snapshot: queued.
    let rec = queue("c-1", 5, Some("snap-1")).expect("queued");
    assert_eq!(rec.to_state, "queued");
    assert_eq!(rec.snapshot_id, "snap-1");
}

#[test]
fn reactivation_references_original_failure() {
    // §16:316: new version referencing the original failure + changed
    // condition; the failure is not erased.
    let r = reactivate(
        "mech-2.0",
        "failure-42",
        "new workload shape documented in workload-study-7",
    );
    assert_eq!(r.original_failure_id, "failure-42");
    assert_eq!(r.new_version_id, "mech-2.0");
    assert!(r.changed_condition_evidence.contains("workload-study-7"));
}

#[test]
fn twin_run_byte_identical() {
    let mut a = EvidenceGraph::default();
    let mut b = EvidenceGraph::default();
    for g in [&mut a, &mut b] {
        g.add_dependency("d-1", "r-1");
        g.invalidate(Correction {
            target_id: s("r-1"),
            correction_id: s("c-1"),
            kind: CorrectionKind::Retraction,
            reason: s("data withdrawn"),
            snapshot_id: s("snap-1"),
        });
    }
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
