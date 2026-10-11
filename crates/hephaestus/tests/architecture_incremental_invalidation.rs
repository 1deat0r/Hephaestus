//! AE-04 S1 — index artifact dependencies (R-015, R-017, R-107, R-108).
//!
//! A rebuildable reverse-dependency index over versioned artifacts:
//! derivatives name the exact artifact versions they were built from,
//! dependents resolve in reverse, and traversal terminates on cycles
//! while counting multiply-reached derivatives once.

use hephaestus::trustprop::{
    DependencyIndex, IndexError, InvalidationCause, InvalidationEvent, InvalidationTracker,
    QualificationError,
};

fn node(id: &str, version: u64) -> (String, u64) {
    (id.to_string(), version)
}

#[test]
fn s1_index_chain_resolves_transitive_dependents() {
    // AC 10 positive: D1 built from A v1, D2 built from D1 v1 — both
    // resolve as affected by A v1, nearest first in sorted order.
    let mut index = DependencyIndex::new();
    index.record("D1", 1, &[("A", 1)]).expect("record D1");
    index.record("D2", 1, &[("D1", 1)]).expect("record D2");
    assert_eq!(index.direct_dependents("A", 1), vec![node("D1", 1)]);
    assert_eq!(index.direct_inputs("D2", 1), vec![node("D1", 1)]);
    assert_eq!(index.affected("A", 1), vec![node("D1", 1), node("D2", 1)]);
}

#[test]
fn s1_index_versions_scope_dependents() {
    // AC 10 positive: a derivative of A v1 is unaffected by A v2.
    let mut index = DependencyIndex::new();
    index.record("D3", 1, &[("A", 1)]).expect("record D3");
    assert_eq!(index.affected("A", 2), vec![]);
    assert_eq!(index.affected("A", 1), vec![node("D3", 1)]);
}

#[test]
fn s1_index_cycles_terminate_and_dedupe() {
    // AC 10 positive: cyclic derivations terminate; multiply-reached
    // derivatives appear once; the queried node itself is excluded.
    let mut index = DependencyIndex::new();
    index.record("X", 1, &[("Y", 1)]).expect("record X");
    index.record("Y", 1, &[("X", 1)]).expect("record Y");
    assert_eq!(index.affected("X", 1), vec![node("Y", 1)]);
    // Self-loop terminates with no downstream.
    index.record("Z", 1, &[("Z", 1)]).expect("record Z");
    assert_eq!(index.affected("Z", 1), vec![]);
    // Diamond: D via B and C from A — each derivative counted once.
    let mut diamond = DependencyIndex::new();
    diamond.record("B", 1, &[("A", 1)]).expect("record B");
    diamond.record("C", 1, &[("A", 1)]).expect("record C");
    diamond
        .record("D", 1, &[("B", 1), ("C", 1)])
        .expect("record D");
    assert_eq!(
        diamond.affected("A", 1),
        vec![node("B", 1), node("C", 1), node("D", 1)]
    );
}

#[test]
fn s1_index_conflicting_inputs_refuse() {
    // AC 10 refusal: re-recording a derivative version with different
    // inputs is conflicting provenance and fails; identical re-recording
    // stays idempotent.
    let mut index = DependencyIndex::new();
    index.record("D", 1, &[("A", 1)]).expect("record D");
    index
        .record("D", 1, &[("A", 1)])
        .expect("identical re-record is idempotent");
    match index.record("D", 1, &[("B", 1)]) {
        Err(IndexError::ConflictingInputs {
            derivative,
            version,
        }) => {
            assert_eq!(derivative, "D");
            assert_eq!(version, 1);
        }
        other => panic!("expected ConflictingInputs, got: {other:?}"),
    }
    // The original inputs survive the refused record.
    assert_eq!(index.direct_inputs("D", 1), vec![node("A", 1)]);
}

#[test]
fn s1_index_rebuilds_from_exported_links() {
    // AC 10 regression: the exported link list rebuilds an identical
    // index — persistence round-trips without drift.
    let mut index = DependencyIndex::new();
    index.record("D1", 1, &[("A", 1)]).expect("record D1");
    index
        .record("D2", 2, &[("D1", 1), ("A", 1)])
        .expect("record D2");
    let links = index.links();
    assert_eq!(links.len(), 3, "one link per input edge");
    let rebuilt = DependencyIndex::rebuild(&links).expect("rebuild");
    assert_eq!(rebuilt.links(), links);
    assert_eq!(rebuilt.affected("A", 1), index.affected("A", 1));
    // Unknown nodes resolve empty, never error.
    assert_eq!(rebuilt.direct_dependents("MISSING", 9), vec![]);
    assert_eq!(rebuilt.affected("MISSING", 9), vec![]);
}

fn tracked_chain() -> InvalidationTracker {
    let mut tracker = InvalidationTracker::new();
    tracker.record("D1", 1, &[("A", 1)]).expect("record D1");
    tracker.record("D2", 1, &[("D1", 1)]).expect("record D2");
    tracker
}

fn event(target: &str, version: u64, cause: InvalidationCause) -> InvalidationEvent {
    InvalidationEvent {
        target_id: target.to_string(),
        target_version: version,
        cause,
        reason: "fixture reason".to_string(),
    }
}

#[test]
fn s2_invalidation_correction_marks_target_and_dependents_stale() {
    // AC 11 positive: a correction to A v1 stales A v1 plus its
    // transitive dependents; A v2 stays fresh.
    let mut tracker = tracked_chain();
    let affected = tracker.apply(event("A", 1, InvalidationCause::Correction));
    assert_eq!(affected, vec![node("A", 1), node("D1", 1), node("D2", 1)]);
    assert!(tracker.is_stale("A", 1));
    assert!(tracker.is_stale("D1", 1));
    assert!(tracker.is_stale("D2", 1));
    assert!(!tracker.is_stale("A", 2));
}

#[test]
fn s2_invalidation_quarantine_and_retraction_propagate() {
    // AC 11 positive: quarantine and retraction use the same
    // version-scoped propagation; causes stay distinct in audit.
    let mut tracker = tracked_chain();
    tracker.apply(event("D1", 1, InvalidationCause::Quarantine));
    assert!(tracker.is_stale("D1", 1));
    assert!(tracker.is_stale("D2", 1));
    assert!(!tracker.is_stale("A", 1));
    tracker.apply(event("A", 1, InvalidationCause::Retraction));
    assert!(tracker.is_stale("A", 1));
    let causes: Vec<InvalidationCause> = tracker.audit().iter().map(|e| e.cause).collect();
    assert_eq!(
        causes,
        vec![InvalidationCause::Quarantine, InvalidationCause::Retraction]
    );
}

#[test]
fn s2_invalidation_preserves_audit_originals() {
    // AC 11 regression: the audit log is append-only; staleness queries
    // never erase or rewrite prior events.
    let mut tracker = tracked_chain();
    tracker.apply(event("A", 1, InvalidationCause::Correction));
    tracker.apply(event("D1", 1, InvalidationCause::Quarantine));
    assert_eq!(tracker.audit().len(), 2);
    assert_eq!(tracker.audit()[0].target_id, "A");
    assert_eq!(tracker.audit()[1].target_id, "D1");
    assert!(tracker.is_stale("D2", 1));
    assert_eq!(tracker.audit().len(), 2, "queries keep audit intact");
}

#[test]
fn s2_invalidation_blocks_stale_qualification_while_review_pending() {
    // AC 11 refusal: a stale derivative cannot qualify while its review
    // stays pending; a fresh derivative still qualifies.
    let mut tracker = tracked_chain();
    tracker.apply(event("A", 1, InvalidationCause::Correction));
    tracker.mark_review_pending("D2", 1);
    match tracker.can_qualify("D2", 1) {
        Err(QualificationError::StalePendingReview {
            derivative,
            version,
        }) => {
            assert_eq!(derivative, "D2");
            assert_eq!(version, 1);
        }
        other => panic!("expected StalePendingReview, got: {other:?}"),
    }
    tracker
        .record("FRESH", 1, &[("B", 1)])
        .expect("record fresh");
    assert_eq!(tracker.can_qualify("FRESH", 1), Ok(()));
}

#[test]
fn s2_invalidation_allows_qualification_after_review_resolves() {
    // AC 11 positive: resolving the pending review lifts the block;
    // staleness without a pending review never blocks.
    let mut tracker = tracked_chain();
    tracker.apply(event("A", 1, InvalidationCause::Correction));
    tracker.mark_review_pending("D1", 1);
    assert!(tracker.can_qualify("D1", 1).is_err());
    tracker.resolve_review("D1", 1);
    assert_eq!(tracker.can_qualify("D1", 1), Ok(()));
    assert!(tracker.is_stale("D2", 1), "stale flag survives review");
    assert_eq!(
        tracker.can_qualify("D2", 1),
        Ok(()),
        "stale without pending review still qualifies"
    );
}
