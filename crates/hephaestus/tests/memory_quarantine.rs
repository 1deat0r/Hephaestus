//! Behavioral memory (T-039, R-109/AT-109: unauthorized writes denied,
//! quarantine propagates to dependents, audit history survives).
//!
//! Integration tests at the public seam: `authorize_write`,
//! `propagate_quarantine`.

use hephaestus::memory::record::{MemoryArtifact, MemoryLabel, WriteRefusal};
use hephaestus::memory::{authorize_write, propagate_quarantine};

fn s(v: &str) -> String {
    v.to_string()
}

fn artifact(id: &str, source: &str) -> MemoryArtifact {
    MemoryArtifact {
        artifact_id: s(id),
        source_id: s(source),
        digest: format!("digest-{id}"),
        current: true,
    }
}

#[test]
fn unauthorized_behavioral_memory_write_denied() {
    // R-109 negative case: a source-induced write without a scoped
    // grant covering the target is denied.
    assert_eq!(
        authorize_write(&[], "policy-memory"),
        Err(WriteRefusal::UnauthorizedWrite)
    );
    assert_eq!(
        authorize_write(&[s("some-other-target")], "policy-memory"),
        Err(WriteRefusal::UnauthorizedWrite)
    );
    // A scoped writer may write.
    assert_eq!(
        authorize_write(&[s("policy-memory")], "policy-memory"),
        Ok(())
    );
}

#[test]
fn quarantine_propagates_to_dependent_artifacts_and_labels() {
    let mut artifacts = vec![
        artifact("a1", "src-1"),
        artifact("a2", "src-1"),
        artifact("a3", "src-2"),
    ];
    let mut labels = vec![
        MemoryLabel {
            label_id: s("l1"),
            artifact_id: s("a1"),
            current: true,
        },
        MemoryLabel {
            label_id: s("l2"),
            artifact_id: s("a3"),
            current: true,
        },
    ];
    // Quarantine src-1: a1+a2 invalidate, label l1 (on a1) invalidates;
    // a3/l2 (other source) untouched.
    let n = propagate_quarantine(&mut artifacts, &mut labels, "src-1");
    assert_eq!(n, 3, "two artifacts + one label invalidated");
    assert!(!artifacts[0].current);
    assert!(!artifacts[1].current);
    assert!(artifacts[2].current);
    assert!(!labels[0].current);
    assert!(labels[1].current);
    // Re-propagation is a no-op for already-invalidated records.
    let n2 = propagate_quarantine(&mut artifacts, &mut labels, "src-1");
    assert_eq!(n2, 0);
}

#[test]
fn audit_history_survives_invalidation() {
    // Records are invalidated in place, never deleted: id, source, and
    // digest all remain for audit (R-109: immutable audit history).
    let mut artifacts = vec![artifact("a1", "src-1")];
    let mut labels: Vec<MemoryLabel> = vec![];
    propagate_quarantine(&mut artifacts, &mut labels, "src-1");
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].artifact_id, "a1");
    assert_eq!(artifacts[0].source_id, "src-1");
    assert_eq!(artifacts[0].digest, "digest-a1");
}

#[test]
fn twin_run_byte_identical() {
    let mut a1 = vec![artifact("a1", "src-1")];
    let mut a2 = vec![artifact("a1", "src-1")];
    let mut l1: Vec<MemoryLabel> = vec![];
    let mut l2: Vec<MemoryLabel> = vec![];
    propagate_quarantine(&mut a1, &mut l1, "src-1");
    propagate_quarantine(&mut a2, &mut l2, "src-1");
    assert_eq!(
        serde_json::to_string(&a1).unwrap(),
        serde_json::to_string(&a2).unwrap()
    );
}
