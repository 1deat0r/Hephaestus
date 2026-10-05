//! AE-01 S1 — concurrent executor contracts (R-051/R-055/R-056/R-057).
//!
//! Bounded submission, completion identities, cancellation, and
//! ambiguous-effect outcomes. Full overlap lands in S2; recovery in S3.

use std::collections::HashSet;

use hephaestus::scheduler::{
    Cancellation, Completion, ExecutorContractError, SubmissionGate, normalize_ambiguous_reason,
};

#[test]
fn s1_contract_bounded_submission_admits_below_and_refuses_above() {
    // Positive: slots below the bound admit with identity + slot.
    let gate = SubmissionGate::new(2, "in-flight").expect("gate");
    let sub = gate.admit("A", 0).expect("slot 0 admits");
    assert_eq!(sub.task_id, "A");
    assert_eq!(sub.slot, 0);
    let top = gate.admit("B", 1).expect("slot 1 admits");
    assert_eq!(top.slot, 1);
    // Refusal: slot at/past the bound is refused before any dispatch.
    assert_eq!(
        gate.admit("C", 2),
        Err(ExecutorContractError::SlotOverBound { slot: 2, bound: 2 })
    );
    // Refusal: a zero bound could never admit work.
    assert_eq!(
        SubmissionGate::new(0, "in-flight"),
        Err(ExecutorContractError::ZeroBound {
            what: "in-flight".to_string()
        })
    );
}

#[test]
fn s1_contract_completion_identities_separate_retries_from_duplicates() {
    // Positive: distinct attempts at one task are retries.
    let first = Completion::new("A", 1).expect("attempt 1");
    let second = Completion::new("A", 2).expect("attempt 2");
    assert!(second.is_retry_of(&first));
    assert!(!second.is_duplicate_of(&first));
    // Duplicate: re-stating the identity is a replay, never a new effect.
    let replay = Completion::new("A", 1).expect("attempt 1 again");
    assert!(replay.is_duplicate_of(&first));
    assert!(!replay.is_retry_of(&first), "same attempt is not a retry");
    // Completion identities collect into a duplicate-detecting set.
    let mut seen = HashSet::new();
    assert!(seen.insert(first.clone()));
    assert!(!seen.insert(replay), "replay is a duplicate");
    // Refusal: empty ids and zero attempts carry no identity.
    assert_eq!(
        Completion::new("", 1),
        Err(ExecutorContractError::EmptyTaskId)
    );
    assert_eq!(
        Completion::new("A", 0),
        Err(ExecutorContractError::ZeroAttempt)
    );
    assert_eq!(
        SubmissionGate::new(2, "in-flight")
            .expect("gate")
            .admit("", 0),
        Err(ExecutorContractError::EmptyTaskId)
    );
}

#[test]
fn s1_contract_cancellation_signals_and_ambiguous_reasons_default() {
    // Positive: an unsignalled guard lets work proceed.
    let guard = Cancellation::new();
    assert!(!guard.is_cancelled());
    assert!(guard.check("A").is_ok());
    // Cancellation: the signal is observed and reported fail-closed.
    guard.signal();
    assert!(guard.is_cancelled());
    assert_eq!(
        guard.check("A"),
        Err(ExecutorContractError::Cancelled {
            task_id: "A".to_string()
        })
    );
    // Regression: blank ambiguous reasons become an explicit default,
    // matching the scheduler's unresolved limb (never a silent empty).
    assert_eq!(
        normalize_ambiguous_reason(""),
        "unspecified ambiguous effect"
    );
    assert_eq!(
        normalize_ambiguous_reason("   "),
        "unspecified ambiguous effect"
    );
    assert_eq!(
        normalize_ambiguous_reason("unknown submit"),
        "unknown submit"
    );
}
