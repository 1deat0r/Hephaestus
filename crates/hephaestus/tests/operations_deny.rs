//! T-011 ticket 01 — durable operation recorder and effect receipts
//! (deny-first, seam: `hephaestus::operations`).
//!
//! R-057/AT-057: recovery preserves audit history — which requires the
//! history to exist: recorded BEFORE dispatch, receipts content-addressed.
//! MASTER_SPEC:369.

use hephaestus::contracts::generated::Money;
use hephaestus::operations::{OperationRecorder, Receipt, RecordingExecutor};
use hephaestus::scheduler::{RetryPolicy, Task, TaskExecutor, TaskOutcome};

fn usd(minor_units: i64) -> Money {
    Money {
        currency: "USD".to_string(),
        minor_units,
    }
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("t011-{}-{}-{}", tag, std::process::id(), nanos));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn task(id: &str) -> Task {
    Task {
        id: id.to_string(),
        priority: hephaestus::scheduler::PriorityClass::Normal,
        depends_on: vec![],
        inputs: vec![],
        outputs: vec![format!("out-{id}")],
        retry: RetryPolicy { max_attempts: 1 },
        timeout_ms: 1_000,
        cost: usd(0),
        trivial: false,
        retryable: true,
        resource_units: 0,
        exclusive: false,
    }
}

/// Inner executor that records whether it was invoked, per task id.
struct SpyExecutor {
    called: std::sync::Mutex<Vec<String>>,
    fail_on: Option<String>,
}

impl TaskExecutor for SpyExecutor {
    fn run(&self, task: &Task) -> TaskOutcome {
        self.called.lock().unwrap().push(task.id.clone());
        if self.fail_on.as_deref() == Some(task.id.as_str()) {
            return TaskOutcome::Failed {
                reason: "boom".to_string(),
            };
        }
        TaskOutcome::Succeeded {
            cost: task.cost.clone(),
        }
    }
}

fn receipt() -> Receipt {
    Receipt {
        outcome: "Succeeded".to_string(),
        cost: usd(0),
        wall_ms: 12,
        reason: None,
        attempt: 1,
        executor: "spy".to_string(),
        artifacts: vec![],
    }
}

#[test]
fn planned_is_durable_before_any_dispatch_possibility() {
    let dir = temp_dir("plan");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    let op = {
        let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
        rec.plan("taskA").expect("plan")
    }; // crash: recorder dropped, nothing dispatched
    let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
    let planned: Vec<String> = rec
        .events_of_type("operation.planned")
        .iter()
        .map(|e| e.operation_id.clone())
        .collect();
    assert!(planned.contains(&op), "planned event durable");
    assert!(
        rec.events_of_type("operation.dispatched")
            .iter()
            .all(|e| e.operation_id != op),
        "nothing dispatched for a mere plan"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recording_executor_appends_dispatched_before_delegating() {
    // MASTER_SPEC:369 — the event exists even when the effect fails.
    let dir = temp_dir("dispatch-order");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    let recorder = OperationRecorder::open(&ledger, &store).expect("open");
    let spy = SpyExecutor {
        called: std::sync::Mutex::new(vec![]),
        fail_on: Some("T-FAIL".into()),
    };
    let mut rec = RecordingExecutor::new(spy, recorder);
    let op_ok = rec.plan_task("T-OK").expect("plan");
    let op_fail = rec.plan_task("T-FAIL").expect("plan");

    let out_ok = rec.run(&task("T-OK"));
    assert!(matches!(out_ok, TaskOutcome::Succeeded { .. }));
    // Failing effect still produces dispatched + receipt events.
    let out_fail = rec.run(&task("T-FAIL"));
    assert!(matches!(out_fail, TaskOutcome::Failed { .. }));

    // Both planned ops got dispatched + receipt events (order in ledger).
    let dispatched: Vec<String> = rec
        .events_of_type("operation.dispatched")
        .iter()
        .map(|e| e.operation_id.clone())
        .collect();
    assert!(dispatched.contains(&op_ok) && dispatched.contains(&op_fail));
    // Receipts are content-addressed: payload event carries a 64-hex hash.
    for op in [&op_ok, &op_fail] {
        let ev = rec.receipt_events_for(op).expect("receipt event");
        assert_eq!(ev.payload_sha256.len(), 64, "payload hash present");
    }
    // The failing effect STILL got both events: record-before-effect holds
    // in both directions (dispatched precedes the failing delegate call).
    assert_eq!(
        rec.events_of_type("operation.receipt").len(),
        2,
        "one receipt per dispatched op"
    );
    // Unplanned task: refused WITHOUT any dispatched event.
    let out = rec.run(&task("T-UNPLANNED"));
    assert!(matches!(out, TaskOutcome::Failed { reason } if reason.contains("OPS_UNPLANNED")));
    assert_eq!(
        rec.events_of_type("operation.dispatched").len(),
        2,
        "unplanned dispatch recorded nothing"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn receipt_payload_lands_in_the_artifact_store() {
    let dir = temp_dir("receipt-store");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
    let op = rec.plan("t").expect("plan");
    rec.dispatched(&op, 1).expect("dispatch");
    rec.receipt(&op, &receipt()).expect("receipt");
    // The payload is retrievable from the store by its event's hash.
    let ev = rec.receipt_events_for(&op).expect("event");
    let bytes = rec
        .store()
        .read(&ev.payload_sha256)
        .expect("payload readable");
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(parsed["attempt"], 1);
    assert_eq!(parsed["outcome"], "Succeeded");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn crash_between_store_commit_and_event_leaves_orphan_no_receipt() {
    // Plan "before output commit" boundary semantics: events rule.
    let dir = temp_dir("orphan");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    {
        let rec = OperationRecorder::open(&ledger, &store).expect("open");
        // Simulate the crash window: payload committed to the store…
        let staged = rec
            .store()
            .stage(b"{\"outcome\":\"Succeeded\"}")
            .expect("stage");
        rec.store().commit(&staged).expect("commit");
        // …but the recorder dies before appending the receipt event.
    }
    let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
    assert!(
        rec.events_of_type("operation.receipt").is_empty(),
        "no receipt claimed"
    );
    let objects = std::fs::read_dir(store.join("objects")).expect("objects dir exists");
    assert!(objects.count() > 0, "orphan artifact retained (T-006 rule)");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cancel_requested_is_recorded_before_the_handle_flips() {
    let dir = temp_dir("cancel");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
    let op = rec.plan("t").expect("plan");
    rec.cancel_requested(&op).expect("record");
    // Crash immediately after: reopen shows the intent durably.
    drop(rec);
    let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
    let cancels: Vec<String> = rec
        .events_of_type("operation.cancel_requested")
        .iter()
        .map(|e| e.operation_id.clone())
        .collect();
    assert!(cancels.contains(&op), "durable cancellation intent");
    let _ = std::fs::remove_dir_all(&dir);
}

// --- Ticket 02: replay view from the verified chain ---

#[test]
fn replay_derives_states_from_events_alone() {
    let dir = temp_dir("replay-states");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    let mut rec = OperationRecorder::open(&ledger, &store).expect("open");

    let planned_only = rec.plan("just-planned").expect("plan");

    let dispatched_amb = rec.plan("amb").expect("plan");
    rec.dispatched(&dispatched_amb, 1).expect("dispatch");

    let succeeded = rec.plan("ok").expect("plan");
    rec.dispatched(&succeeded, 1).expect("dispatch");
    rec.receipt(&succeeded, &receipt()).expect("receipt");

    let mut failed_receipt = receipt();
    failed_receipt.outcome = "Failed".to_string();
    failed_receipt.reason = Some("boom".to_string());
    let failed = rec.plan("bad").expect("plan");
    rec.dispatched(&failed, 1).expect("dispatch");
    rec.receipt(&failed, &failed_receipt).expect("receipt");

    let mut ambiguous_receipt = receipt();
    ambiguous_receipt.outcome = "AmbiguousEffect".to_string();
    ambiguous_receipt.reason = Some("unknown submit".to_string());
    let ambiguous = rec.plan("amb-effect").expect("plan");
    rec.dispatched(&ambiguous, 1).expect("dispatch");
    rec.receipt(&ambiguous, &ambiguous_receipt)
        .expect("receipt");

    let receipt_no_dispatch = rec.plan("forged").expect("plan");
    rec.receipt(&receipt_no_dispatch, &receipt())
        .expect("receipt without dispatch");

    let cancelled = rec.plan("to-cancel").expect("plan");
    rec.cancel_requested(&cancelled).expect("cancel");

    let view = rec.replay();
    assert_eq!(
        format!("{:?}", view.state_of(&planned_only).unwrap()),
        "Planned"
    );
    assert_eq!(
        format!("{:?}", view.state_of(&dispatched_amb).unwrap()),
        "Ambiguous"
    );
    assert_eq!(
        format!("{:?}", view.state_of(&succeeded).unwrap()),
        "Succeeded"
    );
    assert_eq!(format!("{:?}", view.state_of(&failed).unwrap()), "Failed");
    assert_eq!(
        format!("{:?}", view.state_of(&ambiguous).unwrap()),
        "Ambiguous"
    );
    assert_eq!(
        format!("{:?}", view.state_of(&receipt_no_dispatch).unwrap()),
        "Corrupt",
        "receipt without dispatch is impossible history"
    );
    assert_eq!(
        format!("{:?}", view.state_of(&cancelled).unwrap()),
        "Cancelled"
    );
    assert!(view.cancel_requested(&cancelled));
    // Receipts parsed from the store through payload_sha256.
    let r = view.receipt_for(&failed).expect("parsed receipt");
    assert_eq!(r.outcome, "Failed");
    assert_eq!(r.reason.as_deref(), Some("boom"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn replay_is_deterministic_across_reopens() {
    // R-051 spirit: same ledger bytes => same view, byte-identical.
    let dir = temp_dir("replay-determinism");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    {
        let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
        let a = rec.plan("a").expect("plan");
        rec.dispatched(&a, 1).expect("d");
        rec.receipt(&a, &receipt()).expect("r");
        let b = rec.plan("b").expect("plan");
        rec.dispatched(&b, 1).expect("d");
    }
    let first = {
        let rec = OperationRecorder::open(&ledger, &store).expect("open");
        serde_json::to_string(&rec.replay()).expect("serialize")
    };
    let second = {
        let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
        serde_json::to_string(&rec.replay()).expect("serialize")
    };
    assert_eq!(first, second, "byte-identical replay across reopens");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tampered_chain_refuses_replay_entirely() {
    // Fail-closed: corrupted history cannot be replayed at all.
    let dir = temp_dir("replay-corrupt-chain");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    {
        let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
        let a = rec.plan("a").expect("plan");
        rec.dispatched(&a, 1).expect("d");
    }
    let raw = std::fs::read(&ledger).expect("read");
    // Flip a byte inside line 1's payload (same length).
    let mut tampered = raw.clone();
    let pos = tampered
        .windows(9)
        .position(|w| w == b"operation")
        .expect("event type present");
    tampered[pos] = b'x';
    std::fs::write(&ledger, tampered).expect("write tampered");
    assert!(
        OperationRecorder::open(&ledger, &store).is_err(),
        "tampered chain must refuse to open (replay impossible)"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// --- Ticket 03: recovery plan with budget reconciliation ---

use hephaestus::operations::{RetryPosture, apply, recover};

#[test]
fn rule_table_covers_every_state() {
    let dir = temp_dir("rule-table");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    let mut rec = OperationRecorder::open(&ledger, &store).expect("open");

    let planned = rec.plan("p").expect("plan");

    let amb_retry = rec.plan("amb-retry").expect("plan");
    rec.dispatched(&amb_retry, 1).expect("d");

    let amb_noretry = rec.plan("amb-noretry").expect("plan");
    rec.dispatched(&amb_noretry, 1).expect("d");

    let amb_exhausted = rec.plan("amb-exhausted").expect("plan");
    rec.dispatched(&amb_exhausted, 1).expect("d");

    let ok = rec.plan("ok").expect("plan");
    rec.dispatched(&ok, 1).expect("d");
    rec.receipt(&ok, &receipt()).expect("r");

    let cancelled = rec.plan("c").expect("plan");
    rec.cancel_requested(&cancelled).expect("cancel");

    let corrupt = rec.plan("corrupt").expect("plan");
    rec.receipt(&corrupt, &receipt())
        .expect("receipt w/o dispatch");

    let dispatched_then_cancel = rec.plan("d-then-c").expect("plan");
    rec.dispatched(&dispatched_then_cancel, 1).expect("d");
    rec.cancel_requested(&dispatched_then_cancel)
        .expect("cancel");

    // AmbiguousEffect receipt also rests as Ambiguous (grill Q6).
    let mut amb_receipt = receipt();
    amb_receipt.outcome = "AmbiguousEffect".to_string();
    amb_receipt.reason = Some("unknown".to_string());
    let amb_effect = rec.plan("amb-effect").expect("plan");
    rec.dispatched(&amb_effect, 1).expect("d");
    rec.receipt(&amb_effect, &amb_receipt).expect("r");
    let view = rec.replay();

    // Op ids carry tags as suffixes (OP-<n>-<tag>).
    let plan = recover(&view, |op| {
        if op.ends_with("amb-retry") {
            RetryPosture {
                retryable: true,
                max_attempts: 3,
            }
        } else if op.ends_with("amb-exhausted") {
            RetryPosture {
                retryable: true,
                max_attempts: 1,
            }
        } else {
            RetryPosture {
                retryable: false,
                max_attempts: 1,
            }
        }
    });

    assert!(plan.requeue.contains(&planned), "Planned => requeue");
    assert!(
        plan.requeue.contains(&amb_retry),
        "retryable+attempts<max => requeue"
    );
    assert!(
        plan.unresolved.contains(&amb_noretry),
        "non-idempotent => unresolved"
    );
    assert!(
        plan.unresolved.contains(&amb_exhausted),
        "exhausted => unresolved"
    );
    assert!(
        plan.unresolved.contains(&amb_effect),
        "ambiguous-effect receipt => unresolved"
    );
    assert!(plan.terminal.contains(&ok), "Succeeded => terminal");
    assert!(
        plan.cancelled.contains(&cancelled),
        "cancel-requested => cancelled"
    );
    assert!(
        plan.corrupt.contains(&corrupt),
        "receipt-without-dispatch => corrupt"
    );
    assert!(
        plan.unresolved.contains(&dispatched_then_cancel),
        "dispatched+cancel stays Ambiguous (uncertain effect never hides)"
    );
    assert!(!plan.cancelled.contains(&dispatched_then_cancel));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn apply_records_unresolved_exactly_once_and_releases_holds() {
    let dir = temp_dir("apply-once");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    let mut rec = OperationRecorder::open(&ledger, &store).expect("open");

    // Op with a receipt carrying cost: hold exists (in-process simulation).
    let with_cost = rec.plan("costly").expect("plan");
    rec.dispatched(&with_cost, 1).expect("d");
    let mut amb = receipt();
    amb.outcome = "AmbiguousEffect".to_string();
    amb.reason = Some("unknown submit".to_string());
    amb.cost = usd(120);
    rec.receipt(&with_cost, &amb).expect("r");

    // Op without any receipt: amount unknown => null-with-reason entry.
    let no_receipt = rec.plan("silent").expect("plan");
    rec.dispatched(&no_receipt, 1).expect("d");

    let view = rec.replay();
    let plan = recover(&view, |op| match op {
        "costly" => RetryPosture {
            retryable: false,
            max_attempts: 1,
        },
        _ => RetryPosture {
            retryable: false,
            max_attempts: 1,
        },
    });

    let mut budget = hephaestus::budget::BudgetLedger::new(usd(1000)).expect("budget");
    // Simulate the hold the crashed run left behind:
    budget.reserve(&with_cost, usd(120)).expect("hold");
    budget.reserve(&no_receipt, usd(50)).expect("hold2");

    let recorded = apply(&plan, &view, &mut budget).expect("apply");
    assert!(recorded.contains(&with_cost) && recorded.contains(&no_receipt));
    assert_eq!(budget.reserved(), usd(0), "holds released during apply");
    assert_eq!(budget.unresolved(), usd(120), "known amount held");
    assert_eq!(
        budget.unresolved_count(),
        2,
        "both entries (one amount-less)"
    );
    assert!(
        budget.available().minor_units >= 0,
        "four-bucket invariant never negative"
    );

    // Idempotent: second apply must not double-record (AT-056 discipline).
    let recorded2 = apply(&plan, &view, &mut budget).expect("apply again");
    assert!(recorded2.is_empty(), "nothing new on re-apply");
    assert_eq!(budget.unresolved_count(), 2, "no duplicates");
    let _ = std::fs::remove_dir_all(&dir);
}

// --- Ticket 04: fault injection at the plan's boundaries + e2e ---

#[test]
fn fault_boundary_planned_only_requeues_and_touches_no_budget() {
    // Boundary 1: crash after plan, before dispatch.
    let dir = temp_dir("fault-planned");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    {
        let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
        rec.plan("victim").expect("plan");
    } // crash
    let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
    let view = rec.replay();
    let plan = recover(&view, |_| RetryPosture {
        retryable: false,
        max_attempts: 1,
    });
    assert_eq!(plan.requeue.len(), 1, "requeued: {:?}", plan.requeue);
    let mut budget = hephaestus::budget::BudgetLedger::new(usd(100)).expect("b");
    let recorded = apply(&plan, &view, &mut budget).expect("apply");
    assert!(recorded.is_empty(), "nothing unresolved to record");
    assert_eq!(budget.unresolved_count(), 0, "budget untouched");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fault_boundary_dispatched_without_receipt_becomes_unresolved() {
    // Boundary 2: crash after dispatch, before receipt.
    let dir = temp_dir("fault-dispatched");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    {
        let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
        let op = rec.plan("midflight").expect("plan");
        rec.dispatched(&op, 1).expect("dispatch");
    } // crash
    let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
    let view = rec.replay();
    assert_eq!(
        format!("{:?}", view.state_of("OP-0-midflight").unwrap()),
        "Ambiguous"
    );
    let plan = recover(&view, |_| RetryPosture {
        retryable: false,
        max_attempts: 1,
    });
    assert_eq!(plan.unresolved.len(), 1);
    let mut budget = hephaestus::budget::BudgetLedger::new(usd(100)).expect("b");
    budget
        .reserve("OP-0-midflight", usd(40))
        .expect("stale hold");
    let recorded = apply(&plan, &view, &mut budget).expect("apply");
    assert_eq!(recorded.len(), 1);
    assert_eq!(budget.reserved(), usd(0), "stale hold released");
    assert_eq!(
        budget.unresolved_count(),
        1,
        "amount-less entry (no receipt)"
    );
    assert_eq!(
        budget.unresolved(),
        usd(0),
        "null-with-reason, not a zero-claim"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fault_boundary_artifact_committed_without_receipt_stays_ambiguous() {
    // Boundary 3: artifact store committed, crash before receipt event.
    let dir = temp_dir("fault-artifact");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    {
        let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
        let op = rec.plan("output").expect("plan");
        rec.dispatched(&op, 1).expect("dispatch");
        let staged = rec.store().stage(b"result bytes").expect("stage");
        rec.store().commit(&staged).expect("commit output");
        let digest = staged.digest().to_string();
        // Would append output_committed next — crash before any receipt.
        let _ = digest;
    } // crash
    let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
    let view = rec.replay();
    assert_eq!(
        format!("{:?}", view.state_of("OP-0-output").unwrap()),
        "Ambiguous",
        "events rule: committed artifact never claims completion"
    );
    assert!(
        rec.events_of_type("operation.receipt").is_empty(),
        "no receipt was claimed"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fault_boundary_ambiguous_effect_reconciles_exactly_once() {
    // Boundary 4: executor said unknown, crash before reconciled event.
    let dir = temp_dir("fault-ambiguous");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    {
        let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
        let op = rec.plan("external").expect("plan");
        rec.dispatched(&op, 1).expect("dispatch");
        let mut r = receipt();
        r.outcome = "AmbiguousEffect".to_string();
        r.reason = Some("unknown submit".to_string());
        r.cost = usd(75);
        rec.receipt(&op, &r).expect("receipt");
        // Would append reconciled next — crash before it.
    } // crash
    let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
    let view = rec.replay();
    let plan = recover(&view, |_| RetryPosture {
        retryable: false,
        max_attempts: 1,
    });
    let mut budget = hephaestus::budget::BudgetLedger::new(usd(500)).expect("b");
    let first = apply(&plan, &view, &mut budget).expect("apply 1");
    assert_eq!(first.len(), 1);
    assert_eq!(budget.unresolved(), usd(75), "receipt cost held");
    let second = apply(&plan, &view, &mut budget).expect("apply 2 (idempotent)");
    assert!(second.is_empty(), "never duplicated");
    assert_eq!(budget.unresolved(), usd(75), "still exactly one entry");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fault_boundary_cancel_intent_recovers_as_cancelled_not_requeue() {
    // Boundary 5: crash right after cancel_requested.
    let dir = temp_dir("fault-cancel");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    {
        let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
        let op = rec.plan("stoppable").expect("plan");
        rec.cancel_requested(&op).expect("cancel");
    } // crash
    let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
    let view = rec.replay();
    let plan = recover(&view, |_| RetryPosture {
        retryable: true,
        max_attempts: 5,
    });
    assert_eq!(plan.cancelled.len(), 1, "cancelled: {:?}", plan.cancelled);
    assert!(
        plan.requeue.is_empty(),
        "a durable cancel intent is not requeued"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn end_to_end_recorder_survives_death_with_audit_history_intact() {
    // AT-057: recovery preserves audit history — the full story through a
    // decorated executor, then death, then reopen.
    let dir = temp_dir("e2e");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    {
        let recorder = OperationRecorder::open(&ledger, &store).expect("open");
        let spy = SpyExecutor {
            called: std::sync::Mutex::new(vec![]),
            fail_on: None,
        };
        let mut rec = RecordingExecutor::new(spy, recorder);
        let op = rec.plan_task("JOB").expect("plan");
        let out = rec.run(&task("JOB"));
        assert!(matches!(out, TaskOutcome::Succeeded { .. }));
        assert_eq!(rec.events_of_type("operation.planned").len(), 1);
        assert_eq!(rec.events_of_type("operation.dispatched").len(), 1);
        assert_eq!(rec.events_of_type("operation.receipt").len(), 1);
        let _ = op;
    } // death
    let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
    let view = rec.replay();
    assert_eq!(
        format!("{:?}", view.state_of("OP-0-JOB").unwrap()),
        "Succeeded"
    );
    let plan = recover(&view, |_| RetryPosture {
        retryable: false,
        max_attempts: 1,
    });
    assert_eq!(
        plan.terminal.len(),
        1,
        "audit history preserved => terminal"
    );
    assert!(plan.requeue.is_empty() && plan.unresolved.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// --- Review pass 1 remediation tests ---

#[test]
fn reopen_continues_monotonic_ids_and_sequences() {
    // T01 AC gap: re-planning after reopen must never collide.
    let dir = temp_dir("reopen-ids");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    let first = {
        let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
        rec.plan("alpha").expect("plan")
    };
    let (second, third) = {
        let mut rec = OperationRecorder::open(&ledger, &store).expect("reopen");
        let a = rec.plan("beta").expect("replan");
        let b = rec.plan("gamma").expect("replan");
        (a, b)
    };
    assert_ne!(first, second);
    assert_ne!(second, third);
    assert!(
        second.starts_with("OP-1-"),
        "continues past {first}: {second}"
    );
    assert!(third.starts_with("OP-2-"));
    // Replay still sees all three (chain intact across reopen).
    let rec = OperationRecorder::open(&ledger, &store).expect("reopen2");
    let view = rec.replay();
    assert_eq!(view.states.len(), 3, "{:?}", view.states);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn overlong_operation_tags_are_refused_by_contract_validation() {
    // Standards pass 1: the <=64 event-id bound is enforced, not just
    // documented (Event::validate refuses the append).
    let dir = temp_dir("long-tag");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
    let long_tag = "x".repeat(80);
    assert!(
        rec.plan(&long_tag).is_err(),
        "overlong tag must be refused at append (contract id bound)"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn scheduler_dag_runs_through_recording_executor_with_budget() {
    // T04 AC: scheduler DAG + recorder + ledger + budget consistent; then
    // death, reopen, replay preserves audit history (AT-057).
    use hephaestus::budget::BudgetLedger;
    use hephaestus::scheduler::{Scheduler, SchedulerConfig, TaskDag};

    let dir = temp_dir("sched-e2e");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    let mut dag = TaskDag::new(vec![]);
    let mut priced = task("PRICED-TASK");
    priced.cost = usd(200);
    dag.add(priced).expect("add");
    dag.validate().expect("valid dag");

    let recorder = OperationRecorder::open(&ledger, &store).expect("open");
    let spy = SpyExecutor {
        called: std::sync::Mutex::new(vec![]),
        fail_on: None,
    };
    let mut recording = RecordingExecutor::new(spy, recorder);
    let op = recording.plan_task("PRICED-TASK").expect("plan");
    let mut sched = Scheduler::new(
        dag,
        BudgetLedger::new(usd(1000)).expect("budget"),
        SchedulerConfig {
            max_in_flight: 2,
            resource_capacity: 4,
        },
    )
    .expect("scheduler");
    let report = sched.run(&recording).expect("run");
    assert_eq!(report.succeeded().len(), 1);
    assert_eq!(sched.budget().spent(), usd(200), "reserve -> commit exact");
    drop(sched);
    drop(recording); // death

    let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
    let view = rec.replay();
    assert_eq!(
        format!("{:?}", view.state_of(&op).unwrap()),
        "Succeeded",
        "audit history survives the crash"
    );
    let plan = recover(&view, |_| RetryPosture {
        retryable: false,
        max_attempts: 1,
    });
    assert!(plan.terminal.contains(&op));
    assert!(plan.requeue.is_empty() && plan.unresolved.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
