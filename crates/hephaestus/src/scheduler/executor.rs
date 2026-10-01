//! The executor capability contract (T-009).
//!
//! AGENTS.md: keep execution providers behind capability contracts. T-010's
//! sandboxed worker plugs in here; the scheduler sees only these outcomes.

use std::sync::atomic::AtomicBool;

use crate::contracts::generated::Money;

use super::Task;

/// What an executor reports back for one dispatch.
#[derive(Debug, Clone, PartialEq)]
pub enum TaskOutcome {
    /// Completed; `cost` must equal the task's declared cost (deterministic
    /// M1 pricing) or the scheduler fails the task `COST_MISMATCH`.
    Succeeded {
        /// Actual cost as reported.
        cost: Money,
    },
    /// Deterministic-or-not failure; eligible for retry only when the task
    /// is `retryable`.
    Failed {
        /// Failure category/reason.
        reason: String,
    },
    /// Executor-enforced timeout (the scheduler owns no clock).
    TimedOut,
    /// Cancellation observed by the executor.
    Cancelled,
    /// A possibly-completed external effect the executor cannot classify —
    /// lands in the budget ledger's unresolved limb (R-056), never retried.
    AmbiguousEffect {
        /// Why the effect is ambiguous.
        reason: String,
    },
}

/// Capability contract between scheduler and workers (T-009 / T-010).
pub trait TaskExecutor: Send + Sync {
    /// Execute one task to completion.
    fn run(&self, task: &Task) -> TaskOutcome;

    /// Execute a batch of trivial tasks; default = per-task loop. The
    /// scheduler groups simultaneously-ready trivial work into one call —
    /// a batching worker overrides this for efficiency.
    fn run_batch(&self, tasks: &[Task]) -> Vec<TaskOutcome> {
        tasks.iter().map(|t| self.run(t)).collect()
    }

    /// Cooperative cancellation: the scheduler sets the flag for work it
    /// wants stopped; an executor that cannot interrupt itself may ignore
    /// it (M1 contract), but a cooperative one returns [`TaskOutcome::Cancelled`].
    fn run_cancellable(&self, task: &Task, cancel: &AtomicBool) -> TaskOutcome {
        let _ = cancel;
        self.run(task)
    }

    /// Explicit cancel signal for a task the scheduler believes may be
    /// running (called before a selected-but-unstarted dispatch is
    /// abandoned, and available for future async executors).
    fn signal_cancel(&self, _task_id: &str) {}
}
