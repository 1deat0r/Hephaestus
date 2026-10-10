//! AE-01 S1 — concurrent executor contracts (R-051/R-055/R-056/R-057).
//!
//! Bounded submission, completion identities, cancellation, and
//! ambiguous-effect outcomes. Full overlap lands in S2; recovery in S3.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use hephaestus::budget::BudgetLedger;
use hephaestus::contracts::generated::Money;
use hephaestus::operations::{OperationRecorder, Receipt, RetryPosture, apply, recover};
use hephaestus::scheduler::{
    Cancellation, Completion, ExecutorContractError, PriorityClass, RetryPolicy, Scheduler,
    SchedulerConfig, SubmissionGate, Task, TaskDag, TaskExecutor, TaskOutcome,
    normalize_ambiguous_reason,
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

// --- AE-01/S2 --- dispatch without wave barriers (AC2) ----------------

fn usd(minor_units: i64) -> Money {
    Money {
        currency: "USD".to_string(),
        minor_units,
    }
}

/// One non-trivial task over the public scheduler seam. Declared inputs
/// follow the dependency outputs, as in the other scheduler suites.
fn s2_task(id: &str, depends_on: &[&str]) -> Task {
    Task {
        id: id.to_string(),
        priority: PriorityClass::Normal,
        depends_on: depends_on.iter().map(|s| (*s).to_string()).collect(),
        inputs: depends_on.iter().map(|s| format!("out-{s}")).collect(),
        outputs: vec![format!("out-{id}")],
        retry: RetryPolicy { max_attempts: 1 },
        timeout_ms: 1_000,
        cost: usd(0),
        trivial: false,
        retryable: true,
        resource_units: 0,
        memory_mb: 0,
        exclusive: false,
    }
}

/// Overlap probe: UNRELATED stays inside run() until CHILD starts, or a
/// bounded 3 s deadline expires. A wave barrier therefore fails the
/// observable assertion instead of deadlocking.
struct OverlapProbe {
    child_started: AtomicBool,
    child_started_while_unrelated_active: AtomicBool,
}

impl OverlapProbe {
    fn new() -> Self {
        OverlapProbe {
            child_started: AtomicBool::new(false),
            child_started_while_unrelated_active: AtomicBool::new(false),
        }
    }
}

impl TaskExecutor for OverlapProbe {
    fn run(&self, task: &Task) -> TaskOutcome {
        match task.id.as_str() {
            "PARENT" => TaskOutcome::Succeeded {
                cost: task.cost.clone(),
            },
            "CHILD" => {
                self.child_started.store(true, Ordering::SeqCst);
                TaskOutcome::Succeeded {
                    cost: task.cost.clone(),
                }
            }
            _ => {
                // Unrelated earlier work: remain active until the child
                // starts; the bounded deadline keeps failure terminating.
                let deadline = Instant::now() + Duration::from_secs(3);
                loop {
                    if self.child_started.load(Ordering::SeqCst) {
                        self.child_started_while_unrelated_active
                            .store(true, Ordering::SeqCst);
                        return TaskOutcome::Succeeded {
                            cost: task.cost.clone(),
                        };
                    }
                    if Instant::now() >= deadline {
                        return TaskOutcome::Failed {
                            reason: "child never started while unrelated work was active"
                                .to_string(),
                        };
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        }
    }
}

/// AE-01/S2 positive behavior (AC2; R-051 R-055 R-056 R-057): a
/// completed parent releases its child while unrelated earlier work is
/// still active, with one scheduler owner settling every reservation.
#[test]
fn s2_dispatch_child_runs_before_unrelated_completion() {
    let mut dag = TaskDag::new(vec![]);
    let mut parent = s2_task("PARENT", &[]);
    parent.cost = usd(100);
    let mut unrelated = s2_task("UNRELATED", &[]);
    unrelated.cost = usd(25);
    let mut child = s2_task("CHILD", &["PARENT"]);
    child.cost = usd(50);
    for t in [parent, unrelated, child] {
        dag.add(t).expect("add");
    }
    dag.validate().expect("valid");

    let mut sched = Scheduler::new(
        dag,
        BudgetLedger::new(usd(1000)).expect("budget"),
        SchedulerConfig {
            max_in_flight: 4,
            resource_capacity: 4,
            memory_capacity_mb: 1024,
            max_queued: usize::MAX,
        },
    )
    .expect("scheduler");
    let executor = OverlapProbe::new();

    let report = sched
        .run(&executor)
        .expect("run settles within bounded waits");

    // The observable behavior: the child ran while unrelated work was
    // still active. Under the legacy wave barrier this waits out the
    // bounded deadline and fails here.
    assert!(
        executor
            .child_started_while_unrelated_active
            .load(Ordering::SeqCst),
        "child must start while unrelated work is still active;          UNRELATED outcome reason: {:?}; dispatch order: {:?}",
        report.failure_reason("UNRELATED"),
        report.dispatch_order()
    );
    assert_eq!(
        report.dispatch_order(),
        &[
            "PARENT".to_string(),
            "UNRELATED".to_string(),
            "CHILD".to_string()
        ],
        "child dispatches after its parent completes"
    );
    assert_eq!(
        report.succeeded(),
        &[
            "PARENT".to_string(),
            "UNRELATED".to_string(),
            "CHILD".to_string()
        ],
        "every task succeeds"
    );
    assert!(
        report.failed().is_empty(),
        "no task fails, got {:?}",
        report.failed()
    );
    // One owner: each hold settles exactly once (R-055).
    assert_eq!(
        sched.budget().reserved(),
        usd(0),
        "no held reservation survives"
    );
    assert_eq!(sched.budget().spent(), usd(175), "each hold commits once");
}

/// Independent-overlap probe: every task joins a small active set and
/// waits (bounded) for the other independent tasks to run alongside it.
/// An over-dispatching scheduler shows a peak above the configured
/// bound; a barrier shows a peak of one.
struct CapacityProbe {
    active: AtomicUsize,
    peak: AtomicUsize,
}

impl TaskExecutor for CapacityProbe {
    fn run(&self, task: &Task) -> TaskOutcome {
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.peak.fetch_max(active, Ordering::SeqCst);
        if task.id == "C" {
            // C binds the peak: if the scheduler ever admitted three
            // tasks, C is the one most likely to overlap them.
            std::thread::sleep(Duration::from_millis(50));
        } else {
            let deadline = Instant::now() + Duration::from_secs(3);
            while self.active.load(Ordering::SeqCst) < 2 && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        self.active.fetch_sub(1, Ordering::SeqCst);
        TaskOutcome::Succeeded {
            cost: task.cost.clone(),
        }
    }
}

/// AE-01/S2 positive behavior (AC2; R-055): independent tasks overlap,
/// and the observed peak never exceeds max_in_flight.
#[test]
fn s2_dispatch_independent_tasks_overlap_within_capacity() {
    let mut dag = TaskDag::new(vec![]);
    for id in ["A", "B", "C"] {
        dag.add(s2_task(id, &[])).expect("add");
    }
    dag.validate().expect("valid");
    let mut sched = Scheduler::new(
        dag,
        BudgetLedger::new(usd(100)).expect("budget"),
        SchedulerConfig {
            max_in_flight: 2,
            resource_capacity: 4,
            memory_capacity_mb: 1024,
            max_queued: usize::MAX,
        },
    )
    .expect("scheduler");
    let probe = CapacityProbe {
        active: AtomicUsize::new(0),
        peak: AtomicUsize::new(0),
    };
    let report = sched.run(&probe).expect("run");
    let peak = probe.peak.load(Ordering::SeqCst);
    assert!(
        peak >= 2,
        "independent tasks overlap: peak concurrent was {peak}"
    );
    assert!(
        peak <= 2,
        "max_in_flight bound holds: peak concurrent was {peak}"
    );
    assert_eq!(
        report.dispatch_order(),
        &["A".to_string(), "B".to_string(), "C".to_string()],
        "selection order stays priority-class then declaration"
    );
    assert_eq!(
        report.succeeded(),
        &["A".to_string(), "B".to_string(), "C".to_string()],
        "every independent task succeeds"
    );
    assert_eq!(sched.budget().reserved(), usd(0), "no hold survives");
}

/// Records which task ids reached the executor.
struct CallRecorder {
    calls: Mutex<Vec<String>>,
}

impl CallRecorder {
    fn new() -> Self {
        CallRecorder {
            calls: Mutex::new(Vec::new()),
        }
    }

    fn seen(&self) -> Vec<String> {
        self.calls.lock().expect("calls lock").clone()
    }
}

impl TaskExecutor for CallRecorder {
    fn run(&self, task: &Task) -> TaskOutcome {
        self.calls.lock().expect("calls lock").push(task.id.clone());
        TaskOutcome::Succeeded {
            cost: task.cost.clone(),
        }
    }
}

/// AE-01/S2 refusal behavior (AC2; R-055): after an earlier task
/// commits, a priced task above the remaining budget is refused before
/// any executor call, and one owner settles the difference.
#[test]
fn s2_dispatch_budget_refusal_never_reaches_the_executor() {
    let mut dag = TaskDag::new(vec![]);
    let mut priced = s2_task("PRICED", &["CHEAP"]);
    priced.cost = usd(100);
    let mut cheap = s2_task("CHEAP", &[]);
    cheap.cost = usd(60);
    dag.add(priced).expect("add priced");
    dag.add(cheap).expect("add cheap");
    dag.validate().expect("valid");
    let mut sched = Scheduler::new(
        dag,
        BudgetLedger::new(usd(100)).expect("budget"),
        SchedulerConfig {
            max_in_flight: 4,
            resource_capacity: 4,
            memory_capacity_mb: 1024,
            max_queued: usize::MAX,
        },
    )
    .expect("scheduler");
    let recorder = CallRecorder::new();
    let report = sched.run(&recorder).expect("run");
    assert_eq!(
        report.dispatch_order(),
        &["CHEAP".to_string()],
        "the refused task never dispatches"
    );
    assert_eq!(
        report.failure_reason("PRICED"),
        Some("BUDGET_UNAVAILABLE"),
        "refusal keeps its reason"
    );
    assert_eq!(
        recorder.seen(),
        vec!["CHEAP".to_string()],
        "refused work never reaches the executor"
    );
    assert_eq!(
        report.succeeded(),
        &["CHEAP".to_string()],
        "only admitted work succeeds"
    );
    assert_eq!(report.failed(), &["PRICED".to_string()], "one refusal");
    assert_eq!(sched.budget().reserved(), usd(0), "holds settle once");
    assert_eq!(
        sched.budget().spent(),
        usd(60),
        "only admitted work commits"
    );
}

/// AE-01/S2 replay behavior (AC2; R-051): two fresh runs of the same
/// DAG record the same dispatch order and the same report lists.
#[test]
fn s2_dispatch_replay_is_stable_across_runs() {
    let build = || {
        let mut dag = TaskDag::new(vec![]);
        dag.add(s2_task("ROOT", &[])).expect("add root");
        dag.add(s2_task("MID", &["ROOT"])).expect("add mid");
        dag.add(s2_task("LEAF", &["MID"])).expect("add leaf");
        dag.validate().expect("valid");
        dag
    };
    let config = SchedulerConfig {
        max_in_flight: 4,
        resource_capacity: 4,
        memory_capacity_mb: 1024,
        max_queued: usize::MAX,
    };
    let mut runs: Vec<(Vec<String>, Vec<String>, Vec<String>)> = Vec::new();
    for _ in 0..2 {
        let mut sched = Scheduler::new(
            build(),
            BudgetLedger::new(usd(100)).expect("budget"),
            config,
        )
        .expect("scheduler");
        let report = sched.run(&CallRecorder::new()).expect("run");
        runs.push((
            report.dispatch_order().to_vec(),
            report.succeeded().to_vec(),
            report.failed().to_vec(),
        ));
    }
    assert_eq!(
        runs[0], runs[1],
        "recorded replay and report ordering are reproducible"
    );
    assert_eq!(
        runs[0].0,
        vec!["ROOT".to_string(), "MID".to_string(), "LEAF".to_string()],
        "chain dispatch order follows the DAG"
    );
}

// --- AE-01/S3 --- cancellation and recovery (AC3) -------------------

fn s3_temp_dir(tag: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("ae01-s3-{tag}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

/// AE-01/S3 first behavior red (AC3; R-055 R-056 R-057): an operation
/// that was dispatched and then carried a durable cancel request must be
/// reconciled, never requeued — requeue would make a new reservation for
/// cancelled work (MASTER_SPEC:375), and dropping it would hide an
/// uncertain effect behind Cancelled.
#[test]
fn s3_recovery_cancel_intent_race_never_requeues_dispatched_work() {
    let dir = s3_temp_dir("cancel-race");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    // Crash-replay fixture: dispatch raced a durable cancel request.
    let op = {
        let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
        let op = rec.plan("race").expect("plan");
        rec.dispatched(&op, 1).expect("dispatch");
        rec.cancel_requested(&op).expect("cancel");
        op
    }; // crash: recorder dropped, ledger durable
    let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
    let view = rec.replay();
    assert_eq!(
        format!("{:?}", view.state_of(&op).unwrap()),
        "Ambiguous",
        "uncertain effect never hides behind Cancelled"
    );
    // Retry permission is wide open — durable cancel intent must win.
    let plan = recover(&view, |_| RetryPosture {
        retryable: true,
        max_attempts: 3,
    });
    assert!(
        !plan.requeue.contains(&op),
        "cancelled in-flight work must never be requeued; requeue={:?} cancelled={:?} unresolved={:?}",
        plan.requeue,
        plan.cancelled,
        plan.unresolved
    );
    assert!(
        plan.unresolved.contains(&op),
        "reconcile the in-flight effect; requeue={:?} cancelled={:?} unresolved={:?}",
        plan.requeue,
        plan.cancelled,
        plan.unresolved
    );
    assert!(
        !plan.cancelled.contains(&op),
        "cancelled bucket is for never-dispatched work only"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// AE-01/S3 (AC3; R-057): cancel intent with no dispatch classifies as
/// cancelled. Apply records nothing and the budget is untouched — there
/// is no effect to reconcile and nothing to requeue.
#[test]
fn s3_recovery_cancel_before_dispatch_never_touches_budget() {
    let dir = s3_temp_dir("cancel-pre");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    let op = {
        let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
        let op = rec.plan("pre").expect("plan");
        rec.cancel_requested(&op).expect("cancel");
        op
    }; // crash before any dispatch
    let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
    let view = rec.replay();
    assert_eq!(
        format!("{:?}", view.state_of(&op).unwrap()),
        "Cancelled",
        "planned + cancel, no dispatch"
    );
    let plan = recover(&view, |_| RetryPosture {
        retryable: true,
        max_attempts: 3,
    });
    assert_eq!(plan.total(), 1, "every op classified exactly once");
    assert!(
        plan.cancelled.contains(&op),
        "cancel before dispatch lands in cancelled"
    );
    assert!(!plan.requeue.contains(&op), "never requeue cancelled work");
    assert!(!plan.unresolved.contains(&op), "no effect ran to reconcile");
    let mut budget = BudgetLedger::new(usd(100)).expect("budget");
    let recorded = apply(&plan, &view, &mut budget).expect("apply");
    assert!(recorded.is_empty(), "nothing to record");
    assert_eq!(budget.reserved(), usd(0), "no hold was ever taken");
    assert_eq!(budget.spent(), usd(0), "no spend");
    assert_eq!(budget.unresolved_count(), 0, "no unresolved entry");
    let _ = std::fs::remove_dir_all(&dir);
}

/// AE-01/S3 (AC3; R-056): duplicate completion receipts collapse into
/// ONE unresolved record. The effect stays unknown (retry posture
/// withholds permission — never a blind retry), and a repeated apply
/// never records a second effect.
#[test]
fn s3_recovery_duplicate_completion_records_one_unresolved_cost() {
    let dir = s3_temp_dir("dup-completion");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    let receipt = Receipt {
        outcome: "AmbiguousEffect".to_string(),
        cost: usd(40),
        wall_ms: 5,
        reason: Some("effect unknown after crash".to_string()),
        attempt: 1,
        executor: "s3-probe".to_string(),
        artifacts: vec![],
    };
    let op = {
        let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
        let op = rec.plan("dup").expect("plan");
        rec.dispatched(&op, 1).expect("dispatch");
        rec.receipt(&op, &receipt).expect("first completion");
        rec.receipt(&op, &receipt).expect("duplicate completion");
        op
    }; // crash
    let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
    let view = rec.replay();
    assert_eq!(format!("{:?}", view.state_of(&op).unwrap()), "Ambiguous");
    let plan = recover(&view, |_| RetryPosture {
        retryable: false,
        max_attempts: 1,
    });
    assert_eq!(plan.total(), 1, "duplicate receipts, still one operation");
    assert!(
        plan.unresolved.contains(&op),
        "unknown effect reconciles, never requeues"
    );
    assert!(!plan.requeue.contains(&op), "no blind retry");
    let mut budget = BudgetLedger::new(usd(100)).expect("budget");
    let first = apply(&plan, &view, &mut budget).expect("apply 1");
    assert_eq!(first, vec![op.clone()], "one unresolved cost recorded once");
    assert_eq!(budget.unresolved_count(), 1, "exactly one entry");
    assert_eq!(budget.unresolved(), usd(40), "amount held exactly once");
    let second = apply(&plan, &view, &mut budget).expect("apply 2");
    assert!(second.is_empty(), "second apply records nothing");
    assert_eq!(budget.unresolved_count(), 1, "still one entry");
    assert_eq!(budget.unresolved(), usd(40), "no duplicate effect");
    let _ = std::fs::remove_dir_all(&dir);
}

/// AE-01/S3 (AC3; R-051 R-057): crash replay classifies every recorded
/// operation exactly once, and recomputing the plan from the same
/// recorded view yields identical buckets (deterministic replay).
#[test]
fn s3_recovery_crash_replay_classifies_each_operation_once() {
    let dir = s3_temp_dir("mixed");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    let receipt = Receipt {
        outcome: "Succeeded".to_string(),
        cost: usd(10),
        wall_ms: 3,
        reason: None,
        attempt: 1,
        executor: "s3-probe".to_string(),
        artifacts: vec![],
    };
    let (op_p, op_s, op_d, op_c) = {
        let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
        let op_p = rec.plan("p").expect("planned only");
        let op_s = rec.plan("s").expect("succeeded");
        rec.dispatched(&op_s, 1).expect("dispatch");
        rec.receipt(&op_s, &receipt).expect("receipt");
        let op_d = rec.plan("d").expect("dispatched only");
        rec.dispatched(&op_d, 1).expect("dispatch");
        let op_c = rec.plan("c").expect("cancelled before dispatch");
        rec.cancel_requested(&op_c).expect("cancel");
        (op_p, op_s, op_d, op_c)
    }; // crash
    let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
    let view = rec.replay();
    let plan = recover(&view, |_| RetryPosture {
        retryable: false,
        max_attempts: 1,
    });
    assert_eq!(plan.total(), 4, "every recorded op appears exactly once");
    assert_eq!(plan.requeue, vec![op_p], "planned requeues");
    assert_eq!(plan.terminal, vec![op_s], "receipt-backed terminal");
    assert_eq!(
        plan.unresolved,
        vec![op_d],
        "dispatched without receipt reconciles"
    );
    assert_eq!(
        plan.cancelled,
        vec![op_c],
        "cancel without dispatch is cancelled"
    );
    assert!(plan.corrupt.is_empty(), "clean history is not corrupt");
    // Same recorded view, same decision — replay determinism (R-051).
    let again = recover(&view, |_| RetryPosture {
        retryable: false,
        max_attempts: 1,
    });
    assert_eq!(
        format!("{:?}", plan),
        format!("{:?}", again),
        "recomputed plan is identical"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// AE-01/S3 (AC3; R-055 R-056): a stale pre-crash hold is released and
/// the known receipt cost is recorded as unresolved exactly once — a
/// re-apply never duplicates the effect.
#[test]
fn s3_recovery_unresolved_cost_applies_once_over_stale_hold() {
    let dir = s3_temp_dir("stale-hold");
    let (ledger, store) = (dir.join("ops.jsonl"), dir.join("store"));
    let receipt = Receipt {
        outcome: "AmbiguousEffect".to_string(),
        cost: usd(40),
        wall_ms: 7,
        reason: Some("effect unknown".to_string()),
        attempt: 1,
        executor: "s3-probe".to_string(),
        artifacts: vec![],
    };
    let op = {
        let mut rec = OperationRecorder::open(&ledger, &store).expect("open");
        let op = rec.plan("hold").expect("plan");
        rec.dispatched(&op, 1).expect("dispatch");
        rec.receipt(&op, &receipt).expect("receipt");
        op
    }; // crash with the hold still live
    let rec = OperationRecorder::open(&ledger, &store).expect("reopen");
    let view = rec.replay();
    let plan = recover(&view, |_| RetryPosture {
        retryable: false,
        max_attempts: 1,
    });
    assert!(plan.unresolved.contains(&op), "ambiguous needs reconcile");
    let mut budget = BudgetLedger::new(usd(100)).expect("budget");
    budget
        .reserve(&op, usd(60))
        .expect("stale hold from the crashed dispatch");
    let recorded = apply(&plan, &view, &mut budget).expect("apply");
    assert_eq!(recorded, vec![op.clone()], "known cost recorded once");
    assert_eq!(budget.reserved(), usd(0), "stale hold released");
    assert_eq!(budget.unresolved(), usd(40), "receipt amount reconciled");
    assert_eq!(budget.unresolved_count(), 1, "exactly one entry");
    let again = apply(&plan, &view, &mut budget).expect("re-apply");
    assert!(again.is_empty(), "re-apply records nothing");
    assert_eq!(budget.unresolved_count(), 1, "still exactly one entry");
    assert_eq!(budget.unresolved(), usd(40), "no duplicate effect");
    assert_eq!(budget.spent(), usd(0), "nothing marked spent");
    let _ = std::fs::remove_dir_all(&dir);
}

/// AE-01/S3 (AC3; R-055 R-057): the cancellation race settles each
/// hold exactly once at the scheduler seam. PRE is cancelled before
/// dispatch: it never reaches the executor and never reserves. LONG is
/// cancelled while in flight (flag flipped after the executor starts):
/// cooperative cancellation completes it and releases its hold once.
#[test]
fn s3_recovery_cancel_race_settles_each_hold_exactly_once() {
    struct S3CancelRaceExecutor {
        started_long: Arc<AtomicBool>,
        runs: Mutex<Vec<String>>,
        signalled: Mutex<Vec<String>>,
    }

    impl S3CancelRaceExecutor {
        fn new() -> Self {
            S3CancelRaceExecutor {
                started_long: Arc::new(AtomicBool::new(false)),
                runs: Mutex::new(Vec::new()),
                signalled: Mutex::new(Vec::new()),
            }
        }
    }

    impl TaskExecutor for S3CancelRaceExecutor {
        fn run(&self, task: &Task) -> TaskOutcome {
            self.runs.lock().unwrap().push(task.id.clone());
            TaskOutcome::Succeeded {
                cost: task.cost.clone(),
            }
        }

        fn run_cancellable(&self, task: &Task, cancel: &AtomicBool) -> TaskOutcome {
            self.runs.lock().unwrap().push(task.id.clone());
            if task.id == "LONG" {
                self.started_long.store(true, Ordering::SeqCst);
                let deadline = Instant::now() + Duration::from_secs(3);
                while Instant::now() < deadline {
                    if cancel.load(Ordering::SeqCst) {
                        return TaskOutcome::Cancelled;
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
                return TaskOutcome::Failed {
                    reason: "cancel flag never arrived".to_string(),
                };
            }
            TaskOutcome::Succeeded {
                cost: task.cost.clone(),
            }
        }

        fn signal_cancel(&self, task_id: &str) {
            self.signalled.lock().unwrap().push(task_id.to_string());
        }
    }

    let mut dag = TaskDag::new(vec![]);
    let mut pre = s2_task("PRE", &[]);
    pre.cost = usd(30);
    let mut long = s2_task("LONG", &[]);
    long.cost = usd(50);
    dag.add(pre).expect("add PRE");
    dag.add(long).expect("add LONG");
    dag.validate().expect("valid");
    let mut sched = Scheduler::new(
        dag,
        BudgetLedger::new(usd(200)).expect("budget"),
        SchedulerConfig {
            max_in_flight: 4,
            resource_capacity: 4,
            memory_capacity_mb: 1024,
            max_queued: usize::MAX,
        },
    )
    .expect("scheduler");
    let pre_flag = sched.cancel_handle("PRE").expect("PRE handle");
    let long_flag = sched.cancel_handle("LONG").expect("LONG handle");
    // Cancel PRE before dispatch: it must never reserve.
    pre_flag.store(true, Ordering::SeqCst);
    let executor = S3CancelRaceExecutor::new();
    // Flip LONG's flag only after its executor started: a genuine race
    // between dispatch and cancellation.
    let started = executor.started_long.clone();
    let helper = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        while !started.load(Ordering::SeqCst) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        long_flag.store(true, Ordering::SeqCst);
    });
    let report = sched.run(&executor).expect("run");
    helper.join().expect("helper");
    let runs = executor.runs.lock().unwrap().clone();
    assert_eq!(
        runs,
        vec!["LONG".to_string()],
        "PRE never reaches the executor; LONG dispatched once"
    );
    assert!(
        executor
            .signalled
            .lock()
            .unwrap()
            .iter()
            .any(|s| s == "PRE"),
        "PRE got the cancel signal"
    );
    assert!(
        report.cancelled().contains(&"PRE".to_string()),
        "PRE settled as cancelled"
    );
    assert!(
        report.cancelled().contains(&"LONG".to_string()),
        "in-flight cancel settles via its own completion"
    );
    assert_eq!(sched.budget().reserved(), usd(0), "every hold settled");
    assert_eq!(
        sched.budget().spent(),
        usd(0),
        "cancelled work never commits"
    );
    assert_eq!(
        sched.budget().unresolved_count(),
        0,
        "no unresolved side effects"
    );
}
