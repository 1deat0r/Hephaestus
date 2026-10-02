//! Dossier records (T-023, MASTER_SPEC §15:299-300, R-044).

use serde::{Deserialize, Serialize};

/// Pinned environment (R-065 continuity): exact versions recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Environment {
    pub os: String,
    pub tool_versions: Vec<(String, String)>,
    /// Artifact digests the run produced/consumed.
    pub artifact_digests: Vec<String>,
}

/// One cost receipt as a QUANTITY (R-103): no invented conversion price.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostReceipt {
    /// acquisition | generation | failed_invalid_blocked_runs |
    /// evaluator_and_reproduction | human_intervention
    pub kind: String,
    /// Resource/human-time quantity + unit (e.g. "3.5", "CPU-hours").
    pub quantity: String,
    pub unit: String,
    /// Provider charges where they exist (money recorded as charged,
    /// never converted into resource units).
    pub provider_charge: Option<String>,
}

/// A deviation from the frozen plan, retained (§14:282: deviations
/// preserved).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deviation {
    pub from_plan_field: String,
    pub what_changed: String,
    pub reason: String,
}

/// A failed attempt, retained (R-044: failure history present).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureEntry {
    pub stage: String,
    pub description: String,
    pub artifact_digest: String,
}

/// Raw, unsummarized run capture (§15:299 "raw data").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RunRecord {
    pub measurements: Vec<(String, f64)>,
    pub environment: Environment,
    pub deviations: Vec<Deviation>,
    pub cost_receipts: Vec<CostReceipt>,
    pub failure_history: Vec<FailureEntry>,
}

/// Reproduction outcomes (§15:298 clean-environment reproduction;
/// R-080: every class is first-class).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReproductionOutcome {
    /// Same environment, same artifact digests, agreeing measurements.
    Reproduced,
    /// Pinned versions differ — typed mismatch, never a silent pass.
    EnvironmentMismatch,
    /// Same environment but measurements disagree beyond declared
    /// precision.
    Disagrees,
}

/// §15:300: a negative result distinguishes these five.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NegativeResultKind {
    InvalidTest,
    FailedImplementation,
    UnsupportedMechanism,
    UncompetitiveEngineeringRealization,
    ResourceLimitedInvestigation,
}

/// Version-bound lineage (R-097): mission -> opportunity -> mechanism ->
/// hypothesis -> plan -> result IDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Lineage {
    pub mission_id: String,
    pub opportunity_id: String,
    pub mechanism_id: String,
    pub hypothesis_version: String,
    pub plan_id: String,
    pub result_id: String,
}

/// The exported dossier (§15:299 field list).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dossier {
    pub problem_and_beneficiary: String,
    /// Reference into the prior-art report (T-018 inputs).
    pub prior_art_reference: String,
    pub mechanism_explanation: String,
    /// Complete hypothesis versions, earliest first.
    pub hypothesis_versions: Vec<String>,
    /// Predeclared tests: the frozen plan summary.
    pub predeclared_tests: String,
    pub environment: Environment,
    /// Raw data, verbatim from the run records.
    pub raw_data: Vec<(String, f64)>,
    pub analysis: String,
    /// Typed results (T-022) carried verbatim as JSON.
    pub results_json: String,
    pub counterevidence: String,
    /// Failure history and deviations: SEPARATE conclusions (R-044).
    pub failure_history: Vec<FailureEntry>,
    pub deviations: Vec<Deviation>,
    pub uncertainty: String,
    pub scope_limits: String,
    pub cost_ledger: Vec<CostReceipt>,
    pub repro_commands: Vec<String>,
    pub unresolved_risks: String,
    pub next_justified_action: String,
    /// For negative dossiers: which of the five kinds (§15:300).
    pub negative_kind: Option<NegativeResultKind>,
    pub lineage: Lineage,
    /// Required evidence label (R-082): synthetic/illustrative content
    /// can never be rendered as real experimental evidence.
    pub evidence_label: EvidenceLabel,
}

/// Why a dossier's evidence is unmeasured (R-082/AT-082): the two
/// §28 illustrative sources — a packaged synthetic fixture or an
/// illustrative worked example.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnmeasuredReason {
    SyntheticFixture,
    IllustrativeExample,
}

/// The dossier's required evidence label (R-082): `Measured` must
/// carry its version-bound receipt; `Unmeasured` carries why the
/// content is not real experimental evidence. There is no unlabeled
/// dossier — the field is not optional.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceLabel {
    Measured {
        run_receipt: String,
    },
    Unmeasured {
        reason: UnmeasuredReason,
        source: String,
    },
}

/// One record in a classified bundle export (R-092/AT-092): status
/// and reproduction are TYPED fields — structurally always present —
/// while evidence and scope are enforced non-empty by the gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassifiedRecord {
    pub record_id: String,
    pub status: crate::lifecycle::HypothesisState,
    pub evidence_ids: Vec<String>,
    pub scope: String,
    pub reproduction: ReproductionOutcome,
}

/// Why a bundle export refused (typed-rejection convention).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleExportError {
    /// A record carries no evidence ids.
    MissingEvidence { record_id: String },
    /// A record carries no scope.
    MissingScope { record_id: String },
    /// The bundle could not be serialized (never mislabeled as a
    /// classification problem).
    Serialization,
}
