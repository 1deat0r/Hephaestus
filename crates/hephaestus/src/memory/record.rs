//! Behavioral-memory records (T-039, R-109).

use serde::{Deserialize, Serialize};

/// A behavioral-memory artifact derived from a knowledge source. The
/// `current` flag is the invalidation surface: quarantine propagation
/// flips it to false — the record itself survives (immutable audit).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryArtifact {
    pub artifact_id: String,
    /// The knowledge source this artifact was derived from.
    pub source_id: String,
    pub digest: String,
    pub current: bool,
}

/// A label attached to an artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryLabel {
    pub label_id: String,
    pub artifact_id: String,
    pub current: bool,
}

/// Write authorization refusals — named (R-109 negative case).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteRefusal {
    /// No scoped grant covers the behavioral-memory target.
    UnauthorizedWrite,
}
