//! T-047 ticket 01 — review authority service (R-034/AT-034,
//! R-035/AT-035, R-036/AT-036; MASTER_SPEC §12 "Review and Verification
//! services"). Deny-first: votes never substitute the deterministic
//! checker, candidate scopes never touch the evaluator, open blocking
//! objections never advance.

use hephaestus::review::{
    DeterministicOutcome, ReviewRejection, VoteBasis, VoteStance, advance, new_record, record_vote,
};

#[test]
fn at_034_unanimous_approval_never_bypasses_a_deterministic_failure() {
    // R-034 negative case: all model reviewers approve a candidate that
    // carries a deterministic error — the authoritative checker blocks
    // promotion despite agreement.
    let mut rec = new_record("challenger-1", "digest-v1");
    for reviewer in ["rev-a", "rev-b", "rev-c"] {
        record_vote(&mut rec, reviewer, VoteStance::Approve);
    }
    assert_eq!(rec.votes.len(), 3, "votes recorded per subject");
    // Every vote is typed as a judgment (basis is fixed at construction;
    // the record carries no numeric confidence/posterior field at all).
    assert!(
        rec.votes
            .iter()
            .all(|v| matches!(v.basis, VoteBasis::Judgment)),
        "votes are judgments: {rec:?}"
    );

    // Deterministic failure blocks even under unanimous approval.
    let err = advance(
        &rec,
        DeterministicOutcome::Failed {
            reason: "deterministic error".to_string(),
        },
    )
    .expect_err("deterministic failure must block despite unanimous approval");
    assert!(
        matches!(err, ReviewRejection::Deterministic { ref reason } if reason == "deterministic error"),
        "{err:?}"
    );

    // No deterministic evidence at all is equally blocking.
    let err = advance(&rec, DeterministicOutcome::Absent)
        .expect_err("an absent deterministic check must block");
    assert!(
        matches!(err, ReviewRejection::Deterministic { .. }),
        "{err:?}"
    );

    // Passed check + no open blocking objections advances; the tally was
    // never consulted on any path.
    let receipt = advance(
        &rec,
        DeterministicOutcome::Passed {
            receipt_digest: "rcpt-1".to_string(),
        },
    )
    .expect("passed deterministic check advances");
    assert_eq!(receipt.subject, "challenger-1");
    assert_eq!(receipt.receipt_digest, "rcpt-1");
}

#[test]
fn at_035_evaluator_edit_is_scope_denied_with_an_audit_event() {
    // R-035 negative case: attempt to edit the evaluator from a
    // candidate workspace — the write is denied AND an audit event is
    // recorded.
    use hephaestus::review::{Authority, request_evaluator_edit};

    let mut rec = new_record("challenger-1", "digest-v1");
    let err = request_evaluator_edit(&mut rec, Authority::CandidateWorkspace, "evaluator-v2")
        .expect_err("candidate workspace must never edit the evaluator");
    assert!(
        matches!(err, ReviewRejection::AuthorityDenied { .. }),
        "{err:?}"
    );
    let ev = rec.audit.last().expect("audit event recorded");
    assert_eq!(ev.action, "evaluator_edit_request");
    assert_eq!(ev.subject_digest, "digest-v1", "event is version-bound");
    assert_eq!(ev.requested.as_deref(), Some("evaluator-v2"));
    assert_eq!(ev.outcome, "denied");

    // Authority separation: generator and promoter are denied too, each
    // with its own audit event.
    for authority in [Authority::Generator, Authority::Promoter] {
        let err = request_evaluator_edit(&mut rec, authority, "evaluator-v2")
            .expect_err("only the analyzer scope may even register a proposal");
        assert!(
            matches!(err, ReviewRejection::AuthorityDenied { .. }),
            "{err:?}"
        );
    }

    // The analyzer registers a PROPOSAL only — this service exposes no
    // evaluator-mutation surface at all.
    request_evaluator_edit(&mut rec, Authority::Analyzer, "evaluator-v3")
        .expect("analyzer proposal registers");
    assert_eq!(rec.evaluator_edit_proposals.len(), 1);
    assert_eq!(rec.audit.len(), 4, "every attempt leaves an audit event");
    assert!(
        rec.audit
            .iter()
            .all(|e| e.outcome == "denied" || e.outcome == "registered")
    );
}

#[test]
fn at_036_objections_are_retained_with_dispositions_and_block_advancement() {
    // R-036 negative case: resolve one reviewer objection and leave
    // another unresolved — both remain visible, and the unresolved
    // blocking issue prevents advancement.
    use hephaestus::review::{Disposition, raise_objection, resolve_objection};

    let mut rec = new_record("hypothesis-9", "digest-v9");
    let a = raise_objection(&mut rec, "no control arm", "rev-a", true);
    let b = raise_objection(&mut rec, "units undeclared", "rev-b", true);
    resolve_objection(&mut rec, &a, "control arm added in rev-b plan", "rev-a")
        .expect("resolving one objection succeeds");

    // BOTH remain visible with distinct dispositions — resolution never
    // deletes an objection.
    assert_eq!(rec.objections.len(), 2);
    let first = rec.objections.iter().find(|o| o.id == a).expect("a kept");
    let second = rec.objections.iter().find(|o| o.id == b).expect("b kept");
    assert!(
        matches!(first.disposition, Disposition::Resolved { .. }),
        "a resolved: {first:?}"
    );
    assert!(
        matches!(second.disposition, Disposition::Open),
        "b still open: {second:?}"
    );

    // The open BLOCKING objection prevents advancement even with a
    // passing deterministic check.
    let err = advance(
        &rec,
        DeterministicOutcome::Passed {
            receipt_digest: "rcpt-9".to_string(),
        },
    )
    .expect_err("open blocking objection prevents advancement");
    match err {
        ReviewRejection::OpenObjections { ids } => assert_eq!(ids, vec![b.clone()]),
        other => panic!("expected OpenObjections, got {other:?}"),
    }

    // A non-blocking open objection alone does not block.
    resolve_objection(&mut rec, &b, "units declared", "rev-b").expect("resolve b");
    let nonblocking = raise_objection(&mut rec, "wording nit", "rev-c", false);
    let receipt = advance(
        &rec,
        DeterministicOutcome::Passed {
            receipt_digest: "rcpt-9".to_string(),
        },
    )
    .expect("only a non-blocking objection remains");
    assert_eq!(receipt.subject, "hypothesis-9");
    let kept = rec
        .objections
        .iter()
        .find(|o| o.id == nonblocking)
        .expect("kept");
    assert!(matches!(kept.disposition, Disposition::Open));
}
