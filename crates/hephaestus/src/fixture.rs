//! The M1 exit fixture (IMPLEMENTATION_PLAN:44): a constant five-task DAG
//! driven through the REAL control-plane stack by the CLI.
//!
//! Determinism: outcomes are constants keyed by task id; no I/O in the
//! executor; no clock or randomness in any decision. Ledger bytes contain
//! wall-clock `created_at` values by design — the printed summaries
//! exclude them, which is what the byte-equality test pins.
//!
//! Pillars wired into the fixture (plan line 44):
//! - recorded replay — every op's state derives from the ledger;
//! - denied stays denied — `gamma` fails terminally (non-retryable);
//! - bounded parallel budgets — `epsilon`'s reserve is refused (limit 100);
//! - ambiguous non-idempotent never duplicates — `delta` rests in
//!   `unresolved`, and recover never requeues it.
//!
//! (`epsilon` recovers as `planned` — nothing executed, so requeue is the
//! AT-057-correct outcome for a budget refusal against a non-durable
//! budget; the refusal itself is counted in the run summary.)

use serde::Serialize;

use hephaestus::budget::BudgetLedger;
use hephaestus::contracts::generated::Money;
use hephaestus::operations::OperationState;
use hephaestus::operations::{
    OperationRecorder, RecordingExecutor, RetryPosture, recover as recover_fn,
};
use hephaestus::scheduler::{
    RetryPolicy, Scheduler, SchedulerConfig, Task, TaskDag, TaskExecutor, TaskOutcome,
};

/// Fixture identity stamped in summaries.
pub const FIXTURE_NAME: &str = "m1-exit";
/// Authorized budget (integer minor units) — deliberately below the sum of
/// `beta` + `delta` + `epsilon` reservations.
pub const BUDGET_LIMIT: i64 = 100;

fn usd(minor_units: i64) -> Money {
    Money {
        currency: "USD".to_string(),
        minor_units,
    }
}

/// The five constant tasks (declaration order = dispatch priority order
/// within the Normal class).
pub fn build_dag() -> TaskDag {
    let mut dag = TaskDag::new(vec![]);
    for (id, cost) in [
        ("alpha", 0i64),
        ("beta", 60),
        ("gamma", 0),
        ("delta", 40),
        ("epsilon", 50),
    ] {
        let task = Task {
            id: id.to_string(),
            priority: hephaestus::scheduler::PriorityClass::Normal,
            depends_on: vec![],
            inputs: vec![],
            outputs: vec![format!("out-{id}")],
            retry: RetryPolicy { max_attempts: 1 },
            timeout_ms: 60_000,
            cost: usd(cost),
            trivial: false,
            // Single-attempt fixture: retries are off by construction, so
            // recover's posture below matches the recorded history.
            retryable: false,
            resource_units: 0,
            exclusive: false,
        };
        dag.add(task).expect("unique ids");
    }
    dag
}

/// Zero-I/O scripted executor: outcomes are constants keyed by task id.
pub struct DeterministicExecutor;

impl TaskExecutor for DeterministicExecutor {
    fn run(&self, task: &Task) -> TaskOutcome {
        match task.id.as_str() {
            "gamma" => TaskOutcome::Failed {
                reason: "scripted-denial".to_string(),
            },
            "delta" => TaskOutcome::AmbiguousEffect {
                reason: "scripted-unknown-effect".to_string(),
            },
            _ => TaskOutcome::Succeeded {
                cost: task.cost.clone(),
            },
        }
    }
}

/// One row of either summary: id + replayed state.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskState {
    /// Fixture task id.
    pub id: String,
    /// State derived purely from the ledger.
    pub state: OperationState,
}

/// Budget facts at the end of a run (recover has no live budget — it is
/// in-process by design, so recover summaries omit this block).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BudgetState {
    /// Authorized limit.
    pub limit: i64,
    /// Committed spend.
    pub spent: i64,
    /// Known unresolved holds.
    pub unresolved: i64,
    /// Reservations refused at dispatch (bounded-budget pillar).
    pub refused: u32,
}

/// `fixture run` stdout payload (fixed field order — golden bytes).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunSummary {
    /// Fixture identity.
    pub fixture: String,
    /// Replay-derived task states, ledger order (= declaration order).
    pub tasks: Vec<TaskState>,
    /// Budget facts.
    pub budget: BudgetState,
    /// Operations resting as ambiguous (executor-declared unknown effects)
    /// at the end of the run — the ambiguity count the spec asks for.
    pub ambiguous: u32,
}

/// Recovery plan buckets (ledger order within each).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanState {
    /// Safe to run again.
    pub requeue: Vec<String>,
    /// Needs budget reconciliation.
    pub unresolved: Vec<String>,
    /// Receipt-backed terminal states.
    pub terminal: Vec<String>,
    /// Durable cancel intents.
    pub cancelled: Vec<String>,
    /// Impossible histories.
    pub corrupt: Vec<String>,
}

/// `fixture recover` stdout payload (fixed field order).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecoverSummary {
    /// Fixture identity.
    pub fixture: String,
    /// Replay-derived task states.
    pub tasks: Vec<TaskState>,
    /// The AT-057 rule-table plan.
    pub plan: PlanState,
    /// Operations replayed from the ledger.
    pub replayed_ops: usize,
}

/// Operation ids look like `OP-<n>-<task id>` (recorder convention); the
/// summary speaks the operator's vocabulary: task ids.
fn task_id_from_op(op: &str) -> String {
    let mut parts = op.splitn(3, '-');
    let _seq = parts.next();
    let _index = parts.next();
    parts.next().unwrap_or(op).to_string()
}

fn ledger_paths(state_dir: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    (state_dir.join("ops-ledger.jsonl"), state_dir.join("store"))
}

/// Run the fixture into `state_dir` (must be empty or absent).
pub fn run(state_dir: &std::path::Path) -> Result<RunSummary, String> {
    if state_dir.exists() {
        let non_empty = std::fs::read_dir(state_dir)
            .map_err(|e| format!("state dir: {e}"))?
            .next()
            .is_some();
        if non_empty {
            return Err(format!(
                "state dir {} is not empty — refusing to mix histories (pick a fresh dir)",
                state_dir.display()
            ));
        }
    }
    std::fs::create_dir_all(state_dir).map_err(|e| format!("create state dir: {e}"))?;
    let (ledger, store) = ledger_paths(state_dir);

    let recorder = OperationRecorder::open(&ledger, &store).map_err(|e| e.to_string())?;
    let dag = build_dag();
    dag.validate()
        .map_err(|v| format!("fixture DAG invalid: {v:?}"))?;
    let mut recording = RecordingExecutor::new(DeterministicExecutor, recorder);
    for task in dag.tasks() {
        recording
            .plan_task(&task.id)
            .map_err(|e| format!("plan {}: {e}", task.id))?;
    }
    let budget = BudgetLedger::new(usd(BUDGET_LIMIT)).map_err(|e| format!("budget: {e}"))?;
    let mut sched = Scheduler::new(
        dag,
        budget,
        SchedulerConfig {
            max_in_flight: 8,
            resource_capacity: 8,
        },
    )
    .map_err(|e| format!("scheduler: {e:?}"))?;
    let report = sched.run(&recording).map_err(|e| format!("run: {e:?}"))?;

    let refused = report
        .failed()
        .iter()
        .filter(|id| report.failure_reason(id) == Some("BUDGET_UNAVAILABLE"))
        .count() as u32;

    let view = {
        let rec = recording.recorder();
        let guard = rec.lock().map_err(|e| format!("recorder lock: {e}"))?;
        guard.replay()
    };
    Ok(RunSummary {
        fixture: FIXTURE_NAME.to_string(),
        tasks: view
            .states
            .iter()
            .map(|(id, state)| TaskState {
                id: task_id_from_op(id),
                state: *state,
            })
            .collect(),
        budget: BudgetState {
            limit: BUDGET_LIMIT,
            spent: sched.budget().spent().minor_units,
            unresolved: sched.budget().unresolved().minor_units,
            refused,
        },
        ambiguous: report.unresolved().len() as u32,
    })
}

/// Replay `state_dir`'s ledger and print the recovery plan.
pub fn recover(state_dir: &std::path::Path) -> Result<RecoverSummary, String> {
    let (ledger, _store) = ledger_paths(state_dir);
    if !ledger.exists() {
        return Err(format!(
            "no fixture state at {} — run `hephaestus fixture run --state-dir <dir>` first",
            state_dir.display()
        ));
    }
    let recorder = OperationRecorder::open(&ledger, _store.as_path()).map_err(|e| e.to_string())?;
    let view = recorder.replay();
    let plan = recover_fn(&view, |_| RetryPosture {
        retryable: false, // fixture history is single-attempt by construction
        max_attempts: 1,
    });
    Ok(RecoverSummary {
        fixture: FIXTURE_NAME.to_string(),
        tasks: view
            .states
            .iter()
            .map(|(id, state)| TaskState {
                id: task_id_from_op(id),
                state: *state,
            })
            .collect(),
        plan: PlanState {
            // Speak task ids everywhere (same mapping as `tasks`).
            requeue: plan.requeue.iter().map(|op| task_id_from_op(op)).collect(),
            unresolved: plan
                .unresolved
                .iter()
                .map(|op| task_id_from_op(op))
                .collect(),
            terminal: plan.terminal.iter().map(|op| task_id_from_op(op)).collect(),
            cancelled: plan
                .cancelled
                .iter()
                .map(|op| task_id_from_op(op))
                .collect(),
            corrupt: plan.corrupt.iter().map(|op| task_id_from_op(op)).collect(),
        },
        replayed_ops: view.states.len(),
    })
}
