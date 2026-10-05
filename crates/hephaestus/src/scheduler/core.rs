//! The bounded worker scheduler (T-009).
//!
//! Deterministic, in-process, clockless: ready tasks dispatch as soon as
//! they pass priority-class order, max_in_flight, resource-unit and memory
//! capacity, and exclusivity against tasks already in flight (no wave
//! barrier); every priced task **reserves budget before dispatch**
//! (AT-055); outcomes map to commit / release / unresolved (R-056); retries
//! are bounded and non-idempotent work is never auto-retried
//! (MASTER_SPEC:371). Durability/replay belongs to T-011.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::budget::{BudgetError, BudgetLedger};
use crate::contracts::generated::Money;

use super::{DagViolation, Task, TaskDag, TaskExecutor, TaskOutcome};

/// Scheduler construction/execution errors (fail-closed).
#[derive(Debug)]
pub enum SchedulerError {
    /// The DAG failed validation; carries every violation.
    InvalidDag(Vec<DagViolation>),
    /// A task claims more resource units than the scheduler's capacity —
    /// it could never dispatch (would livelock the wave builder).
    ResourceExceedsCapacity {
        /// The oversized task.
        task: String,
        /// Declared units.
        units: u32,
        /// Scheduler capacity.
        capacity: u32,
    },
    /// A nonsensical configuration that could never make progress.
    InvalidConfig(String),
    /// Too many queued tasks were admitted for the configured bound —
    /// refused at construction, before any dispatch (S2 queue bound).
    QueueExceedsBound {
        /// Admitted task count.
        queued: usize,
        /// Configured bound.
        bound: usize,
    },
    /// A budget settlement failed in a way the invariants say cannot
    /// happen — reported loudly, never swallowed (AGENTS.md evidence rule).
    Settlement(String),
    /// Internal invariant broken (reported, never panicked across FFI).
    Stuck(String),
}

/// Bounds for one scheduler.
#[derive(Debug, Clone, Copy)]
pub struct SchedulerConfig {
    /// Maximum tasks in flight at once (must be >= 1).
    pub max_in_flight: usize,
    /// Resource units available across all in-flight tasks.
    pub resource_capacity: u32,
    pub memory_capacity_mb: u64,
    /// Maximum queued (pending, undispatched) tasks admitted at
    /// construction (must be >= 1). S2 bounds admission: a DAG with
    /// more queued tasks than this is refused before any dispatch.
    pub max_queued: usize,
}

/// What one `run` did. Lists are in declaration order; `dispatch_order`
/// records every dispatch **including retries** (each attempt is a dispatch).
#[derive(Debug, Clone, Default)]
pub struct RunReport {
    dispatch_order: Vec<String>,
    succeeded: Vec<String>,
    failed: Vec<String>,
    unresolved: Vec<String>,
    cancelled: Vec<String>,
    attempts: Vec<(String, u32)>,
    failure_reasons: Vec<(String, String)>,
}

impl RunReport {
    /// Ids in dispatch order (class order per round; retries re-listed).
    pub fn dispatch_order(&self) -> &[String] {
        &self.dispatch_order
    }
    /// Tasks that succeeded.
    pub fn succeeded(&self) -> &[String] {
        &self.succeeded
    }
    /// Tasks terminally failed (including budget and cost-mismatch failures).
    pub fn failed(&self) -> &[String] {
        &self.failed
    }
    /// Tasks whose effects are awaiting reconciliation (R-056).
    pub fn unresolved(&self) -> &[String] {
        &self.unresolved
    }
    /// Tasks cancelled before dispatch or by their executor.
    pub fn cancelled(&self) -> &[String] {
        &self.cancelled
    }
    /// Dispatch count for one task (visible attempt accounting).
    pub fn attempts(&self, id: &str) -> Option<u32> {
        self.attempts.iter().find(|(i, _)| i == id).map(|(_, n)| *n)
    }
    /// Why a task terminally failed (`COST_MISMATCH`, `BUDGET_UNAVAILABLE`,
    /// executor reasons, `TIMED_OUT`); cascaded descendants record state
    /// only.
    pub fn failure_reason(&self, id: &str) -> Option<&str> {
        self.failure_reasons
            .iter()
            .find(|(i, _)| i == id)
            .map(|(_, r)| r.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
enum NodeState {
    Pending,
    Succeeded,
    Failed,
    Cancelled,
    /// Terminal: effect recorded in the budget's unresolved limb.
    Unresolved,
}

/// A dispatched task awaiting completion: the resources it holds and the
/// reservation its settlement will commit or release exactly once.
#[derive(Debug)]
struct InFlight {
    units: u32,
    memory_mb: u64,
    exclusive: bool,
    held: Option<Money>,
}

/// One worker completion message: (task index, outcome) pairs, or the
/// executor panic payload, which re-raises fail-closed.
type WorkerMsg = std::thread::Result<Vec<(usize, TaskOutcome)>>;

/// The bounded scheduler over a validated [`TaskDag`] and a [`BudgetLedger`].
#[derive(Debug)]
pub struct Scheduler {
    dag: TaskDag,
    budget: BudgetLedger,
    config: SchedulerConfig,
    states: Vec<NodeState>,
    dispatch_counts: Vec<u32>,
    fail_reasons: HashMap<String, String>,
    cancel_flags: HashMap<String, Arc<AtomicBool>>,
    in_flight: HashMap<usize, InFlight>,
}

impl Scheduler {
    /// Fail-closed construction: the DAG must validate, every task's
    /// resource claim must fit the configured capacity, and the wave
    /// builder must be able to make progress at all.
    pub fn new(
        dag: TaskDag,
        budget: BudgetLedger,
        config: SchedulerConfig,
    ) -> Result<Self, SchedulerError> {
        if config.max_in_flight == 0 {
            return Err(SchedulerError::InvalidConfig(
                "max_in_flight must be >= 1".to_string(),
            ));
        }
        if config.max_queued == 0 {
            return Err(SchedulerError::InvalidConfig(
                "max_queued must be >= 1".to_string(),
            ));
        }
        dag.validate().map_err(SchedulerError::InvalidDag)?;
        for task in dag.tasks() {
            if task.cost.minor_units > 0 && task.cost.currency != budget.currency() {
                return Err(SchedulerError::InvalidConfig(format!(
                    "task {} costs {} but the ledger is {}",
                    task.id,
                    task.cost.currency,
                    budget.currency()
                )));
            }
            if task.resource_units > config.resource_capacity
                || task.memory_mb > config.memory_capacity_mb
            {
                return Err(SchedulerError::ResourceExceedsCapacity {
                    task: task.id.clone(),
                    units: task.resource_units,
                    capacity: config.resource_capacity,
                });
            }
        }
        // S2 queue bound: admission refuses a DAG whose queued task
        // count exceeds the configured bound, before any dispatch.
        if dag.tasks().len() > config.max_queued {
            return Err(SchedulerError::QueueExceedsBound {
                queued: dag.tasks().len(),
                bound: config.max_queued,
            });
        }
        let states = vec![NodeState::Pending; dag.tasks().len()];
        let dispatch_counts = vec![0u32; dag.tasks().len()];
        let fail_reasons = HashMap::new();
        // Cancel handles exist from construction so callers can grab them
        // before `run` starts (cooperative cancellation).
        let mut cancel_flags = HashMap::new();
        for task in dag.tasks() {
            cancel_flags.insert(task.id.clone(), Arc::new(AtomicBool::new(false)));
        }
        Ok(Scheduler {
            dag,
            budget,
            config,
            states,
            dispatch_counts,
            fail_reasons,
            cancel_flags,
            in_flight: HashMap::new(),
        })
    }

    /// The budget this scheduler reserves against (read model for tests).
    pub fn budget(&self) -> &BudgetLedger {
        &self.budget
    }

    /// Cooperative cancel handle for a still-pending task: setting the flag
    /// prevents its reservation and dispatch (MASTER_SPEC:375 — cancellation
    /// prevents new reservations) and cancels its descendants.
    pub fn cancel_handle(&self, task_id: &str) -> Option<Arc<AtomicBool>> {
        self.cancel_flags.get(task_id).cloned()
    }

    /// Run until no progress is possible; returns the deterministic report.
    ///
    /// Dispatch streams: each round starts every task that passes admission
    /// (reserve before dispatch, AT-055), then settles one completion
    /// message. There is no wave barrier, so a child can start while
    /// unrelated earlier work still runs (AC2). Worker threads only execute
    /// tasks and post outcomes; admission, state, budgets, settlement and
    /// the report stay on this thread (one owner). Report lists remain
    /// declaration ordered via collect_terminal; physical completion order
    /// may vary between runs and is not part of the contract.
    pub fn run<E: TaskExecutor>(&mut self, executor: &E) -> Result<RunReport, SchedulerError> {
        let mut report = RunReport::default();
        // The channel must outlive the scope region so its sender can be
        // lent to workers for as long as the scope can run.
        let (tx, rx) = std::sync::mpsc::channel::<WorkerMsg>();
        std::thread::scope(|scope| {
            loop {
                self.cascade(executor);

                let mut wave = self.select_wave();
                if wave.is_empty() && self.in_flight.is_empty() {
                    // Terminalization (cancel/failed deps) may cascade
                    // further — try once more before declaring stuck.
                    if self.cascade(executor) {
                        wave = self.select_wave();
                    }
                    if wave.is_empty() {
                        if self.states.contains(&NodeState::Pending) {
                            return Err(SchedulerError::Stuck(
                                "pending tasks with no dispatchable wave — dependency or budget deadlock"
                                    .to_string(),
                            ));
                        }
                        break;
                    }
                }

                if !wave.is_empty() {
                    self.dispatch_wave(&wave, executor, scope, &tx, &mut report)?;
                }
                if self.in_flight.is_empty() {
                    // Every selected task was budget-refused (now Failed):
                    // re-evaluate; no completion can arrive.
                    continue;
                }

                let msg = rx.recv().map_err(|_| {
                    SchedulerError::Stuck(
                        "completion channel closed while tasks are in flight".to_string(),
                    )
                })?;
                let results = match msg {
                    Ok(results) => results,
                    Err(payload) => std::panic::resume_unwind(payload),
                };
                for (i, outcome) in results {
                    let flight = self
                        .in_flight
                        .remove(&i)
                        .expect("settling task must be in flight");
                    self.resolve(i, outcome, flight.held)?;
                }
            }
            self.collect_terminal(&mut report);
            Ok(report)
        })
    }

    /// Reserve, record and spawn one admission round (no barrier).
    ///
    /// Phase 1 reserves every selected priced task before any spawn
    /// (AT-055): a budget refusal fails that task without dispatching it;
    /// an unexpected reservation error releases the fresh holds of this
    /// round, spawns nothing, and aborts. Phase 2 registers in-flight
    /// holds, then spawns workers: two or more unflagged trivial tasks
    /// share one run_batch call; everything else runs through
    /// run_cancellable.
    fn dispatch_wave<'scope, 'env, E: TaskExecutor>(
        &mut self,
        wave: &[usize],
        executor: &'scope E,
        scope: &'scope std::thread::Scope<'scope, 'env>,
        tx: &'scope std::sync::mpsc::Sender<WorkerMsg>,
        report: &mut RunReport,
    ) -> Result<(), SchedulerError> {
        // Phase 1: reserve before dispatch (AT-055), selection order.
        let mut dispatched: Vec<(usize, Option<Money>)> = Vec::new();
        for &i in wave {
            let task = self.dag.tasks()[i].clone();
            if task.cost.minor_units > 0 {
                match self.budget.reserve(&task.id, task.cost.clone()) {
                    Ok(()) => dispatched.push((i, Some(task.cost.clone()))),
                    Err(BudgetError::InsufficientAvailable { .. }) => {
                        // Budget refuses the reservation: the task cannot
                        // run — and nothing was dispatched for it.
                        self.states[i] = NodeState::Failed;
                        self.fail_reasons
                            .insert(task.id.clone(), "BUDGET_UNAVAILABLE".to_string());
                        continue;
                    }
                    Err(e) => {
                        // Defense in depth: release the fresh holds of this
                        // round before aborting — an abort must never leak.
                        let siblings: Vec<String> = dispatched
                            .iter()
                            .filter(|(_, held)| held.is_some())
                            .map(|(idx, _)| self.dag.tasks()[*idx].id.clone())
                            .collect();
                        let mut cleanup: Vec<String> = Vec::new();
                        for id in &siblings {
                            if let Err(e2) = self.budget.release(id) {
                                cleanup.push(format!("{id}: {e2}"));
                            }
                        }
                        let extra = if cleanup.is_empty() {
                            String::new()
                        } else {
                            format!(" (cleanup failures: {})", cleanup.join("; "))
                        };
                        return Err(SchedulerError::Settlement(format!(
                            "reservation for {}: {e}{extra}",
                            task.id
                        )));
                    }
                }
            } else {
                dispatched.push((i, None));
            }
            self.dispatch_counts[i] += 1;
            report.dispatch_order.push(task.id.clone());
        }

        // Phase 2: register holds, then spawn workers.
        let tasks: Vec<Task> = dispatched
            .iter()
            .map(|(i, _)| self.dag.tasks()[*i].clone())
            .collect();
        let flags: Vec<Arc<AtomicBool>> = tasks
            .iter()
            .map(|t| self.cancel_flags.get(&t.id).cloned().expect("flag exists"))
            .collect();
        for (k, (i, held)) in dispatched.iter().enumerate() {
            self.in_flight.insert(
                *i,
                InFlight {
                    units: tasks[k].resource_units,
                    memory_mb: tasks[k].memory_mb,
                    exclusive: tasks[k].exclusive,
                    held: held.clone(),
                },
            );
        }
        let batchable: Vec<usize> = (0..dispatched.len())
            .filter(|&k| tasks[k].trivial && !flags[k].load(Ordering::SeqCst))
            .collect();
        let batched: Vec<usize> = if batchable.len() > 1 {
            let batch: Vec<Task> = batchable.iter().map(|&k| tasks[k].clone()).collect();
            let indices: Vec<usize> = batchable.iter().map(|&k| dispatched[k].0).collect();
            scope.spawn(move || {
                let msg: WorkerMsg = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let outs = executor.run_batch(&batch);
                    // A short batch panics here, exactly as the
                    // pre-streaming loop did: fail closed, never a
                    // silent missing completion.
                    indices
                        .iter()
                        .enumerate()
                        .map(|(n, &i)| (i, outs[n].clone()))
                        .collect()
                }));
                let _ = tx.send(msg);
            });
            batchable
        } else {
            Vec::new()
        };
        for (k, (task, flag)) in tasks.iter().zip(flags.iter()).enumerate() {
            if batched.contains(&k) {
                continue;
            }
            let i = dispatched[k].0;
            let task = task.clone();
            let flag = flag.clone();
            scope.spawn(move || {
                let msg: WorkerMsg = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    executor.run_cancellable(&task, &flag)
                }))
                .map(|outcome| vec![(i, outcome)]);
                let _ = tx.send(msg);
            });
        }
        Ok(())
    }

    // --- internals -------------------------------------------------------

    /// One cascade pass: terminal states propagate to descendants and
    /// flagged pending tasks are signalled/cancelled. Returns true when any
    /// state changed (i.e. another pass is warranted).
    fn cascade<E: TaskExecutor>(&mut self, executor: &E) -> bool {
        let before = self.states.clone();
        self.propagate_terminal_to_descendants();
        self.apply_pre_dispatch_cancels(executor);
        self.states != before
    }

    fn propagate_terminal_to_descendants(&mut self) {
        for i in 0..self.dag.tasks().len() {
            if self.states[i] != NodeState::Pending {
                continue;
            }
            let deps = &self.dag.tasks()[i].depends_on;
            if deps.is_empty() {
                continue;
            }
            let mut any_cancelled = false;
            let mut any_failed = false;
            let mut all_done = true;
            for d in deps {
                match self.state_of(d) {
                    Some(NodeState::Succeeded) | Some(NodeState::Unresolved) => {}
                    Some(NodeState::Cancelled) => any_cancelled = true,
                    Some(NodeState::Failed) => any_failed = true,
                    _ => all_done = false,
                }
            }
            if !all_done {
                continue;
            }
            // Cascaded descendants are never dispatched, so they carry no
            // reservation; state alone records the outcome.
            self.states[i] = if any_cancelled {
                NodeState::Cancelled
            } else if any_failed {
                NodeState::Failed
            } else {
                continue;
            };
        }
    }

    fn apply_pre_dispatch_cancels<E: TaskExecutor>(&mut self, executor: &E) {
        for i in 0..self.dag.tasks().len() {
            if self.states[i] != NodeState::Pending {
                continue;
            }
            if self.in_flight.contains_key(&i) {
                // Already dispatched: cooperative cancellation reaches it
                // through the flag its worker holds; the hold settles once,
                // at completion (never pre-emptively).
                continue;
            }
            let id = self.dag.tasks()[i].id.clone();
            let flagged = self
                .cancel_flags
                .get(&id)
                .map(|f| f.load(Ordering::SeqCst))
                .unwrap_or(false);
            if flagged {
                // Signal first (the executor may retain resources for it),
                // then resolve — and never reserve for it (MASTER_SPEC:375).
                executor.signal_cancel(&id);
                self.states[i] = NodeState::Cancelled;
            }
        }
    }

    fn state_of(&self, id: &str) -> Option<&NodeState> {
        self.dag
            .tasks()
            .iter()
            .position(|t| t.id == id)
            .map(|i| &self.states[i])
    }

    /// Queued byte volume: the cost bytes of every task still pending
    /// (undispatched or awaiting retry). S2 names the queue the bound
    /// protects; admission-sized at construction, drained by dispatch.
    pub fn queue_bytes(&self) -> u64 {
        self.dag
            .tasks()
            .iter()
            .enumerate()
            .filter(|(i, _)| self.states[*i] == NodeState::Pending)
            .map(|(_, t)| t.cost.minor_units.max(0) as u64)
            .sum()
    }

    fn ready_indices(&self) -> Vec<usize> {
        self.dag
            .tasks()
            .iter()
            .enumerate()
            .filter(|(i, t)| {
                self.states[*i] == NodeState::Pending
                    && !self.in_flight.contains_key(i)
                    && t.depends_on.iter().all(|d| {
                        matches!(
                            self.state_of(d),
                            Some(NodeState::Succeeded) | Some(NodeState::Unresolved)
                        )
                    })
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Priority class first (Critical < Normal < Verification < Exploration),
    /// declaration order within class; bounded by max_in_flight minus tasks
    /// already in flight, remaining resource-unit and memory capacity, and
    /// mutual exclusion of exclusive tasks against in-flight work.
    fn select_wave(&self) -> Vec<usize> {
        // An exclusive task runs alone: admit nothing while any task runs.
        if self.in_flight.values().any(|f| f.exclusive) {
            return Vec::new();
        }
        let slots = self
            .config
            .max_in_flight
            .saturating_sub(self.in_flight.len());
        if slots == 0 {
            return Vec::new();
        }
        let mut candidates = self.ready_indices();
        candidates.sort_by_key(|&i| (self.dag.tasks()[i].priority, i));
        let mut wave: Vec<usize> = Vec::new();
        let mut units: u32 = self.in_flight.values().map(|f| f.units).sum();
        let mut memory: u64 = self.in_flight.values().map(|f| f.memory_mb).sum();
        for i in candidates {
            if wave.len() >= slots {
                break;
            }
            let task = &self.dag.tasks()[i];
            let wave_has_exclusive = wave.iter().any(|&j| self.dag.tasks()[j].exclusive);
            if task.exclusive && (!wave.is_empty() || !self.in_flight.is_empty()) {
                continue;
            }
            if wave_has_exclusive {
                continue;
            }
            if units + task.resource_units > self.config.resource_capacity {
                continue;
            }
            if memory + task.memory_mb > self.config.memory_capacity_mb {
                continue;
            }
            wave.push(i);
            units += task.resource_units;
            memory += task.memory_mb;
        }
        wave
    }

    /// Apply one executor outcome: commit exactly, release, map to
    /// unresolved, or fail — with retries bounded by policy.
    fn resolve(
        &mut self,
        i: usize,
        outcome: TaskOutcome,
        held: Option<Money>,
    ) -> Result<(), SchedulerError> {
        let id = self.dag.tasks()[i].id.clone();
        let declared = self.dag.tasks()[i].cost.clone();
        let retryable = self.dag.tasks()[i].retryable;
        let max_attempts = self.dag.tasks()[i].retry.max_attempts;

        match outcome {
            TaskOutcome::Succeeded { cost } => {
                if cost != declared {
                    // Deterministic M1 pricing: mismatch fails the task and
                    // returns the reservation.
                    self.release(&id, held)?;
                    self.states[i] = NodeState::Failed;
                    self.fail_reasons.insert(id, "COST_MISMATCH".to_string());
                    return Ok(());
                }
                if held.is_some()
                    && let Err(e) = self.budget.commit(&id, declared)
                {
                    {
                        // Never leave a held reservation behind a failed
                        // commit (review pass 1): release, then fail.
                        let release_result = self.budget.release(&id);
                        self.states[i] = NodeState::Failed;
                        return match release_result {
                            Ok(()) => Err(SchedulerError::Settlement(format!(
                                "commit for {id} failed after dispatch: {e}"
                            ))),
                            Err(e2) => Err(SchedulerError::Settlement(format!(
                                "commit for {id} failed ({e}) AND release failed ({e2})"
                            ))),
                        };
                    }
                }
                self.states[i] = NodeState::Succeeded;
            }
            TaskOutcome::Failed { reason } => {
                self.fail_attempt(i, &id, reason, held, retryable, max_attempts)?;
            }
            TaskOutcome::TimedOut => {
                self.fail_attempt(
                    i,
                    &id,
                    "TIMED_OUT".to_string(),
                    held,
                    retryable,
                    max_attempts,
                )?;
            }
            TaskOutcome::Cancelled => {
                self.release(&id, held)?;
                self.states[i] = NodeState::Cancelled;
            }
            TaskOutcome::AmbiguousEffect { reason } => {
                // Free the reservation, then record the charge: known
                // amount when one was held, otherwise an amount-less entry
                // with a mandatory reason (never a silent zero).
                self.release(&id, held)?;
                let reason = if reason.trim().is_empty() {
                    "unspecified ambiguous effect".to_string()
                } else {
                    reason
                };
                let mark = if declared.minor_units > 0 {
                    Some(declared)
                } else {
                    None
                };
                self.budget
                    .mark_unresolved(&id, mark, &reason)
                    .map_err(|e| {
                        SchedulerError::Settlement(format!("mark_unresolved for {id}: {e}"))
                    })?;
                self.states[i] = NodeState::Unresolved;
            }
        }
        Ok(())
    }

    /// Failure path shared by `Failed` and `TimedOut`: release the
    /// reservation, honor retry policy, remember the attempt.
    fn fail_attempt(
        &mut self,
        i: usize,
        id: &str,
        reason: String,
        held: Option<Money>,
        retryable: bool,
        max_attempts: u32,
    ) -> Result<(), SchedulerError> {
        self.release(id, held)?;
        let attempts_so_far = self.dispatch_counts[i];
        if retryable && attempts_so_far < max_attempts {
            self.states[i] = NodeState::Pending; // retry next wave
            return Ok(());
        }
        self.states[i] = NodeState::Failed;
        self.fail_reasons.insert(id.to_string(), reason);
        Ok(())
    }

    fn release(&mut self, id: &str, held: Option<Money>) -> Result<(), SchedulerError> {
        if held.is_some() {
            self.budget
                .release(id)
                .map_err(|e| SchedulerError::Settlement(format!("release for {id}: {e}")))?;
        }
        Ok(())
    }

    fn collect_terminal(&self, report: &mut RunReport) {
        for (i, task) in self.dag.tasks().iter().enumerate() {
            match &self.states[i] {
                NodeState::Succeeded => report.succeeded.push(task.id.clone()),
                NodeState::Failed => report.failed.push(task.id.clone()),
                NodeState::Cancelled => report.cancelled.push(task.id.clone()),
                NodeState::Unresolved => report.unresolved.push(task.id.clone()),
                NodeState::Pending => {}
            }
            if self.dispatch_counts[i] > 0 {
                report
                    .attempts
                    .push((task.id.clone(), self.dispatch_counts[i]));
            }
            if let Some(reason) = self.fail_reasons.get(&task.id) {
                report
                    .failure_reasons
                    .push((task.id.clone(), reason.clone()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::generated::Money;
    use crate::scheduler::{PriorityClass, RetryPolicy, Task, TaskDag};

    fn usd(minor_units: i64) -> Money {
        Money {
            currency: "USD".to_string(),
            minor_units,
        }
    }

    fn task(id: &str, units: u32, exclusive: bool) -> Task {
        Task {
            id: id.to_string(),
            priority: PriorityClass::Normal,
            depends_on: vec![],
            inputs: vec![],
            outputs: vec![format!("out-{id}")],
            retry: RetryPolicy { max_attempts: 1 },
            timeout_ms: 1,
            cost: usd(0),
            trivial: false,
            retryable: true,
            resource_units: units,
            memory_mb: 0,
            exclusive,
        }
    }

    fn scheduler_for(dag: TaskDag, max_in_flight: usize, capacity: u32) -> Scheduler {
        Scheduler::new(
            dag,
            BudgetLedger::new(usd(100)).expect("budget"),
            SchedulerConfig {
                max_in_flight,
                resource_capacity: capacity,
                memory_capacity_mb: 1024,
                max_queued: usize::MAX,
            },
        )
        .expect("scheduler")
    }

    #[test]
    fn wave_builder_enforces_exclusivity_both_directions() {
        // Exclusive added first: it takes the wave alone.
        let mut dag = TaskDag::new(vec![]);
        dag.add(task("EXCL", 1, true)).unwrap();
        dag.add(task("A", 1, false)).unwrap();
        dag.add(task("B", 1, false)).unwrap();
        let sched = scheduler_for(dag, 4, 4);
        assert_eq!(sched.select_wave(), vec![0], "exclusive runs alone");

        // Normals first: exclusive waits for an empty wave.
        let mut dag = TaskDag::new(vec![]);
        dag.add(task("A", 1, false)).unwrap();
        dag.add(task("EXCL", 1, true)).unwrap();
        let sched = scheduler_for(dag, 4, 4);
        assert_eq!(sched.select_wave(), vec![0], "exclusive deferred");
    }

    #[test]
    fn wave_builder_respects_max_in_flight_and_resource_capacity() {
        let mut dag = TaskDag::new(vec![]);
        for i in 0..5 {
            dag.add(task(&format!("T{i}"), 2, false)).unwrap();
        }
        // capacity 3 units => only one 2-unit task per wave despite
        // max_in_flight 4.
        let sched = scheduler_for(dag, 4, 3);
        assert_eq!(sched.select_wave().len(), 1);
        // capacity 8, max 4 => four tasks.
        let mut dag = TaskDag::new(vec![]);
        for i in 0..5 {
            dag.add(task(&format!("T{i}"), 2, false)).unwrap();
        }
        let sched = scheduler_for(dag, 4, 8);
        assert_eq!(sched.select_wave().len(), 4);

        // Oversized claims refused at construction (would livelock).
        let mut dag = TaskDag::new(vec![]);
        dag.add(task("HUGE", 9, false)).unwrap();
        assert!(matches!(
            Scheduler::new(
                dag,
                BudgetLedger::new(usd(10)).expect("b"),
                SchedulerConfig {
                    max_in_flight: 2,
                    resource_capacity: 4,
                    memory_capacity_mb: 1024,
                    max_queued: usize::MAX,
                },
            ),
            Err(SchedulerError::ResourceExceedsCapacity { .. })
        ));
        // A wave of zero could never dispatch anything.
        let mut dag = TaskDag::new(vec![]);
        dag.add(task("T", 1, false)).unwrap();
        assert!(matches!(
            Scheduler::new(
                dag,
                BudgetLedger::new(usd(10)).expect("b"),
                SchedulerConfig {
                    max_in_flight: 0,
                    resource_capacity: 4,
                    memory_capacity_mb: 1024,
                    max_queued: usize::MAX,
                },
            ),
            Err(SchedulerError::InvalidConfig(_))
        ));
    }
}
