//! Learning-reuse gates (T-036, R-116 negative case).
//!
//! Stale/quarantined dependencies invalidate reuse; holdout history
//! survives (append-only state records, never erasure).

use serde::{Deserialize, Serialize};

use super::learning::{EntryState, LearningEntry, StateChange};

/// Apply an append-only state change. The base entry is never mutated
/// in the ledger; this produces the updated derived view (history
/// survives — the old view remains constructible).
pub fn apply_state_change(
    entry: &LearningEntry,
    change: &StateChange,
) -> Result<LearningEntry, String> {
    if change.entry_id != entry.entry_id {
        return Err("state change targets a different entry".to_string());
    }
    // An invalidated entry cannot silently return to Valid: only a
    // fresh learning record from a revalidated source would, and that
    // is a new entry, not a flip.
    if entry.current_state != EntryState::Valid && change.to_state == EntryState::Valid {
        return Err("invalidated entries cannot return to Valid".to_string());
    }
    Ok(LearningEntry {
        entry_id: entry.entry_id.clone(),
        source_digest: entry.source_digest.clone(),
        fresh_partition_id: entry.fresh_partition_id.clone(),
        current_state: change.to_state,
    })
}

/// The reuse decision for a subsequent mission (R-116): only Valid
/// entries whose recorded source digest still matches the CURRENT
/// source digest are usable. Stale/quarantined entries are excluded
/// from reuse but remain visible (history survives).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReuseDecision {
    Usable,
    StaleSource,
    Quarantined,
}

pub fn usable_for_reuse(entry: &LearningEntry, current_source_digest: &str) -> ReuseDecision {
    if entry.current_state == EntryState::Quarantined {
        return ReuseDecision::Quarantined;
    }
    if entry.current_state == EntryState::Stale || entry.source_digest != current_source_digest {
        return ReuseDecision::StaleSource;
    }
    ReuseDecision::Usable
}
