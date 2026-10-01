//! Evaluation-suite records (T-026, R-074/R-075).

use serde::{Deserialize, Serialize};

/// The four baseline arms (IMPLEMENTATION_PLAN:90). Model arms are typed
/// and envelope-enforced here; their EXECUTION needs provider
/// integrations and is out of scope — no simulated outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArmKind {
    /// Model with active retrieval + identical tools.
    ActiveRetrievalModel,
    /// Bounded generate/review harness.
    FixedGenerateReview,
    /// Structured search.
    SimpleSearch,
    /// A named existing domain method.
    DomainMethod,
}

/// The per-arm envelope (campaign: all arms get the same approved
/// compute/retrieval/experiment envelope; tuning/engineering time
/// recorded).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    /// Compute budget in a comparable unit.
    pub compute_budget: u64,
    /// Tool access list (must match across arms).
    pub tool_access: Vec<String>,
    /// Model access level (must match across arms).
    pub model_access: String,
    /// Recorded tuning/engineering allowance.
    pub tuning_record: String,
}

/// A defined arm: kind + id + envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineArm {
    pub id: String,
    pub kind: ArmKind,
    pub envelope: Envelope,
}

/// Named mismatch (R-074): which arm, which resource, what differs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mismatch {
    pub arm_id: String,
    pub resource: String,
    pub detail: String,
}

/// The mismatch report from `check_matched`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MismatchReport {
    pub mismatches: Vec<Mismatch>,
}

/// Ablation kinds (roadmap: graph, review, decomposition,
/// optional-decision-model).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AblationKind {
    RemoveGraph,
    RemoveReview,
    RemoveDecomposition,
    RemoveOptionalDecisionModel,
}

/// A single-ingredient ablation (campaign: removes the candidate's
/// claimed causal ingredient — ONE per ablation).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AblationSpec {
    pub of_arm_id: String,
    pub kind: AblationKind,
}

/// Arm results with the FOUR R-075 fields kept SEPARATE — never
/// collapsed into one number. None = not measured (honest absence).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ArmResult {
    pub judge_score: Option<String>,
    pub rediscovery_status: Option<String>,
    pub novelty_status: Option<String>,
    pub replication_status: Option<String>,
}

/// Arm registration errors (R-024 pattern: named).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArmError {
    DuplicateArmId,
    ZeroBudget,
}
