//! Experiment-plan records (T-019, MASTER_SPEC §13:255).
//!
//! Every field the spec binds, frozen at compile time: hypothesis
//! version, claim IDs, conditions, comparator, intervention artifact,
//! measurement procedure, units, primary endpoints, guardrails, sampling
//! unit, analysis specification, stopping rule, evaluator hash, resource
//! limit, authorization scope.

use serde::{Deserialize, Serialize};

/// Precise blockers (§13:262): compile returns these instead of imagined
/// outcomes. Never a substituted LLM guess.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Blocker {
    MissingInstrument,
    InaccessibleDataset,
    UnvalidatedSimulator,
    InadequatePower,
    ForbiddenAction,
    ResourceOverrun,
}

/// Predeclared analysis specification (R-040): method + decision rule,
/// fixed before confirmatory results are accessed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisSpec {
    pub method: String,
    pub decision_rule: String,
}

/// Valid stopping rule (R-040): when the experiment halts, predeclared.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoppingRule {
    /// Stop after N complete samples.
    FixedSampleCount(usize),
    /// Stop when the measurement window elapses.
    TimeBudgetHours(u64),
    /// Stop when the resource limit is reached.
    ResourceBudgetExhausted,
}

/// Control structure (§13:259).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Controls {
    /// Positive / known-working control, where appropriate.
    pub positive: Option<String>,
    /// Negative control (expected null).
    pub negative: Option<String>,
    pub randomization: bool,
    pub repeatability_checks: bool,
    pub environmental_capture: bool,
    /// Plan for missing or corrupted observations.
    pub missing_observation_plan: Option<String>,
}

/// The immutably-registered experiment plan (§13:255).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExperimentPlan {
    /// Frozen hypothesis version (R-097) - a version string, never a
    /// mutable reference.
    pub hypothesis_version: String,
    pub claim_ids: Vec<String>,
    pub operating_conditions: String,
    pub comparator: String,
    pub intervention_artifact: String,
    pub measurement_procedure: String,
    pub units: String,
    pub primary_endpoints: Vec<String>,
    pub guardrails: Vec<String>,
    pub sampling_unit: String,
    pub analysis: AnalysisSpec,
    pub stopping_rule: StoppingRule,
    /// SHA-256 hex of evaluator identity, obtained from the protected
    /// context (R-094/R-095), never from candidate artifacts.
    pub evaluator_digest: String,
    pub resource_limit: String,
    pub authorization_scope: String,
    pub controls: Controls,
}

/// Named completeness denials (R-096/AT-096): each gap is named.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompletenessDenial {
    MissingPrimaryEndpoint,
    MissingControl,
    MissingSamplingUnit,
    MissingGuardrail,
}

/// Validate verdict: complete or the specific denials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Complete,
    Incomplete(Vec<CompletenessDenial>),
}

/// Evaluator digest mismatch: the plan's digest disagrees with the
/// protected context's digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DigestMismatch {
    pub plan_digest: String,
    pub protected_digest: String,
}
