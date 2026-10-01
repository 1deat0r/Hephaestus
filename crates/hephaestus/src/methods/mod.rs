//! Fixed-sample method family + protected method registry (T-021,
//! campaign M3 slice, R-100/R-103).
//!
//! Registration, allocation, and preregistration gates for the
//! fixed-sample repository-level methods. NO live-data statistics here
//! (that is the evaluator path, T-022); NO fabricated sample sizes - the
//! campaign package deliberately supplies none and none is invented.
//! Sequential methods are a later, separately qualified extension.

pub mod record;

pub use record::{
    ClippingPolicy, Estimand, MethodRegistry, MethodSpec, MissingnessPolicy, RegistryError,
    bonferroni_alpha, zero_failure_bound,
};

use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// The shared protected registry handle: immutable entries behind a
/// mutex (append-only; entries never rewritten).
#[derive(Default)]
pub struct ProtectedRegistry {
    inner: Mutex<MethodRegistry>,
}

impl ProtectedRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, spec: MethodSpec) -> Result<(), RegistryError> {
        self.inner.lock().expect("registry lock").register(spec)
    }

    pub fn qualified(&self, name: &str, version: &str) -> Option<MethodSpec> {
        self.inner
            .lock()
            .expect("registry lock")
            .qualified(name, version)
            .cloned()
    }
}

/// The runtime campaign manifest to gate into confirmation (campaign:
/// "No runtime manifest with `n=null`, an unqualified method, or an
/// unresolved oracle can enter confirmation").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CampaignManifest {
    /// Sample size; None is refused (no n=null into confirmation).
    pub n: Option<usize>,
    /// Name+version of the qualified method to use.
    pub method_name: Option<String>,
    pub method_version: Option<String>,
    /// Oracle status: false = unresolved = refused.
    pub oracle_resolved: bool,
    /// The n-calculation reference (campaign MUST-include list).
    pub n_calculation_reference: Option<String>,
    /// Alpha allocation for the analysis family.
    pub alpha_allocation: Option<f64>,
    /// Reference distribution (campaign MUST-include list).
    pub reference_distribution: Option<String>,
    /// Power/precision target (campaign MUST-include list).
    pub power_precision_target: Option<String>,
    /// Sensitivity assumptions (campaign MUST-include list).
    pub sensitivity_assumptions: Option<String>,
}

/// Named preregistration refusals (campaign MUST-gate list).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreregError {
    NullSampleSize,
    UnqualifiedMethod,
    UnresolvedOracle,
    MissingNCalculation,
    MissingAlphaAllocation,
    MissingReferenceDistribution,
    MissingPowerPrecisionTarget,
    MissingSensitivityAssumptions,
}

/// A frozen preregistration: the manifest as it entered confirmation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preregistration {
    pub n: usize,
    pub method: MethodSpec,
    pub alpha_allocation: f64,
    pub manifest: CampaignManifest,
}

/// Gate a manifest into confirmation (campaign M3 slice): every refusal
/// named; a passing manifest is frozen immutably.
pub fn preregister(
    manifest: &CampaignManifest,
    registry: &ProtectedRegistry,
) -> Result<Preregistration, PreregError> {
    let n = manifest.n.ok_or(PreregError::NullSampleSize)?;
    if !manifest.oracle_resolved {
        return Err(PreregError::UnresolvedOracle);
    }
    let (name, version) = match (&manifest.method_name, &manifest.method_version) {
        (Some(nm), Some(v)) => (nm.clone(), v.clone()),
        _ => return Err(PreregError::UnqualifiedMethod),
    };
    let method = registry
        .qualified(&name, &version)
        .ok_or(PreregError::UnqualifiedMethod)?;
    let alpha = manifest
        .alpha_allocation
        .ok_or(PreregError::MissingAlphaAllocation)?;
    if manifest.n_calculation_reference.is_none() {
        return Err(PreregError::MissingNCalculation);
    }
    if manifest.reference_distribution.is_none() {
        return Err(PreregError::MissingReferenceDistribution);
    }
    if manifest.power_precision_target.is_none() {
        return Err(PreregError::MissingPowerPrecisionTarget);
    }
    if manifest.sensitivity_assumptions.is_none() {
        return Err(PreregError::MissingSensitivityAssumptions);
    }
    Ok(Preregistration {
        n,
        method,
        alpha_allocation: alpha,
        manifest: manifest.clone(),
    })
}
