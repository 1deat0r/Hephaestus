//! Advisory records (T-029, R-005/R-090).

use serde::{Deserialize, Serialize};

/// What the provider recommends for a bounded advisory question.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recommendation {
    Prioritize(String),
    Deprioritize(String),
    /// The provider declines to advise — a first-class outcome.
    Abstain,
}

/// Confidence as a MODEL JUDGMENT (§16 continuity: uncalibrated model
/// probabilities are stored as model judgments, never scientific
/// posteriors — R-048: uncalibrated model confidence is judgment, not
/// posterior belief). The label travels with the number forever.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ModelJudgment {
    pub value: f64,
    pub bucket: ConfidenceBucket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConfidenceBucket {
    Low,
    Medium,
    High,
}

/// An advisory decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdvisoryDecision {
    pub decision_id: String,
    pub recommendation: Recommendation,
    pub confidence: ModelJudgment,
    /// Reference to the rationale record (not inlined).
    pub rationale_ref: String,
}

/// Provider identity (R-090): official Jev vs third-party DISTINCT.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderIdentity {
    pub name: String,
    pub digest: String,
    pub official_jev: bool,
}

/// A fixture case with a KNOWN outcome (calibration is computed only
/// over these).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnownOutcomeCase {
    pub case_id: String,
    /// Whether prioritizing this case was actually correct.
    pub prioritize_was_correct: bool,
}

/// A cost receipt as a quantity (R-103 continuity).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostQuantity {
    pub quantity: String,
    pub unit: String,
}

/// Calibration bucket: predicted-confidence band vs observed correct
/// rate, computed ONLY over known-outcome cases.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CalibrationBucket {
    pub band: ConfidenceBucket,
    pub predicted_mean: f64,
    pub observed_correct_rate: f64,
    pub n: usize,
}

/// The provider report. `NotMeasured` = honest absence (R-078 pattern:
/// no invented numbers).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ProviderReport {
    pub identity: Option<ProviderIdentity>,
    pub calibration: Vec<CalibrationBucket>,
    /// Fraction of cases where the provider abstained.
    pub abstention_rate: Option<f64>,
    /// Cases the provider rejected (abstained) that were actually
    /// resolvable — rejection false negatives.
    pub rejection_false_negatives: usize,
    pub cost: Vec<CostQuantity>,
    pub end_to_end_quality: Option<f64>,
}

/// Control-plane authority attestation (R-005): advisory input cannot
/// change gates, mint budget, or qualify methods.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlAttestation {
    pub advisory_cannot_change_gates: bool,
    pub advisory_cannot_mint_budget: bool,
    pub advisory_cannot_qualify_methods: bool,
    pub core_runs_without_provider: bool,
}
