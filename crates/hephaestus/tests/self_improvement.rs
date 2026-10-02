//! Self-improvement service (T-033, R-070-R-072, R-095/AT-095
//! (self-attested records never gain protected trust), R-115-R-119,
//! AT-117 (self-attested/inconclusive/confounded/leaked-holdout
//! challengers stay undeployed)).
//!
//! Integration tests at the public seam: `evaluate_candidate`,
//! `promote`, `rollback`, `ImprovementLedger`.

use hephaestus::selfimprove::record::{
    Assessment, BenefitStatus, EvaluationManifest, GuardrailStatus, ImprovementCandidate,
    ImprovementLedger, LedgerEntry, PromotionRejection, StandingGrant,
};
use hephaestus::selfimprove::{evaluate_candidate, promote, rollback};

fn s(v: &str) -> String {
    v.to_string()
}

fn candidate() -> ImprovementCandidate {
    ImprovementCandidate {
        incumbent_id: s("champion-prompt-v3"),
        incumbent_digest: s("incumbent-digest"),
        challenger_id: s("challenger-prompt-v4"),
        challenger_digest: s("challenger-digest"),
        target: s("prompt"),
        supporting_observation_ids: vec![s("obs-1"), s("obs-2")],
        fresh_partition_id: s("fresh-partition-7"),
        budget_from: s("standing-envelope-1"),
        intended_primary_benefit: s("higher qualified-yield per mission"),
        guardrails: vec![s("false-promotion-rate")],
        rollout_artifact_digest: s("rollout-digest"),
        rollback_artifact_digest: s("rollback-digest"),
        manifest: EvaluationManifest {
            effect_bound: s("lower bound > 0"),
            noninferiority_bound: s("quality loss <= 0.05"),
            sampling_units: s("repository-level"),
            unsuccessful_run_denominators: s("all assigned missions"),
            tooling_access: s("identical approved envelope"),
            method_qualification: s("paired-repo-difference 1.0.0"),
            stopping_rule: s("fixed sample"),
            multiplicity_rule: s("Bonferroni family"),
            total_resource_budget: 10,
        },
    }
}

fn assessment() -> Assessment {
    Assessment {
        benefit_status: BenefitStatus::Supported,
        guardrail_status: GuardrailStatus::Passing,
        evaluator_digest: s("protected-evaluator-digest"),
        bound_payload_digest: s("challenger-digest"),
        challenger_cost: 5,
        confounds_present: false,
    }
}

fn grant() -> StandingGrant {
    StandingGrant {
        covered_targets: vec![s("prompt"), s("scheduling")],
        protected_targets: vec![s("evaluator"), s("spending-limits"), s("safety-policy")],
        max_budget: 100,
        verified_artifacts: vec![
            s("challenger-digest"),
            s("rollout-digest"),
            s("rollback-digest"),
            s("incumbent-digest"),
        ],
    }
}

// ---- Ticket 01: evaluate + promote + rejection classes ----

#[test]
fn promotion_succeeds_and_retains_incumbent() {
    let d = promote(
        &candidate(),
        &assessment(),
        &grant(),
        &grant().verified_artifacts,
        true,
    )
    .expect("deployed");
    // Transactional pointer: challenger becomes champion, incumbent
    // RETAINED as rollback target (§R-118).
    assert_eq!(d.challenger_digest, "challenger-digest");
    assert_eq!(d.incumbent_digest, "incumbent-digest");
    assert_eq!(d.observed_scope, "canary-bounded");
}

#[test]
fn rejection_classes_undeployed() {
    let c = candidate();
    let a = assessment();
    let g = grant();
    // False benefit.
    let mut a_false = a.clone();
    a_false.benefit_status = BenefitStatus::Unsupported;
    assert_eq!(
        promote(&c, &a_false, &g, &g.verified_artifacts, true),
        Err(PromotionRejection::FalseBenefit)
    );
    // Inconclusive does not deploy (§R-117).
    let mut a_inc = a.clone();
    a_inc.benefit_status = BenefitStatus::Inconclusive;
    assert_eq!(
        promote(&c, &a_inc, &g, &g.verified_artifacts, true),
        Err(PromotionRejection::Inconclusive)
    );
    // Over budget.
    let mut a_over = a.clone();
    a_over.challenger_cost = 99;
    assert_eq!(
        promote(&c, &a_over, &g, &g.verified_artifacts, true),
        Err(PromotionRejection::OverBudget)
    );
    // Permission-expanding: target = protected evaluator (R-071).
    let mut c_perm = c.clone();
    c_perm.target = s("evaluator");
    assert_eq!(
        promote(&c_perm, &a, &g, &g.verified_artifacts, true),
        Err(PromotionRejection::PermissionExpanding)
    );
    // No scoped grant for an uncovered target.
    let mut c_uncovered = c.clone();
    c_uncovered.target = s("distributed-execution");
    assert_eq!(
        promote(&c_uncovered, &a, &g, &g.verified_artifacts, true),
        Err(PromotionRejection::NoScopedGrant)
    );
    // Unverified artifacts.
    assert_eq!(
        promote(&c, &a, &g, &[], true),
        Err(PromotionRejection::UnverifiedArtifacts)
    );
}

#[test]
fn self_attestation_and_budget_multiplication_rejected() {
    let c = candidate();
    // §R-117: challenger digest == evaluator digest -> self-attestation.
    let mut a_self = assessment();
    a_self.evaluator_digest = s("challenger-digest");
    assert_eq!(
        evaluate_candidate(&c, &a_self),
        Err(PromotionRejection::SelfAttestation)
    );
    // §R-115: proposal budgeted from itself.
    let mut c_self = c.clone();
    c_self.budget_from = c_self.challenger_id.clone();
    assert_eq!(
        evaluate_candidate(&c_self, &assessment()),
        Err(PromotionRejection::BudgetMultiplication)
    );
}

// ---- Ticket 02: rollback + ledger ----

#[test]
fn rollback_restores_verified_incumbent() {
    let d = promote(
        &candidate(),
        &assessment(),
        &grant(),
        &grant().verified_artifacts,
        true,
    )
    .unwrap();
    // Violation triggers rollback (§R-118).
    let receipt = rollback(
        &d,
        "incumbent-digest",
        "guardrail drift",
        "canary fp-rate 2% > 1%",
    )
    .expect("rolled back");
    assert_eq!(receipt.restored_incumbent_digest, "incumbent-digest");
    assert_eq!(receipt.rolled_back_challenger_digest, "challenger-digest");
    assert!(receipt.reason.contains("guardrail"));
    // Retries need a new version: the receipt names the rolled-back
    // digest so same-digest retries can be refused by callers.
    // Unverified incumbent is never restored.
    assert_eq!(
        rollback(&d, "WRONG", "r", "o"),
        Err(PromotionRejection::UnverifiedArtifacts)
    );
}

#[test]
fn ledger_append_only_and_restart_survives() {
    let mut ledger = ImprovementLedger::default();
    ledger.append(LedgerEntry {
        challenger_id: s("challenger-prompt-v4"),
        challenger_digest: s("challenger-digest"),
        incumbent_id: s("champion-prompt-v3"),
        incumbent_digest: s("incumbent-digest"),
        outcome: s("deployed"),
        reason: s("supported benefit, guardrails passing"),
        observations: s("fresh partition 7"),
        target: s("prompt"),
        fresh_partition_id: s("fresh-partition-7"),
    });
    ledger.append(LedgerEntry {
        challenger_id: s("challenger-prompt-v5"),
        challenger_digest: s("digest-v5"),
        incumbent_id: s("champion-prompt-v3"),
        incumbent_digest: s("incumbent-digest"),
        outcome: s("rejected"),
        reason: s("inconclusive"),
        observations: s("interval overlaps bound"),
        target: s("prompt"),
        fresh_partition_id: s("fresh-partition-8"),
    });
    // §R-116: restart reconstructs from persisted JSON; learned
    // decisions are not lost.
    let json = ledger.to_json();
    let restored = ImprovementLedger::from_json(&json).expect("round-trip json recovers");
    assert_eq!(restored.entries().len(), 2);
    assert_eq!(restored.entries()[1].outcome, "rejected");
    assert_eq!(restored.entries()[1].reason, "inconclusive");
}

#[test]
fn twin_run_byte_identical() {
    let a = promote(
        &candidate(),
        &assessment(),
        &grant(),
        &grant().verified_artifacts,
        true,
    )
    .unwrap();
    let b = promote(
        &candidate(),
        &assessment(),
        &grant(),
        &grant().verified_artifacts,
        true,
    )
    .unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}

#[test]
fn leaked_holdout_and_confounded_challengers_rejected() {
    let c = candidate();
    let a = assessment();
    let g = grant();
    // §R-117 negative case: leaked holdout (exposed to selection, not
    // separately qualified) — refusal even with otherwise-qualified
    // evidence.
    assert_eq!(
        promote(&c, &a, &g, &g.verified_artifacts, false),
        Err(PromotionRejection::LeakedHoldout)
    );
    // §R-117 negative case: confounded evaluation — a favorable
    // self-rating on a confounded run must not deploy.
    let mut a_conf = a.clone();
    a_conf.confounds_present = true;
    assert_eq!(
        promote(&c, &a_conf, &g, &g.verified_artifacts, true),
        Err(PromotionRejection::ConfoundedEvaluation)
    );
    // Both clean: deploys (prior semantics conserved).
    assert!(promote(&c, &a, &g, &g.verified_artifacts, true).is_ok());
}
