//! Mission record (T-012): the versioned Intent-to-Mission output.
//!
//! The schema mirrors MASTER_SPEC section 4 exactly: intended beneficiary,
//! objective, domain boundaries, practical constraints, resource envelope,
//! permitted tools and destinations, forbidden actions, confidentiality,
//! success metrics, evidence standard, and stop conditions — plus versioning,
//! provenance, explicit assumptions, and authorization requests.
//!
//! Deliberate absence: there is **no hypothesis field** (R-001 — a mission
//! created from a goal must not require a supplied hypothesis). A
//! caller-supplied hypothesis survives only as opaque provenance text, never
//! as a validated claim (validation is T-016's job).

use serde::{Deserialize, Serialize};

use crate::contracts::generated::Money;

/// Autonomy profile: permission scope, not an intelligence setting
/// (MASTER_SPEC section 4). Wider profiles add tools, never remove
/// prohibitions: no profile permits physical actuation or public disclosure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutonomyProfile {
    /// Planning and read-only reasoning only; nothing executable permitted.
    PlanOnly,
    /// Adds already-authorized network research services.
    LocalResearch,
    /// Adds bounded code execution.
    SandboxExperiments,
    /// Adds supervised external actions. Still never actuation/disclosure.
    SupervisedExternal,
}

/// Resource envelope: the mission's budget boundary in T-008 minor-unit
/// money. Unknown prices are explicit, never zero-filled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceEnvelope {
    pub limit: Money,
    /// True when the mission's cost basis is not yet priced: enforcement
    /// must treat the envelope as unplannable, not as zero.
    pub pricing_unknown: bool,
}

/// Caller-side resource posture accompanying a goal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceProfile {
    pub envelope: ResourceEnvelope,
    /// Whether the owner already approved spending inside the envelope.
    /// False turns any priced work into an authorization request (R-011).
    pub spending_approved: bool,
}

/// What the owner submits: a broad goal plus the value frame the compiler
/// must not invent (MASTER_SPEC section 4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intake {
    pub goal: String,
    pub beneficiary: String,
    /// Owner's approved standing priorities. Empty means the compiler has
    /// no value frame and must fail closed (MissingValueFrame).
    pub standing_priorities: Vec<String>,
    pub resource: ResourceProfile,
    pub requested_profile: AutonomyProfile,
    /// Opaque caller-supplied hypothesis, if any: recorded as provenance,
    /// never validated here.
    pub caller_hypothesis: Option<String>,
}

/// Reversible choices the compiler resolved on its own (MASTER_SPEC
/// section 4): terminology, initial search vocabulary, initial corpus, or a
/// tentative subdomain. Each carries provenance and is overridable via
/// `revise`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssumptionKind {
    Terminology,
    SearchVocabulary,
    InitialCorpus,
    TentativeSubdomain,
    OtherReversible,
}

/// One explicit reversible default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assumption {
    pub kind: AssumptionKind,
    pub statement: String,
    pub provenance: String,
}

/// Why compilation yielded requests instead of a Mission (R-011).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthReason {
    /// Priced work without approved spending.
    UnapprovedSpending,
    /// Goal needs external disclosure.
    ExternalDisclosure,
    /// Goal needs an irreversible operation.
    IrreversibleOperation,
    /// Risk is materially ambiguous.
    AmbiguousRisk,
    /// No standing priorities / resource profile: no value frame.
    MissingValueFrame,
    /// Sensitive application chosen by nobody: must not proceed silently.
    SensitiveApplication,
    /// Goal requires physical actuation: no profile grants it.
    ActuationRefused,
}

/// One authorization the owner must grant before a Mission can exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationRequest {
    pub reason: AuthReason,
    pub detail: String,
}

/// Guardrails every compiled Mission carries (MASTER_SPEC section 4):
/// correctness, workload scope, privacy, resource use, unacceptable
/// trade-offs. The objective states intent; these constrain its pursuit.
pub fn default_guardrails() -> Vec<String> {
    vec![
        "correctness: results must be checkable against recorded evidence".to_string(),
        "workload scope: only the stated objective; no adjacent optimization".to_string(),
        "privacy: no confidential material outside the approved boundary".to_string(),
        "resource use: stay inside the resource envelope".to_string(),
        "trade-offs: never sacrifice quality, safety, or scope to move a metric".to_string(),
    ]
}

/// Forbidden actions no profile ever lifts (MASTER_SPEC section 4).
pub fn forbidden_actions() -> Vec<String> {
    vec![
        "physical actuation".to_string(),
        "public disclosure".to_string(),
    ]
}

/// Versioned Mission: the compiler's output record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mission {
    pub beneficiary: String,
    pub objective: String,
    pub domain_boundaries: Vec<String>,
    pub constraints: Vec<String>,
    pub envelope: ResourceEnvelope,
    pub permitted_tools: Vec<String>,
    pub permitted_destinations: Vec<String>,
    pub forbidden_actions: Vec<String>,
    pub confidentiality: String,
    pub success_metrics: Vec<String>,
    pub guardrails: Vec<String>,
    pub evidence_standard: String,
    pub stop_conditions: Vec<String>,
    pub assumptions: Vec<Assumption>,
    /// Provenance of a caller-supplied hypothesis, if one arrived with the
    /// intake. The hypothesis text itself is not stored: it is unvalidated
    /// here and keeping it would invite downstream misuse as a claim.
    pub caller_hypothesis_provenance: Option<String>,
    pub profile: AutonomyProfile,
    /// Monotonic version; 1 for a fresh compile (R-012).
    pub version: u64,
    /// Previous version this one supersedes, if any.
    pub supersedes: Option<u64>,
}

/// Compiler output: a Mission, or the authorizations that must come first.
/// Never a guessed Mission for an authorization-grade goal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Compiled {
    Mission(Box<Mission>),
    NeedsAuthorization(Vec<AuthorizationRequest>),
}

/// Structural intake defects (as opposed to authorization requests, which
/// are semantic and returned as data).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    EmptyGoal,
    EmptyBeneficiary,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyGoal => write!(f, "goal must not be empty"),
            Self::EmptyBeneficiary => write!(f, "beneficiary must not be empty"),
        }
    }
}

impl std::error::Error for CompileError {}

/// A mission change the owner requests: any of these mints a new version
/// (R-012). Plan invalidation downstream is the caller's job; the
/// `ImpactReport` names exactly the sections that moved.
#[derive(Debug, Clone, PartialEq)]
pub enum MissionChange {
    /// New goal text: objective and derived success metrics move.
    Goal(String),
    /// New cost limit: only the envelope moves.
    CostLimit(Money),
    /// New dataset policy: recorded as a constraint.
    DatasetPolicy(String),
    /// New quality margin: recorded as a constraint.
    QualityMargin(String),
    /// Replace one inferred requirement (matched verbatim in constraints,
    /// then guardrails) with owner text, leaving a provenance assumption.
    OverrideRequirement {
        requirement: String,
        replacement: String,
    },
}

/// Which mission sections a `revise` moved. Callers map these to the
/// dependent plans they invalidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactReport {
    pub changed_sections: Vec<String>,
}

/// Why a `revise` was refused. Unknown requirements error rather than
/// guessing (fail-closed override path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviseError {
    UnknownRequirement(String),
    EmptyValue(String),
    /// The revised goal trips authorization screens: the owner must grant
    /// these before the new version can exist. `revise` re-screens because
    /// `compile` promises never to produce an authorization-grade Mission.
    NeedsAuthorization(Vec<AuthorizationRequest>),
}

impl std::fmt::Display for ReviseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownRequirement(r) => {
                write!(f, "no such inferred requirement: {r}")
            }
            Self::EmptyValue(s) => write!(f, "change value must not be empty: {s}"),
            Self::NeedsAuthorization(r) => write!(f, "{} authorization(s) required", r.len()),
        }
    }
}

impl std::error::Error for ReviseError {}
