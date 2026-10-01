//! Import-closure resolution records (T-042, R-102).

use serde::{Deserialize, Serialize};

/// One resolved import edge: the importing module, the target as
/// written, and the canonical resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportEdge {
    pub from_module: String,
    pub target: String,
    /// Canonical resolution: `crate::<path>`, `super::<path>`, or the
    /// aliased target's real path. Empty when resolution is
    /// unsupported (dynamic import, external crate).
    pub resolved: Option<String>,
    /// Why resolution is unsupported, when `resolved` is None.
    pub unresolved_reason: Option<String>,
}

/// One mismatch between the declared closure and the resolved imports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClosureMismatch {
    /// Declared but no resolved import reaches it.
    Stale { module: String },
    /// Resolved import not covered by the declaration.
    Missing { module: String },
}

/// The verdict of a closure check. Conservative: any unresolved import
/// or mismatch blocks; nothing is silently omitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClosureVerdict {
    /// Declared set == resolved set, no unresolved imports.
    Matched,
    /// Named mismatches (stale/missing) — closure not accepted.
    Mismatched(Vec<ClosureMismatch>),
    /// Unsupported resolution present — blocked without omission.
    BlockedUnresolved(Vec<String>),
}

/// Detected import cycle: the modules forming it, in order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportCycle {
    pub modules: Vec<String>,
}

/// Oracle disagreement: the two oracle digests differ.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleDisagreement {
    pub oracle_a_digest: String,
    pub oracle_b_digest: String,
}
