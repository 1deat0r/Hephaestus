//! Release records (T-028, R-077/R-078/R-080, campaign dispositions).

use serde::{Deserialize, Serialize};

/// Severity levels for findings (R-077 gate input).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    Major,
    Minor,
}

/// A security finding or engineering failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub severity: Severity,
    pub resolved: bool,
    /// Whether it falls inside the declared release scope.
    pub in_scope: bool,
    pub description: String,
}

/// Scope labels (campaign release dispositions).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeLabel {
    /// Default: shipped as EXPERIMENTAL unless evidence supports more.
    Experimental,
    QualifiedForDeclaredScope,
}

/// The five qualification inputs for QUALIFIED_FOR_DECLARED_SCOPE
/// (campaign: frozen campaign + protected oracle/method qualification +
/// error/correctness guardrails + independent reproduction + scope-
/// specific security gates).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct QualificationInputs {
    pub frozen_campaign: bool,
    pub oracle_method_qualified: bool,
    pub guardrails_passing: bool,
    pub independent_reproduction: bool,
    pub security_gates_passing: bool,
}

/// Scope label from evidence (campaign dispositions): EXPERIMENTAL
/// unless ALL five qualification inputs hold.
pub fn scope_label(inputs: &QualificationInputs) -> ScopeLabel {
    if inputs.frozen_campaign
        && inputs.oracle_method_qualified
        && inputs.guardrails_passing
        && inputs.independent_reproduction
        && inputs.security_gates_passing
    {
        ScopeLabel::QualifiedForDeclaredScope
    } else {
        ScopeLabel::Experimental
    }
}

/// Finite-suite uncertainty (R-078): measured-on facts only. There is NO
/// universal-reliability field — the struct cannot represent one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Uncertainty {
    pub suite_id: String,
    pub n: usize,
    /// Interval from the qualified analysis.
    pub interval: (f64, f64),
}

/// The five outcome classes (R-080) — all present, possibly zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OutcomeCounts {
    pub positive: u64,
    pub negative: u64,
    pub inconclusive: u64,
    pub invalid: u64,
    pub blocked: u64,
}

/// Quantity-only measurement (R-103 continuity): no invented conversion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Measurement {
    pub kind: String,
    pub quantity: String,
    pub unit: String,
}

/// Reproducibility report (link to T-023 semantics).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReproducibilityReport {
    pub environment_pins: Vec<(String, String)>,
    pub repro_commands: Vec<String>,
    /// Per-artifact digest verification results.
    pub artifact_verifications: Vec<(String, bool)>,
}

/// Named release blocks (R-077 and friends).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseBlock {
    /// Critical unresolved failure inside the declared scope.
    CriticalUnresolvedInScope(String),
    /// Uncertainty must be present (R-078).
    MissingUncertainty,
    /// Reproducibility report must be present.
    MissingReproducibility,
    /// Cost/latency entries must be quantities (no priced conversions).
    PricedMeasurement,
}

/// The release packet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleasePacket {
    pub label: ScopeLabel,
    pub declared_scope: String,
    pub findings: Vec<Finding>,
    pub uncertainty: Uncertainty,
    pub outcome_counts: OutcomeCounts,
    pub cost_latency: Vec<Measurement>,
    pub reproducibility: ReproducibilityReport,
    /// Unresolved research questions — explicitly recorded (honesty;
    /// may be empty but is present).
    pub unresolved_questions: Vec<String>,
}
