//! AE-01 S1 — concurrent executor contracts (R-051/R-055/R-056/R-057).
//!
//! Bounded submission, completion identities, cancellation, and
//! ambiguous-effect outcomes. Full overlap lands in S2; recovery in S3.

use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use hephaestus::budget::BudgetLedger;
use hephaestus::contracts::generated::Money;
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
