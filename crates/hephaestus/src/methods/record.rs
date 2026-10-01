//! Method-registry records (T-021, R-100/R-103, campaign M3 slice).

use serde::{Deserialize, Serialize};

/// What is estimated, on which units, with which denominators and cost
/// accounting (R-100, R-103, campaign metrics table).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Estimand {
    /// What is being estimated (e.g. "paired latency-savings difference").
    pub quantity: String,
    /// The unit of analysis (e.g. "repository-level").
    pub unit: String,
    /// Denominator rule: which missions/cases count, including failures.
    pub denominator: String,
    /// Cost accounting: acquisition, generation, failed/invalid/blocked
    /// runs, evaluator and reproduction work, human intervention.
    pub cost_fields: Vec<String>,
}

/// Missingness policy (campaign M3 paragraph): failed or missing runs
/// remain failures for useful-outcome yield; missing timing data cannot
/// become a favorable latency estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingnessPolicy {
    /// Failed/missing runs count as failures (yield metrics).
    FailedOrMissingIsFailure,
    /// Missing timing is treated as UNfavorable, never favorable.
    TimingMissingIsUnfavorable,
    /// Exclusion only where the qualified method allows; reason recorded.
    ExcludeWithReason,
}

/// Clipping/bounding policy, FROZEN before confirmation (campaign:
/// "Bounds/clipping are frozen before confirmation and clipped rates are
/// reported").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClippingPolicy {
    /// Bounds on outcome values, frozen pre-confirmation.
    pub lower: Option<f64>,
    pub upper: Option<f64>,
    /// Clipped rates are reported, not hidden.
    pub report_clipped_rates: bool,
}

/// A registered statistical method (R-100): immutable once qualified; a
/// change is a new version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodSpec {
    pub name: String,
    pub version: String,
    pub estimand: Estimand,
    /// Independence conditions, distribution assumptions, etc.
    pub assumptions: Vec<String>,
    /// Fixed-family error allocation (see `bonferroni_alpha`).
    pub alpha_allocations: Vec<f64>,
    pub missingness: MissingnessPolicy,
    pub clipping: ClippingPolicy,
    /// Implementation identity digest (protected registration).
    pub implementation_digest: String,
}

/// Registry errors (R-024 pattern: named, retained).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistryError {
    /// (name, version) already registered - immutability.
    DuplicateMethodVersion,
    /// Malformed allocation: negative, or summing above the family alpha.
    InvalidAllocation,
    /// Empty identity digest: registration without provenance.
    MissingImplementationIdentity,
}

/// The protected method registry: append-only, immutable entries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MethodRegistry {
    entries: Vec<MethodSpec>,
}

/// Fixed Bonferroni error allocation (campaign: "alpha allocations from a
/// fixed Bonferroni family"): split `alpha_total` equally across `k`
/// family members. Returns k equal shares.
pub fn bonferroni_alpha(k: usize, alpha_total: f64) -> Vec<f64> {
    let share = alpha_total / k as f64;
    vec![share; k]
}

/// Zero-failure upper bound under IID Bernoulli with zero observed
/// failures: `1 - alpha^(1/n)`.
///
/// PLANNING CALCULATION ONLY: not a prescribed sample size for
/// heterogeneous cases, not a universal safety claim (campaign's own
/// qualification). The campaign's example: n=299, alpha=0.05 gives
/// <= 0.01.
pub fn zero_failure_bound(n: usize, alpha: f64) -> f64 {
    1.0 - alpha.powf(1.0 / n as f64)
}

impl MethodRegistry {
    /// Register a method immutably (R-100). Duplicate (name, version)
    /// refused; any change is a new version.
    pub fn register(&mut self, spec: MethodSpec) -> Result<(), RegistryError> {
        if spec.implementation_digest.is_empty() {
            return Err(RegistryError::MissingImplementationIdentity);
        }
        if spec
            .alpha_allocations
            .iter()
            .any(|a| !a.is_finite() || *a < 0.0)
        {
            return Err(RegistryError::InvalidAllocation);
        }
        if self
            .entries
            .iter()
            .any(|e| e.name == spec.name && e.version == spec.version)
        {
            return Err(RegistryError::DuplicateMethodVersion);
        }
        self.entries.push(spec);
        Ok(())
    }

    /// Look up a qualified method by name+version.
    pub fn qualified(&self, name: &str, version: &str) -> Option<&MethodSpec> {
        self.entries
            .iter()
            .find(|e| e.name == name && e.version == version)
    }
}
