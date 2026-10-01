//! The durable operation recorder (T-011).

use std::path::Path;

use crate::contracts::generated::{
    Event, EventKind, MissionDataOrigin, Provenance, RecordRef, SchemaVersion, TrustOrigin,
};
use crate::ledger::{EventLedger, LedgerError};

use super::replay;
use super::{EMPTY_SHA256, OPS_MISSION, OPS_POLICY, Receipt, now_rfc3339};

/// Recorder failures (every one leaves the ledger unchanged unless noted).
#[derive(Debug)]
pub enum RecorderError {
    /// Ledger/storage failure.
    Ledger(LedgerError),
    /// Operation id generation exhausted (u64 space — reported, never wrapped).
    IdExhausted,
    /// The operation id is unknown to this recorder.
    UnknownOperation {
        /// The offending id.
        id: String,
    },
    /// Receipt payload could not be serialized (never happens for the
    /// fixed Receipt shape — reported for completeness).
    Receipt(String),
}

impl std::fmt::Display for RecorderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecorderError::Ledger(e) => write!(f, "recorder ledger: {e}"),
            RecorderError::IdExhausted => write!(f, "operation id space exhausted"),
            RecorderError::UnknownOperation { id } => write!(f, "unknown operation: {id}"),
            RecorderError::Receipt(e) => write!(f, "receipt: {e}"),
        }
    }
}

impl std::error::Error for RecorderError {}

impl From<LedgerError> for RecorderError {
    fn from(e: LedgerError) -> Self {
        RecorderError::Ledger(e)
    }
}

/// Durable lifecycle recorder over the T-006 event ledger + artifact store
/// (single writer — grill Q9).
#[derive(Debug)]
pub struct OperationRecorder {
    ledger: EventLedger,
    store: crate::ledger::ArtifactStore,
    next_op: u64,
    next_sequence: i64,
}

impl OperationRecorder {
    /// Open (creating) the ledger and store; verifies the chain on open
    /// and derives the next operation id / sequence from history alone.
    pub fn open(ledger_path: &Path, store_root: &Path) -> Result<Self, RecorderError> {
        let ledger = EventLedger::open(ledger_path)?;
        let store = crate::ledger::ArtifactStore::open(store_root)?;
        let mut next_op = 0u64;
        let mut next_sequence = 1i64;
        for event in ledger.events() {
            if event.mission_ref.id == OPS_MISSION {
                next_sequence = next_sequence.max(event.sequence + 1);
            }
            // Operation ids look like OP-<n>-<tag>; derive from the
            // operation_id field (event ids are EVT-…).
            if let Some(n) = event
                .operation_id
                .strip_prefix("OP-")
                .and_then(|rest| rest.split('-').next().unwrap_or("").parse::<u64>().ok())
            {
                next_op = next_op.max(n + 1);
            }
        }
        Ok(OperationRecorder {
            ledger,
            store,
            next_op,
            next_sequence,
        })
    }

    /// The content-addressed receipt store (for orphan tests / reads).
    pub fn store(&self) -> &crate::ledger::ArtifactStore {
        &self.store
    }

    /// The underlying ledger (read model).
    pub fn ledger(&self) -> &EventLedger {
        &self.ledger
    }

    /// Record `operation.planned` — durable BEFORE anything may dispatch
    /// (MASTER_SPEC:369). Returns the fresh operation id.
    pub fn plan(&mut self, tag: &str) -> Result<String, RecorderError> {
        let id = self.next_operation_id(tag)?;
        self.append_event(&id, "operation.planned", None)?;
        Ok(id)
    }

    /// Record `operation.dispatched` for attempt `attempt` (append-before-
    /// effect — the RecordingExecutor calls this before delegating).
    pub fn dispatched(&mut self, op: &str, attempt: u32) -> Result<(), RecorderError> {
        self.ensure_known(op)?;
        let _ = attempt; // attempt lives in the receipt; the event marks the boundary
        self.append_event(op, "operation.dispatched", None)
    }

    /// Commit the receipt payload to the artifact store, then record
    /// `operation.receipt` with its `payload_sha256`. Crash between the two
    /// leaves an orphan artifact and NO receipt claim (events rule).
    pub fn receipt(&mut self, op: &str, receipt: &Receipt) -> Result<(), RecorderError> {
        self.ensure_known(op)?;
        let bytes =
            serde_json::to_vec(receipt).map_err(|e| RecorderError::Receipt(e.to_string()))?;
        let digest = {
            let staged = self.store.stage(&bytes)?;
            self.store.commit(&staged)?
        };
        self.append_event(op, "operation.receipt", Some(digest.as_str()))
    }

    /// Record cancellation intent durably (caller flips handles after).
    pub fn cancel_requested(&mut self, op: &str) -> Result<(), RecorderError> {
        self.ensure_known(op)?;
        self.append_event(op, "operation.cancel_requested", None)
    }

    /// Record that an output artifact was committed for this operation.
    pub fn output_committed(&mut self, op: &str, digest: &str) -> Result<(), RecorderError> {
        self.ensure_known(op)?;
        self.append_event(op, "operation.output_committed", Some(digest))
    }

    /// Durable marker that an ambiguous effect was reconciled. The
    /// reconciliation facts live in the budget's unresolved entry (its
    /// reason text); exactly-once is the recovery apply step's rule
    /// (AT-056). No note parameter — events carry no free-text payloads.
    pub fn reconciled(&mut self, op: &str) -> Result<(), RecorderError> {
        self.ensure_known(op)?;
        self.append_event(op, "operation.reconciled", None)
    }

    /// Pure replay: fold the verified chain into per-operation states and
    /// parsed receipts (receipt payloads read through the store by hash).
    pub fn replay(&self) -> super::OperationView {
        let events: Vec<Event> = self.ledger.events().to_vec();
        let store = &self.store;
        replay::fold(&events, |event| {
            if event.payload_sha256 == super::EMPTY_SHA256 {
                return None;
            }
            let bytes = store.read(&event.payload_sha256).ok()?;
            serde_json::from_slice::<Receipt>(&bytes).ok()
        })
    }

    /// Every ledger event of one lifecycle `event_type` (ledger order).
    pub fn events_of_type(&self, event_type: &str) -> Vec<Event> {
        self.ledger
            .events()
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// The receipt event for one operation, if recorded.
    pub fn receipt_events_for(&self, op: &str) -> Option<Event> {
        self.ledger
            .events()
            .iter()
            .find(|e| e.event_type == "operation.receipt" && e.operation_id == op)
            .cloned()
    }

    // --- internals -----------------------------------------------------

    fn ensure_known(&self, op: &str) -> Result<(), RecorderError> {
        let known = self
            .ledger
            .events()
            .iter()
            .any(|e| e.event_type == "operation.planned" && e.operation_id == op);
        if known {
            Ok(())
        } else {
            Err(RecorderError::UnknownOperation { id: op.to_string() })
        }
    }

    // Tag length is bounded by the contract event-id pattern (<= 64 chars
    // for the whole EVT-OP-<n>-<tag>-<seq> id); an overlong tag is refused
    // by `Event::validate` at append time (covered by a test).
    fn next_operation_id(&mut self, tag: &str) -> Result<String, RecorderError> {
        let n = self.next_op;
        self.next_op = self
            .next_op
            .checked_add(1)
            .ok_or(RecorderError::IdExhausted)?;
        let safe: String = tag
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        Ok(format!("OP-{n}-{safe}"))
    }

    fn append_event(
        &mut self,
        op: &str,
        event_type: &str,
        payload_sha256: Option<&str>,
    ) -> Result<(), RecorderError> {
        let sequence = self.next_sequence;
        let event = Event {
            id: format!("EVT-{op}-{sequence}"),
            schema_version: SchemaVersion::V1_2,
            record_version: 1,
            created_at: now_rfc3339(),
            data_origin: MissionDataOrigin::Live,
            provenance: Provenance {
                actor_id: "operation-recorder".to_string(),
                method: "t011-recorder".to_string(),
                input_refs: vec![],
                artifact_hashes: payload_sha256.map(|h| h.to_string()).into_iter().collect(),
                trust_origin: TrustOrigin::Owner,
            },
            kind: EventKind::Event,
            mission_ref: RecordRef {
                id: OPS_MISSION.to_string(),
                version: 1,
            },
            sequence,
            event_type: event_type.to_string(),
            subject_ref: RecordRef {
                id: op.to_string(),
                version: 1,
            },
            operation_id: op.to_string(),
            policy_version: OPS_POLICY.to_string(),
            payload_sha256: payload_sha256.unwrap_or(EMPTY_SHA256).to_string(),
            previous_event_sha256: None, // filled by the ledger
        };
        let mut event = event;
        self.ledger.append(&mut event)?;
        self.next_sequence += 1;
        Ok(())
    }
}
