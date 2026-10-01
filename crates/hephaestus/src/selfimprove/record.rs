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
}

impl ImprovementLedger {
    pub fn append(&mut self, entry: LedgerEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }

    /// Reconstruct from persisted JSON (restart recovery, §R-116).
    pub fn from_json(json: &str) -> Self {
        serde_json::from_str(json).unwrap_or_default()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}
