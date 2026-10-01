//! The typed task DAG and bounded worker scheduler (T-009).
//!
//! Plan T-009: validate acyclicity, dependency outputs, read/write sets,
//! retry semantics, timeouts, resource limits, and cancellation; batch
//! trivial deterministic tasks. The scheduler owns no clock (timeouts are
//! executor-enforced) and no durability (T-011 owns recovery) — bounds are
//! in-process, matching MASTER_SPEC's local MVP.

pub mod core;
pub mod dag;
pub mod executor;

pub use core::{RunReport, Scheduler, SchedulerConfig, SchedulerError};
pub use dag::{DagViolation, PriorityClass, RetryPolicy, Task, TaskDag};
pub use executor::{TaskExecutor, TaskOutcome};
