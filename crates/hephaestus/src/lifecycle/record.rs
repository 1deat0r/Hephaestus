//! Evidence-lifecycle records (T-024, MASTER_SPEC §16).

use serde::{Deserialize, Serialize};

/// Opportunity states (§16:311).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpportunityState {
    Discovered,
    Grounded,
    Prioritized,
    Explored,
    Parked,
    Closed,
}

/// Hypothesis states (§16:311), with blocked/archived available from
/// relevant stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HypothesisState {
    Exploratory,
    Compiled,
    Reviewed,
    TestReady,
    Testing,
    Assessed,
    Blocked,
    Archived,
}

/// Orthogonal candidate fields (§16:312, R-046): four separate
/// dimensions, never collapsed into one score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateFields {
    pub scientific: String,
    pub engineering: String,
    pub novelty: String,
    pub execution: String,
}

/// A named illegal transition (§16:311).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionError {
    pub from: String,
    pub to: String,
    pub reason: String,
}

/// An append-only correction/retraction event (§16:314, R-017): the
/// prior record is never erased; the correction references it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Correction {
    /// ID of the record being corrected/retracted.
    pub target_id: String,
    /// ID of the new (correction) event itself.
    pub correction_id: String,
    pub kind: CorrectionKind,
    pub reason: String,
    /// Snapshot the correction binds (§16:318).
    pub snapshot_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrectionKind {
    CorrectedObservation,
    Retraction,
    VersionChange,
    NewPriorArt,
}

/// A dependent record marked stale by invalidation (§16:314).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaleMark {
    pub dependent_id: String,
    pub via: String,
    /// Historical result intact; current label stale (§16:317).
    pub historical_result_intact: bool,
}

/// A queued re-evaluation entry: AWAITS budget+permission (§16:314 —
/// no automatic expensive/external rework).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReevaluationEntry {
    pub target_id: String,
    pub requires_budget_and_permission: bool,
}

/// The invalidation report (§16:314).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InvalidationReport {
    pub stale: Vec<StaleMark>,
    pub reevaluation: Vec<ReevaluationEntry>,
}

/// Portfolio queue errors (§16:314 + roadmap: restarting research still
/// requires available budget).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueueError {
    BudgetUnavailable,
    MissingSnapshot,
}

/// A reactivated failure version (§16:316): new version referencing the
/// ORIGINAL failure + the changed-condition evidence. The failure is
/// never erased.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reactivation {
    pub new_version_id: String,
    pub original_failure_id: String,
    pub changed_condition_evidence: String,
}

/// Version-bound lineage for a transition (R-097): every state change
/// records the chain IDs + the snapshot it binds (§16:318).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TransitionRecord {
    pub entity_id: String,
    pub from_state: String,
    pub to_state: String,
    pub snapshot_id: String,
    pub lineage_mission: String,
    pub lineage_hypothesis_version: String,
}
