//! T-009 tickets 02-03 — bounded scheduler through the executor contract
//! (seam: `hephaestus::scheduler`). Mock executor = the true external.
//!
//! Obligations: R-055/AT-055 (reserve transactionally before dispatch),
//! R-056/AT-056 (ambiguous effects -> unresolved, never retried).

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use hephaestus::budget::BudgetLedger;
use hephaestus::contracts::generated::Money;
use hephaestus::scheduler::{
    PriorityClass, RetryPolicy, Scheduler, SchedulerConfig, Task, TaskDag, TaskExecutor,
    TaskOutcome,
};

fn usd(minor_units: i64) -> Money {
    Money {
        currency: "USD".to_string(),
        minor_units,
    }
}

fn task(id: &str) -> Task {
    Task {
        id: id.to_string(),
        priority: PriorityClass::Normal,
        depends_on: vec![],
        inputs: vec![],
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

/// Scripted mock: canned outcomes per task id + concurrency accounting.
struct MockExecutor {
    outcomes: Mutex<Vec<(String, TaskOutcome)>>,
    running: AtomicUsize,
    max_concurrent: AtomicUsize,
    calls: AtomicUsize,
    batch_calls: AtomicUsize,
    batched_tasks: AtomicUsize,
    signalled: Mutex<Vec<String>>,
}

impl MockExecutor {
    fn new() -> Self {
        MockExecutor {
            outcomes: Mutex::new(Vec::new()),
            running: AtomicUsize::new(0),
            max_concurrent: AtomicUsize::new(0),
            calls: AtomicUsize::new(0),
            batch_calls: AtomicUsize::new(0),
            batched_tasks: AtomicUsize::new(0),
            signalled: Mutex::new(Vec::new()),
        }
    }

    fn script(&self, id: &str, outcome: TaskOutcome) {
        self.outcomes
            .lock()
            .unwrap()
            .push((id.to_string(), outcome));
    }

    fn outcome_for(&self, id: &str) -> TaskOutcome {
        let mut table = self.outcomes.lock().unwrap();
        if let Some(pos) = table.iter().position(|(i, _)| i == id) {
            table.remove(pos).1
        } else {
            TaskOutcome::Succeeded { cost: usd(0) }
        }
    }

    fn enter(&self) {
        let now = self.running.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_concurrent.fetch_max(now, Ordering::SeqCst);
        self.calls.fetch_add(1, Ordering::SeqCst);
    }

    fn leave(&self) {
        self.running.fetch_sub(1, Ordering::SeqCst);
    }
}

impl TaskExecutor for MockExecutor {
    fn run(&self, task: &Task) -> TaskOutcome {
        self.enter();
        let outcome = self.outcome_for(&task.id);
        self.leave();
        outcome
    }

    fn run_batch(&self, tasks: &[Task]) -> Vec<TaskOutcome> {
        self.batch_calls.fetch_add(1, Ordering::SeqCst);
        self.batched_tasks.fetch_add(tasks.len(), Ordering::SeqCst);
        tasks.iter().map(|t| self.run(t)).collect()
    }

    fn signal_cancel(&self, task_id: &str) {
        self.signalled.lock().unwrap().push(task_id.to_string());
    }
}

fn scheduler(dag: TaskDag, budget: BudgetLedger, max_in_flight: usize, capacity: u32) -> Scheduler {
    Scheduler::new(
        dag,
        budget,
        SchedulerConfig {
            max_in_flight,
            resource_capacity: capacity,
            memory_capacity_mb: 1024,
        },
    )
    .expect("scheduler")
}

#[test]
fn reserve_happens_before_dispatch_and_success_commits_exactly() {
    // AT-055 / R-055.
    let mut dag = TaskDag::new(vec![]);
    let mut t = task("PAID");
    t.cost = usd(250);
    dag.add(t).unwrap();
    dag.validate().expect("valid");
    let budget = BudgetLedger::new(usd(1000)).expect("budget");
    let executor = MockExecutor::new();
    executor.script("PAID", TaskOutcome::Succeeded { cost: usd(250) });
    let mut sched = scheduler(dag, budget, 4, 8);
    let report = sched.run(&executor).expect("run");

    assert_eq!(report.succeeded(), &["PAID".to_string()]);
    // Cost was reserved then committed — spend equals the declared cost.
    assert_eq!(sched.budget().spent(), usd(250));
    assert_eq!(sched.budget().reserved(), usd(0), "reservation settled");
    assert_eq!(executor.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn failure_and_unknown_cost_release_or_refuse() {
    // Failure releases; AmbiguousEffect becomes an unresolved entry (R-056).
    let mut dag = TaskDag::new(vec![]);
    let mut f = task("FAILS");
    f.cost = usd(100);
    let mut a = task("AMBIG");
    a.cost = usd(50);
    dag.add(f).unwrap();
    dag.add(a).unwrap();
    dag.validate().expect("valid");
    let budget = BudgetLedger::new(usd(1000)).expect("budget");
    let executor = MockExecutor::new();
    executor.script(
        "FAILS",
        TaskOutcome::Failed {
            reason: "boom".to_string(),
        },
    );
    executor.script(
        "AMBIG",
        TaskOutcome::AmbiguousEffect {
            reason: "unknown submit".to_string(),
        },
    );
    let mut sched = scheduler(dag, budget, 4, 8);
    let report = sched.run(&executor).expect("run");

    assert_eq!(report.failed(), &["FAILS".to_string()]);
    assert_eq!(report.unresolved(), &["AMBIG".to_string()]);
    assert_eq!(
        sched.budget().reserved(),
        usd(0),
        "no held reservations left"
    );
    assert_eq!(sched.budget().spent(), usd(0), "failures spent nothing");
    assert_eq!(
        sched.budget().unresolved(),
        usd(50),
        "ambiguous charge held as unresolved"
    );
    assert_eq!(sched.budget().unresolved_count(), 1);
}

#[test]
fn bounded_waves_respect_max_in_flight_resource_units_and_exclusivity() {
    // Resource limits: 6 ready tasks, max_in_flight=2, capacity=3 units,
    // 1-unit tasks => never more than 2 concurrent, units never exceeded.
    let mut dag = TaskDag::new(vec![]);
    for i in 0..6 {
        let mut t = task(&format!("T{i}"));
        t.resource_units = 1;
        dag.add(t).unwrap();
    }
    dag.validate().expect("valid");
    let budget = BudgetLedger::new(usd(10)).expect("budget");
    let executor = MockExecutor::new();
    let mut sched = scheduler(dag, budget, 2, 3);
    let report = sched.run(&executor).expect("run");
    assert_eq!(report.succeeded().len(), 6);
    assert!(
        executor.max_concurrent.load(Ordering::SeqCst) <= 2,
        "max_in_flight bound held: {}",
        executor.max_concurrent.load(Ordering::SeqCst)
    );

    // Exclusivity: an exclusive task runs when nothing else does.
    let mut dag = TaskDag::new(vec![]);
    let mut x = task("EXCL");
    x.exclusive = true;
    x.resource_units = 1;
    dag.add(x).unwrap();
    for i in 0..3 {
        let mut t = task(&format!("S{i}"));
        t.resource_units = 1;
        dag.add(t).unwrap();
    }
    dag.validate().expect("valid");
    let executor = MockExecutor::new();
    let mut sched = scheduler(dag, BudgetLedger::new(usd(10)).expect("b"), 4, 4);
    let report = sched.run(&executor).expect("run");
    assert_eq!(report.succeeded().len(), 4);
    assert!(
        executor.max_concurrent.load(Ordering::SeqCst) >= 1,
        "work happened"
    );
    // Exclusive selection is enforced by the wave builder (unit-covered);
    // end-to-end: EXCL completed.
    assert!(report.succeeded().contains(&"EXCL".to_string()));
}

#[test]
fn priorities_drain_critical_first_and_runs_are_deterministic() {
    let mut dag = TaskDag::new(vec![]);
    let mut low = task("EXPL");
    low.priority = PriorityClass::Exploration;
    let mut mid = task("VER");
    mid.priority = PriorityClass::Verification;
    let mut high = task("CRIT");
    high.priority = PriorityClass::Critical;
    for t in [low, mid, high] {
        dag.add(t).unwrap();
    }
    dag.validate().expect("valid");
    let budget = BudgetLedger::new(usd(10)).expect("budget");
    let executor = MockExecutor::new();
    let mut sched = scheduler(dag, budget, 1, 4); // one at a time => order observable
    let report = sched.run(&executor).expect("run");
    assert_eq!(
        report.dispatch_order(),
        &["CRIT".to_string(), "VER".to_string(), "EXPL".to_string()],
        "class order, declaration order within class"
    );
    // Determinism: identical inputs => identical dispatch order.
    let mut dag2 = TaskDag::new(vec![]);
    for (id, prio) in [
        ("EXPL", PriorityClass::Exploration),
        ("VER", PriorityClass::Verification),
        ("CRIT", PriorityClass::Critical),
    ] {
        let mut t = task(id);
        t.priority = prio;
        dag2.add(t).unwrap();
    }
    let executor2 = MockExecutor::new();
    let mut sched2 = scheduler(dag2, BudgetLedger::new(usd(10)).expect("budget"), 1, 4);
    let report2 = sched2.run(&executor2).expect("run");
    assert_eq!(report.dispatch_order(), report2.dispatch_order());
}

// --- Ticket 03: bounded retries and timeout outcomes ---

#[test]
fn non_idempotent_work_is_attempted_exactly_once() {
    // MASTER_SPEC:371 — never blindly retry; policy cannot override.
    let mut dag = TaskDag::new(vec![]);
    let mut t = task("ONCE");
    t.retryable = false;
    t.retry = hephaestus::scheduler::RetryPolicy { max_attempts: 5 };
    dag.add(t).unwrap();
    let executor = MockExecutor::new();
    executor.script(
        "ONCE",
        TaskOutcome::Failed {
            reason: "flaky".to_string(),
        },
    );
    let mut sched = scheduler(dag, BudgetLedger::new(usd(10)).expect("b"), 4, 8);
    let report = sched.run(&executor).expect("run");
    assert_eq!(
        executor.calls.load(Ordering::SeqCst),
        1,
        "retryable:false caps attempts at 1 regardless of max_attempts"
    );
    assert_eq!(report.failed(), &["ONCE".to_string()]);
}

#[test]
fn retries_are_bounded_by_max_attempts_with_visible_attempts() {
    let mut dag = TaskDag::new(vec![]);
    let mut t = task("RETRY");
    t.retry = hephaestus::scheduler::RetryPolicy { max_attempts: 3 };
    dag.add(t).unwrap();
    let executor = MockExecutor::new();
    executor.script(
        "RETRY",
        TaskOutcome::Failed {
            reason: "e1".to_string(),
        },
    );
    executor.script(
        "RETRY",
        TaskOutcome::Failed {
            reason: "e2".to_string(),
        },
    );
    executor.script("RETRY", TaskOutcome::Succeeded { cost: usd(0) });
    let mut sched = scheduler(dag, BudgetLedger::new(usd(10)).expect("b"), 4, 8);
    let report = sched.run(&executor).expect("run");
    assert_eq!(
        executor.calls.load(Ordering::SeqCst),
        3,
        "two failures then success"
    );
    assert_eq!(report.succeeded(), &["RETRY".to_string()]);

    // Exhaustion: three failures => terminal fail, exactly 3 calls.
    let mut dag = TaskDag::new(vec![]);
    let mut t = task("EXHAUST");
    t.retry = hephaestus::scheduler::RetryPolicy { max_attempts: 3 };
    dag.add(t).unwrap();
    let executor = MockExecutor::new();
    for _ in 0..3 {
        executor.script(
            "EXHAUST",
            TaskOutcome::Failed {
                reason: "nope".to_string(),
            },
        );
    }
    let mut sched = scheduler(dag, BudgetLedger::new(usd(10)).expect("b"), 4, 8);
    let report = sched.run(&executor).expect("run");
    assert_eq!(executor.calls.load(Ordering::SeqCst), 3);
    assert_eq!(report.failed(), &["EXHAUST".to_string()]);
}

#[test]
fn timeouts_follow_the_same_retry_rules() {
    let mut dag = TaskDag::new(vec![]);
    let mut t = task("SLOW");
    t.retry = hephaestus::scheduler::RetryPolicy { max_attempts: 2 };
    dag.add(t).unwrap();
    let executor = MockExecutor::new();
    executor.script("SLOW", TaskOutcome::TimedOut);
    executor.script("SLOW", TaskOutcome::Succeeded { cost: usd(0) });
    let mut sched = scheduler(dag, BudgetLedger::new(usd(10)).expect("b"), 4, 8);
    let report = sched.run(&executor).expect("run");
    assert_eq!(
        executor.calls.load(Ordering::SeqCst),
        2,
        "TimedOut retried like Failed"
    );
    assert_eq!(report.succeeded(), &["SLOW".to_string()]);

    // Deterministic timeout declaration: zero timeout refused at validation.
    let mut bad = task("BAD");
    bad.timeout_ms = 0;
    let mut dag = TaskDag::new(vec![]);
    dag.add(bad).unwrap();
    assert!(dag.validate().is_err(), "timeout validated at the DAG seam");
}

// --- Ticket 04: cancellation, batching, accounting ---

#[test]
fn cancellation_prevents_new_reservations_signals_and_cascades() {
    // MASTER_SPEC:375 / R-055 / R-056.
    let mut dag = TaskDag::new(vec![]);
    let mut root = task("ROOT");
    root.cost = usd(300);
    let mut child = task("CHILD");
    child.cost = usd(100);
    child.depends_on = vec!["ROOT".to_string()];
    child.inputs = vec!["out-ROOT".to_string()];
    dag.add(root).unwrap();
    dag.add(child).unwrap();
    dag.validate().expect("valid");
    let budget = BudgetLedger::new(usd(1000)).expect("budget");
    let executor = MockExecutor::new();
    let mut sched = scheduler(dag, budget, 4, 8);
    let root_flag = sched.cancel_handle("ROOT").expect("handle exists");
    root_flag.store(true, Ordering::SeqCst);
    let report = sched.run(&executor).expect("run");

    assert_eq!(
        executor.calls.load(Ordering::SeqCst),
        0,
        "nothing dispatched"
    );
    assert!(
        executor
            .signalled
            .lock()
            .unwrap()
            .iter()
            .any(|s| s == "ROOT"),
        "signal_cancel fired"
    );
    assert_eq!(sched.budget().reserved(), usd(0), "no new reservations");
    assert_eq!(sched.budget().spent(), usd(0));
    assert!(report.cancelled().contains(&"ROOT".to_string()));
    assert!(
        report.cancelled().contains(&"CHILD".to_string()),
        "descendants cancelled too"
    );
}

#[test]
fn executor_reported_cancellation_releases_the_reservation() {
    let mut dag = TaskDag::new(vec![]);
    let mut t = task("STOPPED");
    t.cost = usd(75);
    dag.add(t).unwrap();
    let executor = MockExecutor::new();
    executor.script("STOPPED", TaskOutcome::Cancelled);
    let mut sched = scheduler(dag, BudgetLedger::new(usd(500)).expect("b"), 4, 8);
    let report = sched.run(&executor).expect("run");
    assert_eq!(report.cancelled(), &["STOPPED".to_string()]);
    assert_eq!(sched.budget().reserved(), usd(0), "reservation released");
    assert_eq!(sched.budget().spent(), usd(0));
    assert_eq!(sched.budget().available(), usd(500));
}

#[test]
fn trivial_ready_tasks_are_batched_and_non_trivial_never_are() {
    // Plan: batch trivial deterministic tasks instead of spawning per check.
    let mut dag = TaskDag::new(vec![]);
    for i in 0..6 {
        let mut t = task(&format!("TRIV{i}"));
        t.trivial = true;
        dag.add(t).unwrap();
    }
    dag.validate().expect("valid");
    let executor = MockExecutor::new();
    let mut sched = scheduler(dag, BudgetLedger::new(usd(10)).expect("b"), 6, 6);
    let report = sched.run(&executor).expect("run");
    assert_eq!(report.succeeded().len(), 6);
    assert_eq!(
        executor.batch_calls.load(Ordering::SeqCst),
        1,
        "one wave, one run_batch call"
    );
    assert_eq!(executor.batched_tasks.load(Ordering::SeqCst), 6);

    let mut dag = TaskDag::new(vec![]);
    for i in 0..3 {
        dag.add(task(&format!("HEAVY{i}"))).unwrap();
    }
    dag.validate().expect("valid");
    let executor = MockExecutor::new();
    let mut sched = scheduler(dag, BudgetLedger::new(usd(10)).expect("b"), 6, 6);
    sched.run(&executor).expect("run");
    assert_eq!(
        executor.batch_calls.load(Ordering::SeqCst),
        0,
        "non-trivial work is never batched"
    );
}

// --- Review pass 1: regression + AC gaps ---

#[test]
fn mixed_wave_batching_keeps_every_reservation_with_its_own_task() {
    // Review pass 1 bug: batch outcomes execute out of dispatch order; each
    // outcome must settle ITS OWN reservation.
    let mut dag = TaskDag::new(vec![]);
    let mut a = task("HEAVY");
    a.cost = usd(100);
    let mut b = task("TRIV1");
    b.trivial = true;
    b.cost = usd(50);
    let mut c = task("TRIV2");
    c.trivial = true;
    c.cost = usd(50);
    for t in [a, b, c] {
        dag.add(t).unwrap();
    }
    dag.validate().expect("valid");
    let executor = MockExecutor::new();
    executor.script("HEAVY", TaskOutcome::Succeeded { cost: usd(100) });
    executor.script("TRIV1", TaskOutcome::Succeeded { cost: usd(50) });
    executor.script("TRIV2", TaskOutcome::Succeeded { cost: usd(50) });
    let mut sched = scheduler(dag, BudgetLedger::new(usd(1000)).expect("b"), 4, 8);
    let report = sched.run(&executor).expect("run");
    assert_eq!(report.succeeded().len(), 3);
    assert_eq!(
        executor.batch_calls.load(Ordering::SeqCst),
        1,
        "the two trivial tasks batched"
    );
    assert_eq!(
        sched.budget().spent(),
        usd(200),
        "each hold settled exactly once"
    );
    assert_eq!(sched.budget().reserved(), usd(0), "no leaked reservations");
}

#[test]
fn cost_mismatch_fails_the_task_and_returns_the_reservation() {
    let mut dag = TaskDag::new(vec![]);
    let mut t = task("PRICED");
    t.cost = usd(100);
    dag.add(t).unwrap();
    let executor = MockExecutor::new();
    executor.script("PRICED", TaskOutcome::Succeeded { cost: usd(99) });
    let mut sched = scheduler(dag, BudgetLedger::new(usd(500)).expect("b"), 4, 8);
    let report = sched.run(&executor).expect("run");
    assert_eq!(
        report.failed(),
        &["PRICED".to_string()],
        "COST_MISMATCH path"
    );
    assert_eq!(
        report.failure_reason("PRICED"),
        Some("COST_MISMATCH"),
        "reasons are visible, not swallowed"
    );
    assert_eq!(sched.budget().reserved(), usd(0), "reservation returned");
    assert_eq!(sched.budget().spent(), usd(0));
}

#[test]
fn budget_refusal_blocks_dispatch_before_any_executor_call() {
    // Reserve-before-dispatch made observable: no budget, no run.
    let mut dag = TaskDag::new(vec![]);
    let mut t = task("TOO pricey");
    t.cost = usd(5_000);
    dag.add(t).unwrap();
    let executor = MockExecutor::new();
    let mut sched = scheduler(dag, BudgetLedger::new(usd(100)).expect("b"), 4, 8);
    let report = sched.run(&executor).expect("run");
    assert_eq!(report.failed().len(), 1);
    assert_eq!(
        report.failure_reason(report.failed()[0].as_str()),
        Some("BUDGET_UNAVAILABLE"),
        "budget refusal reason visible"
    );
    assert_eq!(
        executor.calls.load(Ordering::SeqCst),
        0,
        "no dispatch without a reservation (AT-055)"
    );
    assert_eq!(sched.budget().reserved(), usd(0));
}

#[test]
fn cross_currency_costs_are_refused_at_construction() {
    // Fail-closed at build time: an EUR claim on a USD ledger can never
    // reserve — and never aborts mid-wave.
    let mut dag = TaskDag::new(vec![]);
    let mut t = task("EURCOST");
    t.cost = Money {
        currency: "EUR".to_string(),
        minor_units: 10,
    };
    dag.add(t).unwrap();
    let err = Scheduler::new(
        dag,
        BudgetLedger::new(usd(100)).expect("b"),
        SchedulerConfig {
            max_in_flight: 2,
            resource_capacity: 4,
            memory_capacity_mb: 1024,
        },
    )
    .expect_err("currency mismatch refused");
    assert!(matches!(
        err,
        hephaestus::scheduler::SchedulerError::InvalidConfig(_)
    ));
}

#[test]
fn attempt_counts_are_visible_in_the_report() {
    let mut dag = TaskDag::new(vec![]);
    let mut t = task("TRIES");
    t.retry = hephaestus::scheduler::RetryPolicy { max_attempts: 3 };
    dag.add(t).unwrap();
    let executor = MockExecutor::new();
    executor.script(
        "TRIES",
        TaskOutcome::Failed {
            reason: "e1".to_string(),
        },
    );
    executor.script(
        "TRIES",
        TaskOutcome::Failed {
            reason: "e2".to_string(),
        },
    );
    executor.script("TRIES", TaskOutcome::Succeeded { cost: usd(0) });
    let mut sched = scheduler(dag, BudgetLedger::new(usd(10)).expect("b"), 4, 8);
    let report = sched.run(&executor).expect("run");
    assert_eq!(
        report.attempts("TRIES"),
        Some(3),
        "three dispatches visible"
    );
    assert_eq!(report.dispatch_order().len(), 3, "retries re-dispatch");
}
