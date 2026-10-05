//! AE-02 S1 — resource envelope admission (R-067/AT-067 S1).
//!
//! Tasks carry a two-dimensional envelope (CPU units + memory MiB).
//! Admission reserves both dimensions per wave before dispatch; claims
//! above node capacity are refused at construction.

use std::sync::Mutex;

use hephaestus::budget::BudgetLedger;
use hephaestus::contracts::generated::Money;
use hephaestus::scheduler::{
    PriorityClass, RetryPolicy, Scheduler, SchedulerConfig, SchedulerError, Task, TaskDag,
    TaskExecutor, TaskOutcome,
};

fn usd(minor: i64) -> Money {
    Money {
        currency: "USD".to_string(),
        minor_units: minor,
    }
}

fn task(id: &str, units: u32, memory_mb: u64) -> Task {
    Task {
        id: id.to_string(),
        priority: PriorityClass::Normal,
        depends_on: vec![],
        inputs: vec![],
        outputs: vec![format!("out-{id}")],
        retry: RetryPolicy { max_attempts: 1 },
        timeout_ms: 1_000,
        cost: usd(0),
        trivial: true, // batchable: multi-task waves pass through run_batch
        retryable: true,
        resource_units: units,
        memory_mb,
        exclusive: false,
    }
}

struct BatchRecorder {
    sizes: Mutex<Vec<usize>>,
}

impl TaskExecutor for BatchRecorder {
    fn run(&self, _task: &Task) -> TaskOutcome {
        TaskOutcome::Succeeded { cost: usd(0) }
    }

    fn run_batch(&self, tasks: &[Task]) -> Vec<TaskOutcome> {
        self.sizes.lock().unwrap().push(tasks.len());
        tasks.iter().map(|t| self.run(t)).collect()
    }

    fn signal_cancel(&self, _task_id: &str) {}
}

fn scheduler(dag: TaskDag) -> Scheduler {
    Scheduler::new(
        dag,
        BudgetLedger::new(usd(1000)).expect("budget"),
        SchedulerConfig {
            max_in_flight: 4,
            resource_capacity: 8,
            memory_capacity_mb: 1024,
        },
    )
    .expect("scheduler")
}

#[test]
fn s1_envelope_memory_claims_split_waves_before_dispatch() {
    // A(900) + B(900) exceed 1024 MiB together but each fits; C(100)
    // fits beside either. Admission reserves memory per wave before
    // dispatch: wave 1 carries A+C (batch of two — B cannot join),
    // wave 2 carries B alone. CPU alone would admit all three at once.
    let mut dag = TaskDag::new(vec![]);
    dag.add(task("A", 1, 900)).unwrap();
    dag.add(task("B", 1, 900)).unwrap();
    dag.add(task("C", 1, 100)).unwrap();
    dag.validate().expect("valid");
    let mut sched = scheduler(dag);
    let recorder = BatchRecorder {
        sizes: Mutex::new(Vec::new()),
    };
    let report = sched.run(&recorder).expect("run");
    assert_eq!(report.succeeded().len(), 3, "all tasks succeed");
    // Only the two-task wave passes through run_batch; B's solo wave
    // takes the single-task run() path. Ignored memory would yield [3].
    assert_eq!(*recorder.sizes.lock().unwrap(), vec![2]);
}

#[test]
fn s1_envelope_zero_memory_keeps_cpu_only_admission() {
    // Regression: tasks without a memory claim admit exactly as before
    // -- two CPU-fitting tasks share one wave (one run_batch of two).
    let mut dag = TaskDag::new(vec![]);
    dag.add(task("A", 1, 0)).unwrap();
    dag.add(task("B", 1, 0)).unwrap();
    dag.validate().expect("valid");
    let mut sched = scheduler(dag);
    let recorder = BatchRecorder {
        sizes: Mutex::new(Vec::new()),
    };
    let report = sched.run(&recorder).expect("run");
    assert_eq!(report.succeeded().len(), 2, "both tasks succeed");
    assert_eq!(*recorder.sizes.lock().unwrap(), vec![2]);
}

#[test]
fn s1_envelope_oversize_claims_refused_at_admission() {
    // Memory above node capacity is refused exactly like CPU above
    // capacity — neither claim can ever dispatch.
    for (id, units, memory_mb) in [("TOO-MUCH-MEM", 1, 2048), ("TOO-MANY-UNITS", 99, 0)] {
        let mut dag = TaskDag::new(vec![]);
        dag.add(task(id, units, memory_mb)).unwrap();
        let err = Scheduler::new(
            dag,
            BudgetLedger::new(usd(1000)).expect("budget"),
            SchedulerConfig {
                max_in_flight: 4,
                resource_capacity: 8,
                memory_capacity_mb: 1024,
            },
        )
        .expect_err("oversize claim refused");
        assert!(
            matches!(err, SchedulerError::ResourceExceedsCapacity { .. }),
            "{id}: oversize refused by name, got: {err:?}"
        );
    }
}
