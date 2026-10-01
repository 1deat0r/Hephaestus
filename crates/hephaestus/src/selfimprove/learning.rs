//! Learning-reuse records (T-036, R-116 negative case).

use serde::{Deserialize, Serialize};

/// State of a learned entry's source dependency. State changes are
/// append-only records — history is never erased (R-116: holdout
/// history survives).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryState {
    /// Source unchanged since learning: reusable.
    Valid,
    /// Source altered after recording (digest mismatch): reuse
    /// invalidated.
    Stale,
    /// Source explicitly quarantined: reuse invalidated.
    Quarantined,
}

/// An append-only state change for a learned entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateChange {
    pub entry_id: String,
    pub to_state: EntryState,
    pub reason: String,
}

/// A learned entry as seen for reuse: the ledger entry plus its CURRENT
/// derived state (recomputed from the append-only change log).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningEntry {
    pub entry_id: String,
    /// Digest of the learning source recorded at learning time.
    pub source_digest: String,
    /// Fresh partition used — survives invalidation (R-116).
    pub fresh_partition_id: String,
    pub current_state: EntryState,
}
