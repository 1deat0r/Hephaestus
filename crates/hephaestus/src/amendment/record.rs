//! Amendment records (T-035, R-100/R-101/R-114).

use serde::{Deserialize, Serialize};

/// Declared migration gates for an explicit version transition (R-114).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationGates {
    /// The old version's identity, preserved verbatim in the receipt.
    pub old_version: String,
    pub new_version: String,
    /// Declared schema compatibility gate result.
    pub schema_compatible: bool,
    /// Declared policy migration present.
    pub policy_migration_declared: bool,
}

/// Receipt errors — each named.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationError {
    /// Schema compatibility gate not passed.
    SchemaIncompatible,
    /// No policy migration declared for the transition.
    PolicyMigrationUndeclared,
}

/// An immutable migration receipt: the old identity is preserved
/// verbatim — historical identities never rewritten (R-114).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationReceipt {
    pub old_version: String,
    pub new_version: String,
    pub schema_compatible: bool,
    pub policy_migration_declared: bool,
}

/// A sealed-data holdout with monotonic exposure (R-101): once exposed,
/// never reset to sealed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Holdout {
    pub holdout_id: String,
    /// Has this holdout ever been exposed in any campaign?
    pub exposed: bool,
    /// Is this a FRESH holdout never touched by selection?
    pub fresh: bool,
    /// Separately qualified (independent oracle) — alternative to fresh.
    pub separately_qualified: bool,
    /// Authenticated identity of the accessor (R-101 tracking).
    pub last_accessor: Option<String>,
}

/// Holdout errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HoldoutError {
    /// The holdout was exposed and can no longer serve as sealed.
    ExposureResetForbidden,
    /// Challenger evaluated on stale (exposed, non-qualified) data.
    StaleHoldout,
}

/// Family binding (R-100): typed experiment family with selection
/// lineage, method qualification, and explicit error allocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FamilyBinding {
    pub family_id: String,
    /// Which campaigns selected this challenger (selection lineage).
    pub selection_lineage: Vec<String>,
    /// Method qualification reference (registry continuity).
    pub method_qualification: String,
    /// Family-level error allocation, declared BEFORE evaluation.
    pub error_allocation: f64,
}
