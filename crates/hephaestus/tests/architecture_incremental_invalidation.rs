//! AE-04 S1 — index artifact dependencies (R-015, R-017, R-107, R-108).
//!
//! A rebuildable reverse-dependency index over versioned artifacts:
//! derivatives name the exact artifact versions they were built from,
//! dependents resolve in reverse, and traversal terminates on cycles
//! while counting multiply-reached derivatives once.

use hephaestus::trustprop::{DependencyIndex, IndexError};

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
