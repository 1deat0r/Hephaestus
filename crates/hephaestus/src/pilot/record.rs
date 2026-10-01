//! Pilot-campaign records (T-027, R-103, campaign pilot paragraphs).

use serde::{Deserialize, Serialize};

/// The three partitions, separated BY REPOSITORY (campaign).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Partition {
    TrainTune,
    Pilot,
    Confirmatory,
}

/// Stratification axes (campaign: size and dependency topology).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Stratification {
    pub size_bands: Vec<String>,
    pub topology_classes: Vec<String>,
}

/// A repository assignment: exactly one partition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryAssignment {
    pub repo_id: String,
    pub partition: Partition,
    /// Change-episode ids sampled BEFORE confirmation (R-103).
    pub episode_ids: Vec<String>,
    /// Stratification band labels.
    pub size_band: String,
    pub topology_class: String,
}

/// The pilot plan (R-103): preregistration-bound, stratified, partitioned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PilotPlan {
    /// Frozen preregistration id from the methods registry (T-021).
    pub preregistration_id: String,
    pub stratification: Stratification,
    pub assignments: Vec<RepositoryAssignment>,
    /// Batch size: FREE — twenty is a possible debugging batch, not a
    /// mandatory evidence threshold (campaign).
    pub batch_size: usize,
    /// Debug-vs-evidence intent, recorded.
    pub intent: String,
    /// Budget reserved for the batch.
    pub budget: u64,
}

/// Named plan errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanError {
    /// A repository assigned to two partitions.
    PartitionConflict(String),
    /// No frozen preregistration behind the id.
    UnregisteredPreregistration,
    /// Episodes must be sampled before confirmation (R-103).
    EpisodesNotPreSampled,
    /// Empty batch measures nothing.
    EmptyBatch,
}

/// One pilot mission outcome (R-103 denominators: failures retained).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionOutcome {
    pub repo_id: String,
    pub arm_id: String,
    /// Measured value (e.g. latency delta); None for failed/blocked.
    pub value: Option<f64>,
    pub failed_or_blocked: bool,
}

/// The variance estimate (campaign: pilot estimates variance for the
/// larger study).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct VarianceEstimate {
    /// Per-arm mean + population variance.
    pub per_arm: Vec<(String, Option<f64>, Option<f64>)>,
    /// Paired-difference variance across repositories.
    pub paired_difference_variance: Option<f64>,
    /// Failure/blocked count — retained in the denominator (R-103).
    pub failures_retained: usize,
    pub total_missions: usize,
}

/// Named confirmation blocks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfirmationBlock {
    MissingVarianceEstimate,
    UnresolvedOracle,
    /// Pilot outcomes drawn from the confirmatory partition.
    PartitionContamination,
    UnqualifiedAnalysis,
}
