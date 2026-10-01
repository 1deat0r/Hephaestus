//! Acceleration records (T-031, campaign ablation + exit rules).

use serde::{Deserialize, Serialize};

/// The six candidate acceleration mechanisms (IMPLEMENTATION_PLAN:104),
/// each independently evaluable and disablable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MechanismKind {
    GraphRetrieval,
    VectorSearch,
    GpuWorkers,
    MonteCarloTreeSearch,
    QualityDiversitySearch,
    LearnedAllocation,
}

/// A candidate mechanism with its envelope (matched-budget continuity)
/// and the mandatory disablability flag (campaign exit: the core path
/// must stay functional).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateMechanism {
    pub kind: MechanismKind,
    pub id: String,
    pub disablable: bool,
    pub compute_budget: u64,
    pub tool_access: Vec<String>,
    /// Infrastructure present? Absent -> honestly NotEvaluable.
    pub infrastructure_available: bool,
}

/// Envelope mismatch: a pair whose arms differ is not a matched ablation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvelopeMismatch {
    pub field: String,
    pub detail: String,
}

/// A matched ablation pair: identical envelopes, the mechanism is the
/// ONLY difference (campaign: removes exactly the claimed causal
/// ingredient).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AblationPair {
    pub mechanism: CandidateMechanism,
    /// The measured metric the promotion claim is about.
    pub metric: String,
}

/// Recorded pair measurement — values are RECORDED, never generated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PairMeasurement {
    pub with_value: f64,
    pub without_value: f64,
    /// Number of quality-guardrail violations observed with the
    /// mechanism enabled.
    pub guardrail_violations: usize,
}

/// The promotion verdict (roadmap: promote only when matched ablations
/// demonstrate value within quality guardrails).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionVerdict {
    /// Measured improvement + guardrails pass + mechanism disablable.
    Promoted,
    /// No measured improvement (or guardrail violations): incumbent kept.
    Retained,
    /// Infrastructure absent: honestly not evaluated.
    NotEvaluable,
}
