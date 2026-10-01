//! Evaluation records (T-022, MASTER_SPEC §14:280-282).

use serde::{Deserialize, Serialize};

/// Execution validity (§14:280): valid, invalid, incomplete — preserved,
/// never collapsed into the science dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionValidity {
    Valid,
    Invalid,
    Incomplete,
}

/// Scientific conclusion (§14:280). NO "proven true" variant exists:
/// the strongest state is `Supported` under a scope and conditions
/// (§14:282 — no run may report proven true from a finite test).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScientificConclusion {
    Supported,
    Contradicted,
    Inconclusive,
    NotAssessed,
}

/// Engineering target (§14:280): separate from the science dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineeringTarget {
    Met,
    NotMet,
    Inconclusive,
    NotAssessed,
}

/// Threshold provenance (R-026): where the threshold, estimand, units,
/// and comparator were declared.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThresholdProvenance {
    pub estimand: String,
    pub units: String,
    pub comparator: String,
    pub threshold_source: String,
}

/// Measured inputs to interpretation. The interval arrives COMPUTED by
/// the qualified method — this module interprets, it does not fit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Measurements {
    /// Confidence/uncertainty interval from the qualified analysis.
    pub interval: Option<(f64, f64)>,
    /// Claimed-benefit threshold theta.
    pub threshold: f64,
    /// Allowed quality loss m for noninferiority.
    pub quality_margin: Option<f64>,
    /// Lower bound of the quality difference (noninferiority input).
    pub quality_lower_bound: Option<f64>,
}

/// The four guardrail classes R-104 requires before qualification
/// (scoped error, correctness, precision/power, noninferiority).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardrailClass {
    ScopedError,
    Correctness,
    PrecisionPower,
    Noninferiority,
}

/// A named per-class gap: which required guardrail class was not
/// declared before qualification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardrailGap {
    pub missing: GuardrailClass,
}

/// One independent guardrail outcome (roadmap: "Check guardrails
/// independently").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuardrailOutcome {
    pub name: String,
    pub value: f64,
    pub limit: f64,
    pub ok: bool,
}

/// The typed result (§14:280): all three dimensions, the claim and
/// conditions each conclusion applies to, and R-026 provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypedResult {
    pub execution: ExecutionValidity,
    pub science: ScientificConclusion,
    pub engineering: EngineeringTarget,
    /// The claim this result applies to.
    pub claim: String,
    /// The conditions (regime, dataset, environment) it applies under.
    pub conditions: String,
    pub provenance: ThresholdProvenance,
    /// Invalid-execution reason, when execution is invalid.
    pub invalid_reason: Option<String>,
}

/// Independent guardrail report: outcomes recorded separately; a failed
/// guardrail does NOT flip the science/engineering dimensions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuardrailReport {
    pub outcomes: Vec<GuardrailOutcome>,
    pub all_pass: bool,
}
