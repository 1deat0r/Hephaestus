//! Self-improvement service core (T-033, docs/SELF_IMPROVEMENT.md,
//! R-070-R-072, R-115-R-119).
//!
//! Deterministic candidate evaluation, promotion gates, rollback, and an
//! append-only ledger. Benefit status arrives from the qualified
//! evaluation (T-022 semantics); this service decides DEPLOYMENT. False,
//! inconclusive, over-budget, and permission-expanding challengers stay
//! undeployed (§R-119); a candidate cannot attest itself (§R-117);
//! budgets cannot multiply (§R-115); safety policy, spending limits,
//! disclosure scope, protected data, evaluator, analysis qualification,
//! and promotion authority are NEVER changeable through this loop
//! (§R-119 boundary).
//!
//! Limitations (also in the spec): live trigger wiring and canary
//! watchdog are typed records, not wired services; end-to-end crash
//! demonstration remains for the T-033 exit demo.

pub mod learning;
pub mod learning_gate;
pub mod record;

pub use record::{
    Assessment, BenefitStatus, Deployment, EvaluationManifest, GuardrailStatus,
    ImprovementCandidate, ImprovementLedger, LedgerEntry, PromotionRejection, RollbackReceipt,
    StandingGrant,
};

/// Evaluate a candidate's eligibility for promotion BEFORE trusting any
/// outcome (§R-115, §R-117): self-attestation and budget-multiplication
/// checks run first; a proposal that budgets from itself is refused.
pub fn evaluate_candidate(
    candidate: &ImprovementCandidate,
    assessment: &Assessment,
) -> Result<(), PromotionRejection> {
    // §R-117: a model cannot attest its own qualification.
    if candidate.challenger_digest == assessment.evaluator_digest {
        return Err(PromotionRejection::SelfAttestation);
    }
    // §R-115: recursion stays in the original envelope.
    if candidate.budget_from == candidate.challenger_id {
        return Err(PromotionRejection::BudgetMultiplication);
    }
    Ok(())
}

/// Promote a qualified candidate (§R-117, §R-118): named rejections; on
/// success the champion pointer updates transactionally WITHOUT
/// overwriting the incumbent (rollback target = retained incumbent).
pub fn promote(
    candidate: &ImprovementCandidate,
    assessment: &Assessment,
    grant: &StandingGrant,
    protected_verified_artifacts: &[String],
    holdout_admissible: bool,
) -> Result<Deployment, PromotionRejection> {
    evaluate_candidate(candidate, assessment)?;

    // §R-117 negative case: a leaked holdout (exposed to selection, not
    // separately qualified) never grounds a deployment decision.
    if !holdout_admissible {
        return Err(PromotionRejection::LeakedHoldout);
    }

    // §R-117 negative case: confounded evaluation — favorable
    // self-ratings on confounded runs must not deploy.
    if assessment.confounds_present {
        return Err(PromotionRejection::ConfoundedEvaluation);
    }

    // §R-117: verified candidate/evaluation/rollback artifacts.
    let verified = |d: &str| protected_verified_artifacts.iter().any(|v| v == d);
    if !verified(&candidate.challenger_digest)
        || !verified(&candidate.rollout_artifact_digest)
        || !verified(&candidate.rollback_artifact_digest)
    {
        return Err(PromotionRejection::UnverifiedArtifacts);
    }

    // §R-117: supported predeclared benefit; inconclusive does not deploy.
    match assessment.benefit_status {
        BenefitStatus::Supported => {}
        BenefitStatus::Inconclusive => return Err(PromotionRejection::Inconclusive),
        BenefitStatus::Unsupported => return Err(PromotionRejection::FalseBenefit),
    }

    // §R-117: passing guardrails.
    if assessment.guardrail_status != GuardrailStatus::Passing {
        return Err(PromotionRejection::FalseBenefit);
    }

    // §R-117: resource budget from the frozen manifest.
    if assessment.challenger_cost > candidate.manifest.total_resource_budget {
        return Err(PromotionRejection::OverBudget);
    }

    // R-071 / §R-119: the grant must cover the target; protected targets
    // (evaluator, spending limits, safety policy...) are never covered.
    if grant.protected_targets.contains(&candidate.target) {
        return Err(PromotionRejection::PermissionExpanding);
    }
    if !grant.covered_targets.contains(&candidate.target) {
        return Err(PromotionRejection::NoScopedGrant);
    }

    Ok(Deployment {
        challenger_id: candidate.challenger_id.clone(),
        challenger_digest: candidate.challenger_digest.clone(),
        incumbent_id: candidate.incumbent_id.clone(),
        incumbent_digest: candidate.incumbent_digest.clone(),
        grant_scope: grant.covered_targets.clone(),
        assessment_payload_digest: assessment.bound_payload_digest.clone(),
        observed_scope: "canary-bounded".to_string(),
    })
}

/// Roll back a deployment (§R-118): restore the VERIFIED incumbent,
/// retain reason + observations; the rolled-back candidate keeps its
/// identity so retries require a NEW version (same-digest retry is
/// refused by callers checking `rolled_back_challenger_digest`).
pub fn rollback(
    deployment: &Deployment,
    verified_incumbent_digest: &str,
    reason: &str,
    observations: &str,
) -> Result<RollbackReceipt, PromotionRejection> {
    // Restore only a verified incumbent (never an unverified guess).
    if deployment.incumbent_digest != verified_incumbent_digest {
        return Err(PromotionRejection::UnverifiedArtifacts);
    }
    Ok(RollbackReceipt {
        restored_incumbent_digest: verified_incumbent_digest.to_string(),
        rolled_back_challenger_digest: deployment.challenger_digest.clone(),
        reason: reason.to_string(),
        observations: observations.to_string(),
    })
}
