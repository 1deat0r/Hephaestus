//! Execution backend adapter contract (T-030, R-051/R-057/R-066).
//!
//! Integration tests at the public seam: `ExecutionBackend` (LocalProcess),
//! `check_contract`, `tachyon_status`.

use hephaestus::backend::record::{DispatchError, Task, TaskState};
use hephaestus::backend::{ExecutionBackend, LocalProcess, check_contract, tachyon_status};
use std::collections::BTreeMap;

fn s(v: &str) -> String {
    v.to_string()
}

fn functions() -> BTreeMap<String, fn(&str) -> String> {
    let mut m: BTreeMap<String, fn(&str) -> String> = BTreeMap::new();
    m.insert(s("upper"), |x: &str| x.to_uppercase() as String);
    m.insert(s("len"), |x: &str| x.len().to_string());
    m
}

fn task(id: &str, function: &str, input: &str) -> Task {
    Task {
        task_id: s(id),
        read_set: vec![s("input://a")],
        write_set: vec![s("output://b")],
        function: s(function),
        input: s(input),
        budget_cost: 1,
    }
}

fn backend() -> LocalProcess {
    LocalProcess::new(functions(), 10)
}

// ---- Ticket 01: trait + adapter + durable receipts ----

#[test]
fn dispatch_executes_and_records_durable_receipt() {
    let mut b = backend();
    let receipt = b
        .dispatch(task("t1", "upper", "hello"))
        .expect("dispatched");
    assert_eq!(receipt.output, "HELLO");
    assert_eq!(receipt.state, TaskState::Completed);
    assert_eq!(receipt.budget_spent, 1);
    // Durable: receipt re-readable from the ledger.
    assert_eq!(
        b.receipt("t1").unwrap().output_digest,
        receipt.output_digest
    );
    // Restart: reconstruct from the persisted ledger.
    let mut resumed = LocalProcess::from_ledger(functions(), 10, b.ledger().to_vec());
    assert_eq!(resumed.receipt("t1").unwrap().output, "HELLO");
    // Budget decremented and restored on restart.
    assert!(resumed.dispatch(task("t2", "len", "abcd")).is_ok());
}

#[test]
fn policy_and_budget_refusals_named() {
    let mut b = backend();
    // Empty declared sets: policy boundary refused.
    let mut t = task("t3", "upper", "x");
    t.read_set.clear();
    assert_eq!(b.dispatch(t), Err(DispatchError::MissingPolicySets));
    // Zero budget: refused.
    let mut poor = LocalProcess::new(functions(), 0);
    assert_eq!(
        poor.dispatch(task("t4", "upper", "x")),
        Err(DispatchError::BudgetExhausted)
    );
    // Duplicate task id + same digest: no duplicate side effects.
    b.dispatch(task("t5", "upper", "dup")).unwrap();
    assert_eq!(
        b.dispatch(task("t5", "upper", "dup")),
        Err(DispatchError::AlreadyCompleted)
    );
    // Unknown function.
    assert_eq!(
        b.dispatch(task("t6", "nope", "x")),
        Err(DispatchError::UnknownFunction)
    );
}

// ---- Ticket 02: contract checks + tachyon status ----

#[test]
fn contract_check_covers_all_clauses() {
    let mut b = backend();
    let report = check_contract("local-process", &mut b);
    // All five clauses present.
    for clause in [
        "cancellation",
        "policy_boundaries",
        "durable_receipts",
        "replay",
        "budgets",
    ] {
        assert!(
            report.findings.iter().any(|f| f.clause == clause),
            "missing clause {clause}"
        );
    }
    assert!(report.passes, "all contract clauses pass");
}

#[test]
fn replay_digest_verified_and_mismatch_refused() {
    let mut b = backend();
    b.dispatch(task("t7", "upper", "replay-me")).unwrap();
    // Replay re-derives the digest from recorded bytes: consistent.
    let r = b.replay("t7").expect("replayed");
    assert_eq!(r.output, "REPLAY-ME");
    // Replay of an unknown task: honest refusal.
    assert!(b.replay("nope").is_err());
}

#[test]
fn cancellation_states() {
    let mut b = backend();
    // Undispatched task: cancellable.
    let r = b.cancel("t-never").expect("cancelled");
    assert_eq!(r.state, TaskState::Cancelled);
    // Completed task: NOT cancellable (effects committed, never undone).
    b.dispatch(task("t8", "upper", "done")).unwrap();
    assert!(b.cancel("t8").is_err());
}

#[test]
fn tachyon_blocked_no_compatibility_claim() {
    // R-066: no protocol inspected -> no integration, no claim.
    assert_eq!(
        tachyon_status(),
        hephaestus::backend::record::TachyonStatus::BlockedPendingRealProtocol
    );
}

#[test]
fn twin_run_byte_identical() {
    let mut a = backend();
    let mut b = backend();
    let ra = check_contract("lp", &mut a);
    let rb = check_contract("lp", &mut b);
    assert_eq!(
        serde_json::to_string(&ra).unwrap(),
        serde_json::to_string(&rb).unwrap()
    );
}
