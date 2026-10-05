//! The executor capability contract (T-009).
//!
//! AGENTS.md: keep execution providers behind capability contracts. T-010's
//! sandboxed worker plugs in here; the scheduler sees only these outcomes.

use std::sync::atomic::{AtomicBool, Ordering};

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

/// Fail-closed contract errors (AE-01 S1): construction and admission
/// refuse invalid bounds, slots, and identities before any dispatch.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutorContractError {
    /// A bound that could never admit work (carries what it bounds).
    ZeroBound {
        /// What the bound limits ("in-flight", "queued").
        what: String,
    },
    /// A submission slot outside the gate's bound.
    SlotOverBound {
        /// Requested slot.
        slot: usize,
        /// Gate bound.
        bound: usize,
    },
    /// An empty task id carries no completion identity.
    EmptyTaskId,
    /// Attempt numbers start at 1; zero names no dispatch.
    ZeroAttempt,
    /// A signalled cancellation observed before completion.
    Cancelled {
        /// Task that was cancelled.
        task_id: String,
    },
}

impl std::fmt::Display for ExecutorContractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutorContractError::ZeroBound { what } => {
                write!(f, "{what} bound must be >= 1")
            }
            ExecutorContractError::SlotOverBound { slot, bound } => {
                write!(f, "slot {slot} exceeds bound {bound}")
            }
            ExecutorContractError::EmptyTaskId => write!(f, "task id must not be empty"),
            ExecutorContractError::ZeroAttempt => write!(f, "attempt must be >= 1"),
            ExecutorContractError::Cancelled { task_id } => {
                write!(f, "task {task_id} cancelled")
            }
        }
    }
}

impl std::error::Error for ExecutorContractError {}

/// Bounded submission (AE-01 S1 / R-055): a gate admits dispatch slots
/// below its bound and refuses the rest before any dispatch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubmissionGate {
    bound: usize,
}

impl SubmissionGate {
    /// A zero bound could never admit work — refused at construction.
    pub fn new(bound: usize, what: &str) -> Result<Self, ExecutorContractError> {
        if bound == 0 {
            return Err(ExecutorContractError::ZeroBound {
                what: what.to_string(),
            });
        }
        Ok(SubmissionGate { bound })
    }

    /// The bound slots are checked against.
    pub fn bound(&self) -> usize {
        self.bound
    }

    /// Admit one slot; refuses empty ids and slots at or past the bound.
    pub fn admit(&self, task_id: &str, slot: usize) -> Result<Submission, ExecutorContractError> {
        if task_id.is_empty() {
            return Err(ExecutorContractError::EmptyTaskId);
        }
        if slot >= self.bound {
            return Err(ExecutorContractError::SlotOverBound {
                slot,
                bound: self.bound,
            });
        }
        Ok(Submission {
            task_id: task_id.to_string(),
            slot,
        })
    }
}

/// One admitted dispatch: task identity plus its bounded slot.
#[derive(Debug, Clone, PartialEq)]
pub struct Submission {
    /// Task admitted for dispatch.
    pub task_id: String,
    /// Slot below the gate bound.
    pub slot: usize,
}

/// Completion identity (AE-01 S1 / R-057): task id plus attempt number.
/// Retries of one task carry distinct identities; a re-stated identity
/// is a duplicate replay, never a new effect.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Completion {
    /// Task that completed.
    pub task_id: String,
    /// Dispatch attempt, starting at 1.
    pub attempt: u32,
}

impl Completion {
    /// Refuses empty ids and zero attempts before recording anything.
    pub fn new(task_id: &str, attempt: u32) -> Result<Self, ExecutorContractError> {
        if task_id.is_empty() {
            return Err(ExecutorContractError::EmptyTaskId);
        }
        if attempt == 0 {
            return Err(ExecutorContractError::ZeroAttempt);
        }
        Ok(Completion {
            task_id: task_id.to_string(),
            attempt,
        })
    }

    /// A completion that re-states a recorded identity is a duplicate.
    pub fn is_duplicate_of(&self, other: &Completion) -> bool {
        self == other
    }

    /// A later attempt at the same task is a retry, not a duplicate.
    pub fn is_retry_of(&self, other: &Completion) -> bool {
        self.task_id == other.task_id && self.attempt > other.attempt
    }
}

/// Cooperative cancellation (AE-01 S1 / R-057): the scheduler signals the
/// flag for work it wants stopped; a cooperative executor observes the
/// signal and reports [`TaskOutcome::Cancelled`].
#[derive(Debug, Default)]
pub struct Cancellation {
    flag: AtomicBool,
}

impl Cancellation {
    /// Unsignalled: work may proceed.
    pub fn new() -> Self {
        Cancellation {
            flag: AtomicBool::new(false),
        }
    }

    /// Signal cancellation for the flagged work.
    pub fn signal(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    /// Whether cancellation was signalled.
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    /// Fail-closed check: signalled work reports cancellation.
    pub fn check(&self, task_id: &str) -> Result<(), ExecutorContractError> {
        if self.is_cancelled() {
            return Err(ExecutorContractError::Cancelled {
                task_id: task_id.to_string(),
            });
        }
        Ok(())
    }

    /// The underlying flag, for [`TaskExecutor::run_cancellable`] interop.
    pub fn flag(&self) -> &AtomicBool {
        &self.flag
    }
}

/// Normalize an ambiguous-effect reason (AE-01 S1 / R-056): blank reasons
/// become an explicit default so the unresolved limb never holds a silent
/// empty string. Non-blank reasons pass through verbatim.
///
/// Matches the scheduler's `resolve` default exactly.
pub fn normalize_ambiguous_reason(reason: &str) -> String {
    if reason.trim().is_empty() {
        "unspecified ambiguous effect".to_string()
    } else {
        reason.to_string()
    }
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
