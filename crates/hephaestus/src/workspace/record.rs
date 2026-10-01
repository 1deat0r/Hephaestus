//! Workspace records (T-025, R-067/R-068/R-069).

use serde::{Deserialize, Serialize};

/// Persisted workspace state the views read from (R-067): never narrated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PersistedState {
    /// Entity counts per lifecycle state (name -> count).
    pub entity_counts: Vec<(String, u64)>,
    /// Last processed event id (event-cursor, R-068).
    pub last_event_id: u64,
    pub running: bool,
    /// Missions with unknown status stay unknown — no invention.
    pub missions_unknown: u64,
}

/// Truthful progress (R-067): counts from persisted state; unknown
/// stays unknown; NO percentage, NO narrative.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressView {
    pub entity_counts: Vec<(String, u64)>,
    pub last_event_id: u64,
    pub running: bool,
    pub missions_unknown: u64,
}

/// A persisted hypothesis record for comparison (read-only view).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HypothesisRecord {
    pub id: String,
    pub version: String,
    pub state: String,
    pub claim_ids: Vec<String>,
    pub has_discriminator: bool,
    pub evidence_ids: Vec<String>,
}

/// Comparison of two hypotheses (§2:42 hypothesis comparison).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonView {
    pub same_version: bool,
    pub same_state: bool,
    pub claims_only_in_a: Vec<String>,
    pub claims_only_in_b: Vec<String>,
    /// Shared evidence overlap.
    pub shared_evidence: Vec<String>,
}

/// Drill-down: the record + lineage + staleness (R-097, §2:42).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceView {
    pub record_id: String,
    pub lineage_ids: Vec<String>,
    pub stale: bool,
}

/// Typed intervention commands (R-068).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlCommand {
    Pause,
    Resume,
    CancelMission(String),
    SteerMission(String, String),
}

/// Auth token: a digest checked against the policy's authorized digests
/// (contract layer; NOT a crypto system).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthToken(pub String);

/// Control errors (R-068/R-069).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlError {
    /// Token not in the policy's authorized digests.
    Unauthorized,
    /// Command not permitted in the current state (e.g. pause while
    /// paused).
    InvalidState,
    /// Unknown mission id.
    UnknownMission,
}

/// Acknowledgment: the applied transition + new cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ack {
    pub applied: String,
    pub new_event_id: u64,
}

/// Gateway event outcomes (R-068 event-cursor recovery).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GatewayOutcome {
    /// Fresh event: applied.
    Applied(Ack),
    /// Already-processed event: replay recommendation, not an error.
    Duplicate(u64),
    /// Gap detected: the client should replay from the given id.
    StaleCursor { expect_from: u64 },
}
