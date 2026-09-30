//! The derived, rebuildable timeline projection.

use std::path::Path;

use crate::contracts::generated::Event;

use super::{EventLedger, LedgerError, json_error};

/// One row of the per-mission timeline projection.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TimelineEntry {
    /// Mission the event belongs to.
    pub mission_id: String,
    /// Event sequence within the mission.
    pub sequence: i64,
    /// Contract id of the event.
    pub event_id: String,
    /// Event type string.
    pub event_type: String,
    /// Operation the event records.
    pub operation_id: String,
}

/// A derived, rebuildable timeline view — never authoritative (T-006).
///
/// Views are written only from events already durable in the ledger
/// (append-then-project); losing this file loses nothing that
/// [`Timeline::rebuild`] cannot restore from the ledger.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Timeline {
    entries: Vec<TimelineEntry>,
}

impl Timeline {
    /// Derive the view from every event in `ledger`, in ledger order.
    pub fn rebuild(ledger: &EventLedger) -> Self {
        let entries = ledger.events().iter().map(Timeline::row_for).collect();
        Timeline { entries }
    }

    /// Rows of the view.
    pub fn entries(&self) -> &[TimelineEntry] {
        &self.entries
    }

    /// Persist the view to `path`.
    pub fn write(&self, path: &Path) -> Result<(), LedgerError> {
        let bytes = serde_json::to_vec_pretty(self).map_err(json_error)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Load a previously persisted view.
    pub fn load(path: &Path) -> Result<Self, LedgerError> {
        let bytes = std::fs::read(path)?;
        serde_json::from_slice(&bytes).map_err(json_error)
    }

    fn row_for(event: &Event) -> TimelineEntry {
        TimelineEntry {
            mission_id: event.mission_ref.id.clone(),
            sequence: event.sequence,
            event_id: event.id.clone(),
            event_type: event.event_type.clone(),
            operation_id: event.operation_id.clone(),
        }
    }
}
