//! Backend records (T-030, R-051/R-057/R-066).

use serde::{Deserialize, Serialize};

/// A task the backend can execute: a deterministic function of its input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub task_id: String,
    /// Declared read set (policy boundary, T-009 continuity).
    pub read_set: Vec<String>,
    /// Declared write set.
    pub write_set: Vec<String>,
    /// Deterministic computation: input bytes -> output bytes via a
    /// registered function name.
    pub function: String,
    pub input: String,
    pub budget_cost: u64,
}

/// Dispatch errors — each named.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DispatchError {
    /// Declared sets empty: policy boundary requires them.
    MissingPolicySets,
    /// Budget exhausted (R-055 continuity).
    BudgetExhausted,
    /// Same task id already completed with the same digest: no duplicate
    /// side effects.
    AlreadyCompleted,
    /// Function not registered with the backend.
    UnknownFunction,
}

impl std::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Task states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Running,
    Completed,
    Cancelled,
}

/// A durable dispatch receipt (R-057): recorded output bytes + digest,
/// budget spent, cursor. Survives restart via the append-only ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchReceipt {
    pub task_id: String,
    pub state: TaskState,
    pub output: String,
    pub output_digest: String,
    pub budget_spent: u64,
    pub cursor: u64,
}

/// Contract-check findings — one per clause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractFinding {
    pub clause: String, // cancellation | durable_receipts | policy_boundaries | budgets | replay
    pub ok: bool,
    pub detail: String,
}

/// The contract report (R-066 path: inspect + contract-test BEFORE any
/// compatibility claim).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContractReport {
    pub findings: Vec<ContractFinding>,
    pub passes: bool,
}

/// The compatibility stance: Tachyon is BLOCKED until a real protocol is
/// inspected (AGENTS.md + R-066). No compatibility claim is made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TachyonStatus {
    /// No protocol inspected; no integration attempted; no claim made.
    BlockedPendingRealProtocol,
}
