//! AE-02 S2+S3 — admission queue bound, budgeted waves, measured memory
//! limits (R-067/AT-067 S2, S3).
//!
//! Admission reserves per-wave dimensions before dispatch (S1); refuses
//! queues over the bound at construction (S2); measures worker RSS peaks
//! against the hard memory limit (S3).

use std::sync::Mutex;

use hephaestus::budget::BudgetLedger;
use hephaestus::contracts::generated::Money;
use hephaestus::sandbox::{IsolationSpec, NetworkPolicy, RunSpec, SandboxProvider};
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
    scheduler_with_queue(dag, usize::MAX)
}

fn scheduler_with_queue(dag: TaskDag, max_queued: usize) -> Scheduler {
    Scheduler::new(
        dag,
        BudgetLedger::new(usd(1000)).expect("budget"),
        SchedulerConfig {
            max_in_flight: 4,
            resource_capacity: 8,
            memory_capacity_mb: 1024,
            max_queued,
        },
    )
    .expect("scheduler")
}

/// A costed task: nonzero cost reserves budget at dispatch and settles
/// (commit or release) at its terminal outcome.
fn costed_task(id: &str, minor: i64) -> Task {
    // Non-retryable: failure and cancel outcomes below are terminal on
    // the first attempt, so each reservation settles exactly once.
    let mut t = task(id, 1, 0);
    t.cost = usd(minor);
    t.retryable = false;
    t
}

/// An executor that scripts one outcome per task id.
struct ScriptedExecutor {
    outcomes: std::collections::HashMap<String, TaskOutcome>,
}

impl TaskExecutor for ScriptedExecutor {
    fn run(&self, task: &Task) -> TaskOutcome {
        self.outcomes
            .get(&task.id)
            .cloned()
            .unwrap_or(TaskOutcome::Succeeded {
                cost: task.cost.clone(),
            })
    }

    fn run_batch(&self, tasks: &[Task]) -> Vec<TaskOutcome> {
        tasks.iter().map(|t| self.run(t)).collect()
    }

    fn signal_cancel(&self, _task_id: &str) {}
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
fn s2_admission_queue_over_bound_refused_before_dispatch() {
    // S2 positive/refusal: three admitted tasks against a bound of two
    // are refused at construction — no wave runs, no budget moves.
    let mut dag = TaskDag::new(vec![]);
    dag.add(task("A", 1, 0)).unwrap();
    dag.add(task("B", 1, 0)).unwrap();
    dag.add(task("C", 1, 0)).unwrap();
    dag.validate().expect("valid");
    let err = Scheduler::new(
        dag,
        BudgetLedger::new(usd(1000)).expect("budget"),
        SchedulerConfig {
            max_in_flight: 4,
            resource_capacity: 8,
            memory_capacity_mb: 1024,
            max_queued: 2,
        },
    )
    .expect_err("queue over bound refused");
    assert!(
        matches!(
            err,
            SchedulerError::QueueExceedsBound {
                queued: 3,
                bound: 2
            }
        ),
        "named refusal, got: {err:?}"
    );
}

#[test]
fn s2_admission_queue_at_bound_dispatches() {
    // S2 positive: a queue exactly at the bound admits and drains fully.
    let mut dag = TaskDag::new(vec![]);
    dag.add(task("A", 1, 0)).unwrap();
    dag.add(task("B", 1, 0)).unwrap();
    dag.validate().expect("valid");
    let mut sched = scheduler_with_queue(dag, 2);
    let recorder = BatchRecorder {
        sizes: Mutex::new(Vec::new()),
    };
    let report = sched.run(&recorder).expect("run");
    assert_eq!(report.succeeded().len(), 2, "bound admits full drain");
}

#[test]
fn s2_admission_reservations_release_exactly_once() {
    // S2 release discipline: mixed terminal outcomes (success commits,
    // failure and cancel release, ambiguous releases then marks) leave
    // zero held reservations — each held exactly once, never leaked or
    // double-freed.
    let mut dag = TaskDag::new(vec![]);
    dag.add(costed_task("OK", 100)).unwrap();
    dag.add(costed_task("BAD", 100)).unwrap();
    dag.add(costed_task("STOP", 100)).unwrap();
    dag.add(costed_task("WEIRD", 100)).unwrap();
    dag.validate().expect("valid");
    let mut sched = scheduler(dag);
    let executor = ScriptedExecutor {
        outcomes: [
            ("OK".to_string(), TaskOutcome::Succeeded { cost: usd(100) }),
            (
                "BAD".to_string(),
                TaskOutcome::Failed {
                    reason: "BOOM".to_string(),
                },
            ),
            ("STOP".to_string(), TaskOutcome::Cancelled),
            (
                "WEIRD".to_string(),
                TaskOutcome::AmbiguousEffect {
                    reason: "maybe ran".to_string(),
                },
            ),
        ]
        .into_iter()
        .collect(),
    };
    let report = sched.run(&executor).expect("run");
    assert_eq!(report.succeeded(), &["OK".to_string()]);
    assert_eq!(report.failed(), &["BAD".to_string()]);
    assert_eq!(report.cancelled(), &["STOP".to_string()]);
    assert_eq!(report.unresolved(), &["WEIRD".to_string()]);
    assert_eq!(
        sched.budget().reserved(),
        usd(0),
        "no held reservation survives terminal outcomes"
    );
}

#[test]
fn s2_admission_queue_bytes_drain_with_dispatch() {
    // S2 regression: queued byte volume starts at the full admitted
    // cost and reaches zero once every task reaches a terminal state.
    let mut dag = TaskDag::new(vec![]);
    dag.add(costed_task("A", 100)).unwrap();
    dag.add(costed_task("B", 200)).unwrap();
    dag.validate().expect("valid");
    let mut sched = scheduler(dag);
    assert_eq!(sched.queue_bytes(), 300, "admission-sized queue");
    let recorder = ScriptedExecutor {
        outcomes: std::collections::HashMap::new(),
    };
    let report = sched.run(&recorder).expect("run");
    assert_eq!(report.succeeded().len(), 2);
    assert_eq!(sched.queue_bytes(), 0, "dispatch drains the queue");
}

fn sandbox_spec() -> IsolationSpec {
    // S3: small limit (64 MiB) with resident headroom, so the suite
    // exercises the real enforcement path on the shared host.
    IsolationSpec {
        declared_tools: vec!["python3".to_string()],
        env_allowlist: vec![],
        extra_ro_binds: vec![],
        wall_timeout_ms: 10_000,
        memory_bytes: 64 * 1024 * 1024,
        cpu_secs: 10,
        nproc: 64,
        fsize_bytes: 64 * 1024 * 1024,
        network: NetworkPolicy::Off,
        max_stdout: 4096,
        max_stderr: 4096,
        max_output_files: 8,
        max_output_file_bytes: 4096,
        max_output_bytes: 32 * 1024,
        keep_workdir: false,
    }
}

fn sandbox_run_argv(
    spec: IsolationSpec,
    argv: Vec<&str>,
) -> Result<hephaestus::sandbox::RunOutcome, hephaestus::sandbox::SandboxError> {
    SandboxProvider::with_default_tools().run(RunSpec {
        argv: argv.iter().map(|s| s.to_string()).collect(),
        spec,
        input_files: vec![],
    })
}

#[test]
fn s3_limits_measured_peak_stays_under_limit() {
    // S3 positive: a small worker reports a nonzero peak far under
    // the 64 MiB limit and exits clean.
    let out = sandbox_run_argv(sandbox_spec(), vec!["python3", "-c", "print('small')"])
        .expect("small worker runs");
    assert!(out.peak_rss_bytes > 0, "peak measures something");
    assert!(
        out.peak_rss_bytes < 64 * 1024 * 1024,
        "peak stays under the limit: {} bytes",
        out.peak_rss_bytes
    );
}

#[test]
fn s3_limits_memory_hog_dies_with_receipts() {
    // S3 refusal: a 256 MiB allocation against the 64 MiB limit dies
    // boundedly. RLIMIT_AS refuses the allocation before RSS grows, so
    // the receipt is the (peak, limit, stderr) triple — not a breached
    // peak. peak names what was measured, limit names the bound, and
    // stderr names the cause (MemoryError).
    let err = sandbox_run_argv(
        sandbox_spec(),
        vec![
            "python3",
            "-c",
            "x = bytearray(256 * 1024 * 1024); print(len(x))",
        ],
    )
    .expect_err("hog must die");
    match err {
        hephaestus::sandbox::SandboxError::WorkerFailed {
            exit_code,
            wall_ms,
            peak_rss_bytes,
            memory_limit_bytes,
            stderr,
            ..
        } => {
            assert!(exit_code != 0, "nonzero death");
            assert!(wall_ms < 10_000, "dies bounded in {wall_ms} ms");
            assert_eq!(memory_limit_bytes, 64 * 1024 * 1024);
            assert!(peak_rss_bytes > 0, "peak measured");
            assert!(
                peak_rss_bytes < memory_limit_bytes,
                "limit refused before RSS grew: peak={peak_rss_bytes}"
            );
            assert!(
                stderr.contains("MemoryError"),
                "stderr names the refused allocation"
            );
        }
        other => panic!("hog dies named, got {other:?}"),
    }
}

#[test]
fn s3_limits_clean_exit_carries_no_violation() {
    // S3 regression: a clean exit never sets the violation flag,
    // even when the peak approaches but stays under the limit.
    let out = sandbox_run_argv(
        sandbox_spec(),
        vec![
            "python3",
            "-c",
            "x = bytearray(8 * 1024 * 1024); print(len(x))",
        ],
    )
    .expect("8 MiB worker fits");
    assert!(out.peak_rss_bytes > 0, "peak measured");
    assert!(
        out.peak_rss_bytes < 64 * 1024 * 1024,
        "peak under the limit: {} bytes",
        out.peak_rss_bytes
    );
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
                max_queued: usize::MAX,
            },
        )
        .expect_err("oversize claim refused");
        assert!(
            matches!(err, SchedulerError::ResourceExceedsCapacity { .. }),
            "{id}: oversize refused by name, got: {err:?}"
        );
    }
}
