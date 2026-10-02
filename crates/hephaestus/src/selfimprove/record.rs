//! Self-improvement records (T-033, docs/SELF_IMPROVEMENT.md, R-115-R-119).

use serde::{Deserialize, Serialize};

/// The frozen evaluation manifest (§R-117): effect/noninferiority
/// bounds, sampling units, unsuccessful-run denominators, tooling
/// access, method qualification, stopping + multiplicity rules — frozen
/// BEFORE outcomes are accessed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EvaluationManifest {
    pub effect_bound: String,
    pub noninferiority_bound: String,
    pub sampling_units: String,
    /// Complete unsuccessful-run denominators (failed/blocked/invalid).
    pub unsuccessful_run_denominators: String,
    pub tooling_access: String,
    pub method_qualification: String,
    pub stopping_rule: String,
    pub multiplicity_rule: String,
    /// Total resource budget the challenger may consume.
    pub total_resource_budget: u64,
}

/// An improvement candidate (§R-117): every field required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovementCandidate {
    pub incumbent_id: String,
    pub incumbent_digest: String,
    pub challenger_id: String,
    pub challenger_digest: String,
    /// What the change targets (prompt / operator / context / scheduling
    /// / code — code deployment needs its own capability).
    pub target: String,
    pub supporting_observation_ids: Vec<String>,
    /// Fresh evaluation partition/family id (never a used confirmation
    /// set).
    pub fresh_partition_id: String,
    /// Parent budget source (§R-115: recursion stays in the original
    /// envelope — a proposal budgeted from itself is rejected).
    pub budget_from: String,
    pub intended_primary_benefit: String,
    pub guardrails: Vec<String>,
    pub rollout_artifact_digest: String,
    pub rollback_artifact_digest: String,
    pub manifest: EvaluationManifest,
}

/// Benefit status from the qualified evaluation (T-022 semantics):
/// arrives computed, never guessed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenefitStatus {
    Supported,
    Inconclusive,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardrailStatus {
    Passing,
    Violated,
}

/// The authenticated assessment (§R-117): bound to the exact payload,
/// evaluator identity EXTERNAL to the challenger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assessment {
    pub benefit_status: BenefitStatus,
    pub guardrail_status: GuardrailStatus,
    /// Digest of the evaluator that produced this assessment.
    pub evaluator_digest: String,
    /// Challenger payload digest this assessment is bound to.
    pub bound_payload_digest: String,
    /// Observed challenger resource cost.
    pub challenger_cost: u64,
    /// Does the evaluation declare confounds present? (§R-117 negative
    /// case: a favorable rating on a confounded evaluation must not
    /// deploy.)
    pub confounds_present: bool,
}

/// Named promotion rejections (§R-117, §R-119): each class undeployed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionRejection {
    /// Benefit not supported by the protected evaluation.
    FalseBenefit,
    /// Interval overlaps the bound (§14:281 continuity).
    Inconclusive,
    /// Challenger cost exceeds the manifest budget.
    OverBudget,
    /// Grant scope does not cover the change target — permission
    /// expansion through the improvement loop is refused (R-071).
    PermissionExpanding,
    /// The challenger attested itself (challenger digest == evaluator
    /// digest, §R-117).
    SelfAttestation,
    /// Candidate/evaluation/rollback artifacts not verified.
    UnverifiedArtifacts,
    /// No current scoped grant covers this change.
    NoScopedGrant,
    /// The proposal budgets from itself (§R-115).
    BudgetMultiplication,
    /// The evaluation holdout was exposed to the selection process and
    /// is not separately qualified (§R-117 negative case, R-101
    /// continuity).
    LeakedHoldout,
    /// The assessment declares confounds present — a favorable rating
    /// on a confounded evaluation must not deploy (§R-117 negative
    /// case).
    ConfoundedEvaluation,
}

/// The standing grant (owner-authorized scope + envelope, §R-115).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandingGrant {
    /// Change targets covered, e.g. ["prompt", "scheduling"].
    pub covered_targets: Vec<String>,
    /// Protected targets this grant can NEVER cover (R-071).
    pub protected_targets: Vec<String>,
    pub max_budget: u64,
    /// Artifact digests verified by the protected side (R-095).
    pub verified_artifacts: Vec<String>,
}

/// A deployment (§R-118): transactional champion-pointer update WITHOUT
/// overwriting the incumbent; rollback target recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deployment {
    pub challenger_id: String,
    pub challenger_digest: String,
    /// The retained incumbent (rollback target).
    pub incumbent_id: String,
    pub incumbent_digest: String,
    pub grant_scope: Vec<String>,
    pub assessment_payload_digest: String,
    pub observed_scope: String,
}

/// A rollback receipt (§R-118): reason + observations retained; retries
/// need new evidence/version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackReceipt {
    pub restored_incumbent_digest: String,
    pub rolled_back_challenger_digest: String,
    pub reason: String,
    pub observations: String,
}

/// One ledger entry (§R-116): appended for EVERY candidate, deployed or
/// rejected, with provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub challenger_id: String,
    pub challenger_digest: String,
    pub incumbent_id: String,
    /// Provenance of the retained incumbent this entry supersedes or
    /// restores (R-116: both identities carried with digests).
    pub incumbent_digest: String,
    pub outcome: String,
    pub reason: String,
    pub observations: String,
    pub target: String,
    pub fresh_partition_id: String,
}

/// Append-only improvement ledger (§R-116): durable learning that
/// survives restart by re-reading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ImprovementLedger {
    entries: Vec<LedgerEntry>,
    /// Recorded triggers (R-115: triggers ARE recorded, durable with
    /// the ledger). `serde(default)` keeps pre-trigger ledgers valid
    /// while entries stay strict.
    #[serde(default)]
    triggers: Vec<TriggerRecord>,
}

impl ImprovementLedger {
    pub fn append(&mut self, entry: LedgerEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }

    /// Reconstruct from persisted JSON (restart recovery, §R-116/118).
    /// FAIL-CLOSED: a corrupt or truncated ledger REFUSES — returning an
    /// empty ledger would silently erase every learned decision, which
    /// the obligations forbid. Callers must handle the error.
    pub fn from_json(json: &str) -> Result<Self, RecoverError> {
        serde_json::from_str(json).map_err(|e| RecoverError::CorruptLedger(e.to_string()))
    }

    /// Atomic persist (R-118): write a temp file, then rename — a crash
    /// mid-write can never leave a half-state behind as the main file
    /// (the leftover temp is ignored + noted by `recover`).
    pub fn persist(&self, path: &std::path::Path) -> Result<(), RecoverError> {
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, self.to_json()).map_err(|e| RecoverError::Io(e.to_string()))?;
        std::fs::rename(&tmp, path).map_err(|e| RecoverError::Io(e.to_string()))?;
        Ok(())
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// The recorded triggers, in order (R-115).
    pub fn triggers(&self) -> &[TriggerRecord] {
        &self.triggers
    }

    /// Append a recorded trigger (crate-internal: record_trigger owns
    /// validation).
    pub(crate) fn push_trigger(&mut self, trigger: TriggerRecord) {
        self.triggers.push(trigger);
    }
}

/// What fired a cycle (R-115: mission completion, predeclared
/// batch/interval, independently measured drift, repeated failure
/// class, or an owner request — recorded, never inferred).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerKind {
    MissionCompletion,
    BatchInterval,
    DriftMeasured,
    RepeatedFailure,
    OwnerRequest,
}

/// One recorded trigger on the ledger (R-115). `cycle_index` feeds the
/// cycle's declared deadline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerRecord {
    pub kind: TriggerKind,
    pub subject: String,
    pub detail: String,
    pub cycle_index: u64,
}

/// Why trigger recording refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerError {
    EmptySubject,
}

impl std::fmt::Display for TriggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TriggerError::EmptySubject => write!(f, "trigger subject is empty"),
        }
    }
}

impl std::error::Error for TriggerError {}

/// What one improvement cycle is allowed to do (R-115: reserved
/// budget, candidate cap, deadline, stop rule — declared, never
/// interpreted as model prose).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CyclePolicy {
    pub max_candidates: usize,
    pub reserved_budget: u64,
    pub deadline_cycle: u64,
    pub stop_rules: Vec<String>,
}

/// The outcome of a bounded cycle (R-115): a proposal, or the
/// RECORDED absence of a justified change.
#[derive(Debug, Clone, PartialEq)]
pub enum CycleOutcome {
    /// Boxed: the candidate outweighs the no-change arm by ~20x
    /// (clippy large_enum_variant — box, never pad the common path).
    Proposed {
        candidate: Box<ImprovementCandidate>,
    },
    NoJustifiedChange {
        reason: String,
    },
}

/// Why a cycle refused to run (typed-rejection convention).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CycleError {
    MissingStopRule,
    CandidateCapExceeded,
    BudgetCapExceeded,
    DeadlinePassed,
    /// The single proposal edge refused the input.
    ProposalRefused {
        reason: String,
    },
}

impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CycleError::MissingStopRule => write!(f, "cycle has no stop rule"),
            CycleError::CandidateCapExceeded => write!(f, "cycle exceeds its candidate cap"),
            CycleError::BudgetCapExceeded => write!(f, "cycle exceeds its reserved budget"),
            CycleError::DeadlinePassed => write!(f, "cycle is past its declared deadline"),
            CycleError::ProposalRefused { reason } => write!(f, "proposal edge refused: {reason}"),
        }
    }
}

impl std::error::Error for CycleError {}

/// The monitor's declared bounds (R-118): how many checks it may run,
/// its stop rules, and the incumbent digest the caller has VERIFIED
/// (rollback refuses without this proof).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorPolicy {
    pub max_checks: u64,
    pub stop_rules: Vec<String>,
    pub verified_incumbent_digest: String,
}

/// Result of monitoring a deployment to its bound (R-118): either the
/// stream completed inside the bound, or a breach stopped the rollout
/// with a real rollback receipt.
#[derive(Debug, Clone, PartialEq)]
pub enum MonitorOutcome {
    Completed {
        checks: u64,
    },
    Stopped {
        checks: u64,
        rollback: RollbackReceipt,
    },
}

/// Why monitoring refused to start (typed-rejection convention).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorError {
    MissingStopRule,
    InvalidPolicy,
    /// The caller could not prove the incumbent — checked BEFORE any
    /// observation (never a vacuous rollback).
    UnverifiedIncumbent {
        expected: String,
        found: String,
    },
    /// The rollback itself refused (pre-checked digest mismatch).
    RollbackRefused {
        reason: String,
    },
}

impl std::fmt::Display for MonitorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitorError::MissingStopRule => write!(f, "monitor has no stop rule"),
            MonitorError::InvalidPolicy => write!(f, "monitor policy is invalid"),
            MonitorError::UnverifiedIncumbent { expected, found } => write!(
                f,
                "unverified incumbent: policy {expected} != deployment {found}"
            ),
            MonitorError::RollbackRefused { reason } => write!(f, "rollback refused: {reason}"),
        }
    }
}

impl std::error::Error for MonitorError {}

/// The bounded, evidence-attached proposal input (R-115): the service
/// constructs a candidate FROM recorded observations — never from a
/// guess, never budgeted from itself.
#[derive(Debug, Clone, PartialEq)]
pub struct ProposalInput {
    pub target: String,
    pub incumbent_id: String,
    pub incumbent_digest: String,
    pub challenger_id: String,
    pub challenger_digest: String,
    pub supporting_observation_ids: Vec<String>,
    pub fresh_partition_id: String,
    pub budget_from: String,
    pub intended_primary_benefit: String,
    pub guardrails: Vec<String>,
    pub rollout_artifact_digest: String,
    pub rollback_artifact_digest: String,
    pub manifest: EvaluationManifest,
}

/// Why proposal construction refused (typed-rejection convention).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalError {
    /// No (or blank) supporting observations — a guess, not evidence.
    UnseededProposal,
    /// The candidate would budget itself (R-115 envelope rule).
    SelfBudgeting,
}

impl std::fmt::Display for ProposalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalError::UnseededProposal => {
                write!(f, "proposal has no supporting observations")
            }
            ProposalError::SelfBudgeting => write!(f, "proposal budgets itself"),
        }
    }
}

impl std::error::Error for ProposalError {}

/// The ACTIVE champion for a target, resolved from the append-only
/// ledger: last `deployed` entry wins until a `rolled_back` entry
/// restores that deployment's incumbent (R-118/R-119).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Champion {
    pub id: String,
    pub digest: String,
}

/// Why ledger persistence/recovery refused (typed-rejection convention).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoverError {
    /// The persisted ledger is corrupt/truncated — never silently
    /// replaced with an empty one (R-116: restart loses nothing).
    CorruptLedger(String),
    Io(String),
}

impl std::fmt::Display for RecoverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoverError::CorruptLedger(m) => write!(f, "corrupt improvement ledger: {m}"),
            RecoverError::Io(m) => write!(f, "ledger io: {m}"),
        }
    }
}

impl std::error::Error for RecoverError {}

/// One thing recovery noticed while reconstructing state (R-118:
/// interrupted deployments are RECONCILED, never guessed away).
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryAction {
    /// No main file yet — a fresh start, recorded not assumed.
    FreshStart,
    /// Leftover temp from an interrupted atomic persist — ignored.
    StaleTempIgnored,
    /// A deployment was decided but never committed: the exact entry
    /// is returned for exactly-once replay by the caller.
    DeploymentInterrupted { entry: LedgerEntry },
    /// A torn pending file was discarded LOUDLY (reason retained).
    PendingDiscarded { reason: String },
}

/// Predeclared canary indicator (R-118): observed value against its
/// declared bound; a breach stops the rollout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Indicator {
    pub name: String,
    pub kind: BoundKind,
    pub observed: f64,
    pub limit: f64,
}

/// Which side of `limit` is acceptable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundKind {
    /// Lower bound: breach when observed < limit.
    AtLeast,
    /// Upper bound: breach when observed > limit.
    AtMost,
}

/// The predeclared indicator set a deployment is monitored against.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuardrailIndicators {
    pub indicators: Vec<Indicator>,
}

/// A breached guardrail — names the indicator (R-118 stop condition).
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryViolation {
    pub indicator: String,
    pub observed: f64,
    pub limit: f64,
    pub challenger_id: String,
}

impl std::fmt::Display for CanaryViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "guardrail '{}' breached: observed {} vs limit {} (challenger {})",
            self.indicator, self.observed, self.limit, self.challenger_id
        )
    }
}

impl std::error::Error for CanaryViolation {}
