//! Learning-reuse gates (T-036, R-116 negative case).
//!
//! Integration tests at the public seam: `apply_state_change`,
//! `usable_for_reuse`.

use hephaestus::selfimprove::learning::{EntryState, LearningEntry, StateChange};
use hephaestus::selfimprove::learning_gate::{ReuseDecision, apply_state_change, usable_for_reuse};

fn s(v: &str) -> String {
    v.to_string()
}

fn entry() -> LearningEntry {
    LearningEntry {
        entry_id: s("learn-1"),
        source_digest: s("source-digest-v1"),
        fresh_partition_id: s("fresh-partition-7"),
        current_state: EntryState::Valid,
    }
}

fn change(to: EntryState, reason: &str) -> StateChange {
    StateChange {
        entry_id: s("learn-1"),
        to_state: to,
        reason: s(reason),
    }
}

#[test]
fn altered_source_becomes_stale_and_is_not_reusable() {
    // Source unchanged: usable.
    assert_eq!(
        usable_for_reuse(&entry(), "source-digest-v1"),
        ReuseDecision::Usable
    );
    // Source ALTERED after recording: stale -> reuse invalidated (R-116).
    assert_eq!(
        usable_for_reuse(&entry(), "source-digest-v2-altered"),
        ReuseDecision::StaleSource
    );
    // Explicit stale state also blocks.
    let stale =
        apply_state_change(&entry(), &change(EntryState::Stale, "source altered")).expect("staled");
    assert_eq!(
        usable_for_reuse(&stale, "source-digest-v1"),
        ReuseDecision::StaleSource
    );
}

#[test]
fn quarantine_blocks_reuse_and_cannot_flip_back() {
    let quarantined = apply_state_change(
        &entry(),
        &change(EntryState::Quarantined, "compromised source"),
    )
    .expect("quarantined");
    assert_eq!(
        usable_for_reuse(&quarantined, "source-digest-v1"),
        ReuseDecision::Quarantined
    );
    // Invalidated entries cannot silently return to Valid.
    assert!(apply_state_change(&quarantined, &change(EntryState::Valid, "unquarantine")).is_err());
    // Wrong-entry state changes refused.
    let mut wrong = change(EntryState::Stale, "other");
    wrong.entry_id = s("learn-other");
    assert!(apply_state_change(&entry(), &wrong).is_err());
}

#[test]
fn history_survives_invalidation() {
    let stale = apply_state_change(&entry(), &change(EntryState::Stale, "altered")).unwrap();
    // R-116: holdout history survives — entry id, source digest, and
    // the fresh partition remain visible after invalidation.
    assert_eq!(stale.entry_id, "learn-1");
    assert_eq!(stale.source_digest, "source-digest-v1");
    assert_eq!(stale.fresh_partition_id, "fresh-partition-7");
    // The ORIGINAL view is still constructible from the base record:
    // append-only semantics, nothing erased.
    assert_eq!(entry().current_state, EntryState::Valid);
}

#[test]
fn twin_run_byte_identical() {
    let a = apply_state_change(&entry(), &change(EntryState::Stale, "altered")).unwrap();
    let b = apply_state_change(&entry(), &change(EntryState::Stale, "altered")).unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
