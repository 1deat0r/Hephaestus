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
    Assessment, BenefitStatus, BoundKind, CanaryViolation, Champion, CycleError, CycleOutcome,
    CyclePolicy, Deployment, EvaluationManifest, GuardrailIndicators, GuardrailStatus,
    ImprovementCandidate, ImprovementLedger, Indicator, LedgerEntry, MonitorError, MonitorOutcome,
    MonitorPolicy, PromotionRejection, ProposalError, ProposalInput, RecoverError, RecoveryAction,
    RollbackReceipt, StandingGrant, TriggerError, TriggerKind, TriggerRecord,
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

/// Construct a bounded improvement candidate FROM recorded observations
/// (R-115 observe -> propose): evidence-attached, fresh-partitioned, and
/// never budgeted from itself. The refusal checks live HERE at the
/// proposal edge as well as in `evaluate_candidate` (defense in depth:
/// an unseeded or self-budgeted proposal never even becomes a candidate).
pub fn propose_from_observations(
    input: record::ProposalInput,
) -> Result<ImprovementCandidate, record::ProposalError> {
    if input.supporting_observation_ids.is_empty()
        || input
            .supporting_observation_ids
            .iter()
            .any(|o| o.trim().is_empty())
    {
        return Err(record::ProposalError::UnseededProposal);
    }
    if input.budget_from == input.challenger_id {
        return Err(record::ProposalError::SelfBudgeting);
    }
    Ok(ImprovementCandidate {
        incumbent_id: input.incumbent_id,
        incumbent_digest: input.incumbent_digest,
        challenger_id: input.challenger_id,
        challenger_digest: input.challenger_digest,
        target: input.target,
        supporting_observation_ids: input.supporting_observation_ids,
        fresh_partition_id: input.fresh_partition_id,
        budget_from: input.budget_from,
        intended_primary_benefit: input.intended_primary_benefit,
        guardrails: input.guardrails,
        rollout_artifact_digest: input.rollout_artifact_digest,
        rollback_artifact_digest: input.rollback_artifact_digest,
        manifest: input.manifest,
    })
}

/// Resolve the active champion for `target` from the append-only ledger
/// (R-118/R-119): the last `deployed` entry's challenger, replaced by
/// that deployment's incumbent when a later `rolled_back` entry restores
/// it. `None` means no deployment has ever happened for the target —
/// callers keep their incumbent default.
pub fn active_champion(
    ledger: &record::ImprovementLedger,
    target: &str,
) -> Option<record::Champion> {
    let mut active: Option<record::Champion> = None;
    for entry in ledger.entries().iter().filter(|e| e.target == target) {
        match entry.outcome.as_str() {
            "deployed" => {
                active = Some(record::Champion {
                    id: entry.challenger_id.clone(),
                    digest: entry.challenger_digest.clone(),
                });
            }
            "rolled_back" => {
                active = Some(record::Champion {
                    id: entry.incumbent_id.clone(),
                    digest: entry.incumbent_digest.clone(),
                });
            }
            _ => {}
        }
    }
    active
}

/// Stable identity of a ledger entry for exactly-once replay/dedup
/// (target + challenger + outcome): the same committed decision never
/// appends twice.
pub fn canonical_entry_id(entry: &record::LedgerEntry) -> String {
    format!("{}|{}|{}", entry.target, entry.challenger_id, entry.outcome)
}

fn pending_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("pending.json")
}

/// Write-ahead: the deployment decision becomes durable BEFORE it is
/// appended (R-118 interrupted deployments become replayable, not
/// lost). A crash here commits nothing and contradicts nothing.
pub fn begin_deployment(
    dir: &std::path::Path,
    entry: &record::LedgerEntry,
) -> Result<(), record::RecoverError> {
    std::fs::write(
        pending_path(dir),
        serde_json::to_string(entry).expect("entry serializes"),
    )
    .map_err(|e| record::RecoverError::Io(e.to_string()))
}

/// Commit a deployment exactly once: append (unless the canonical id is
/// already present), persist atomically, then clear the pending file —
/// every crash window therefore reconciles to exactly one committed
/// entry (R-118).
pub fn commit_deployment(
    dir: &std::path::Path,
    ledger: &mut record::ImprovementLedger,
    entry: record::LedgerEntry,
) -> Result<(), record::RecoverError> {
    let id = canonical_entry_id(&entry);
    let already = ledger.entries().iter().any(|e| canonical_entry_id(e) == id);
    if !already {
        ledger.append(entry);
    }
    ledger.persist(&dir.join("improvement-ledger.json"))?;
    let _ = std::fs::remove_file(pending_path(dir));
    Ok(())
}

/// Recover improvement state from a state directory (R-118): the
/// ledger is reconstructed FAIL-CLOSED (corrupt main refuses rather
/// than silently emptying — R-116), stale temps are noted and
/// ignored, and an interrupted deployment is reported for exactly-once
/// replay. Returns (ledger, reconciliation actions).
pub fn recover(
    dir: &std::path::Path,
) -> Result<(record::ImprovementLedger, Vec<record::RecoveryAction>), record::RecoverError> {
    let mut actions = Vec::new();
    let tmp = dir.join("improvement-ledger.json.tmp");
    if tmp.exists() {
        actions.push(record::RecoveryAction::StaleTempIgnored);
        let _ = std::fs::remove_file(&tmp);
    }
    let main = dir.join("improvement-ledger.json");
    let ledger = if main.exists() {
        let text =
            std::fs::read_to_string(&main).map_err(|e| record::RecoverError::Io(e.to_string()))?;
        record::ImprovementLedger::from_json(&text)?
    } else {
        actions.push(record::RecoveryAction::FreshStart);
        record::ImprovementLedger::default()
    };
    let pending = pending_path(dir);
    if pending.exists() {
        let text = std::fs::read_to_string(&pending)
            .map_err(|e| record::RecoverError::Io(e.to_string()))?;
        match serde_json::from_str::<record::LedgerEntry>(&text) {
            Ok(entry) => actions.push(record::RecoveryAction::DeploymentInterrupted { entry }),
            Err(e) => {
                actions.push(record::RecoveryAction::PendingDiscarded {
                    reason: e.to_string(),
                });
                let _ = std::fs::remove_file(&pending);
            }
        }
    }
    Ok((ledger, actions))
}

/// The wired canary step (R-118): a deployment checked against its
/// PREDECLARED indicators; any breach stops the rollout by naming the
/// indicator — the caller rolls back on `Err`.
pub fn check_deployment_guardrails(
    deployment: &record::Deployment,
    indicators: &record::GuardrailIndicators,
) -> Result<(), record::CanaryViolation> {
    for ind in &indicators.indicators {
        let breached = match ind.kind {
            record::BoundKind::AtLeast => ind.observed < ind.limit,
            record::BoundKind::AtMost => ind.observed > ind.limit,
        };
        if breached {
            return Err(record::CanaryViolation {
                indicator: ind.name.clone(),
                observed: ind.observed,
                limit: ind.limit,
                challenger_id: deployment.challenger_id.clone(),
            });
        }
    }
    Ok(())
}

/// Record a trigger on the ledger (R-115: triggers are recorded, not
/// inferred). The subject must be non-empty; durability rides the
/// ledger's persist path.
pub fn record_trigger(
    ledger: &mut record::ImprovementLedger,
    kind: record::TriggerKind,
    subject: &str,
    detail: &str,
    cycle_index: u64,
) -> Result<record::TriggerRecord, record::TriggerError> {
    if subject.trim().is_empty() {
        return Err(record::TriggerError::EmptySubject);
    }
    let trigger = record::TriggerRecord {
        kind,
        subject: subject.to_string(),
        detail: detail.to_string(),
        cycle_index,
    };
    ledger.push_trigger(trigger.clone());
    Ok(trigger)
}

/// One bounded improvement cycle (R-115): every bound is declared
/// policy — stop rule required, candidate cap, reserved budget,
/// deadline. A trigger with nothing justified produces the RECORDED
/// `NoJustifiedChange` outcome; the only proposal path remains
/// `propose_from_observations`.
pub fn run_improvement_cycle(
    trigger: &record::TriggerRecord,
    candidates: &[record::ProposalInput],
    policy: &record::CyclePolicy,
) -> Result<record::CycleOutcome, record::CycleError> {
    if policy.stop_rules.is_empty() || policy.stop_rules.iter().any(|r| r.trim().is_empty()) {
        return Err(record::CycleError::MissingStopRule);
    }
    if trigger.cycle_index > policy.deadline_cycle {
        return Err(record::CycleError::DeadlinePassed);
    }
    if candidates.len() > policy.max_candidates {
        return Err(record::CycleError::CandidateCapExceeded);
    }
    if candidates
        .iter()
        .any(|c| c.manifest.total_resource_budget > policy.reserved_budget)
    {
        return Err(record::CycleError::BudgetCapExceeded);
    }
    let seeded = candidates.iter().find(|c| {
        !c.supporting_observation_ids.is_empty()
            && c.supporting_observation_ids
                .iter()
                .all(|o| !o.trim().is_empty())
    });
    match seeded {
        None => Ok(record::CycleOutcome::NoJustifiedChange {
            reason: if candidates.is_empty() {
                "trigger produced no candidates".to_string()
            } else {
                "no candidate carries seeded observations".to_string()
            },
        }),
        Some(input) => match propose_from_observations(input.clone()) {
            Ok(candidate) => Ok(record::CycleOutcome::Proposed {
                candidate: Box::new(candidate),
            }),
            Err(record::ProposalError::UnseededProposal) => {
                Ok(record::CycleOutcome::NoJustifiedChange {
                    reason: "proposal edge found no seeded observations".to_string(),
                })
            }
            Err(other) => Err(record::CycleError::ProposalRefused {
                reason: other.to_string(),
            }),
        },
    }
}

/// Monitor a deployment against its predeclared indicators, bounded by
/// the policy (R-118): the loop runs at most `max_checks` observations
/// (the bound IS the stop rule — no wall clock in the module); the
/// first breach stops the rollout through the wired guardrail check and
/// a real rollback to the caller-PROVEN incumbent, with the violation
/// and observations retained in the receipt.
pub fn monitor_deployment(
    deployment: &record::Deployment,
    policy: &record::MonitorPolicy,
    observations: &[record::GuardrailIndicators],
) -> Result<record::MonitorOutcome, record::MonitorError> {
    if policy.stop_rules.is_empty() || policy.stop_rules.iter().any(|r| r.trim().is_empty()) {
        return Err(record::MonitorError::MissingStopRule);
    }
    if policy.max_checks == 0 {
        return Err(record::MonitorError::InvalidPolicy);
    }
    if policy.verified_incumbent_digest != deployment.incumbent_digest {
        return Err(record::MonitorError::UnverifiedIncumbent {
            expected: policy.verified_incumbent_digest.clone(),
            found: deployment.incumbent_digest.clone(),
        });
    }
    let mut checks = 0u64;
    for observation in observations.iter().take(policy.max_checks as usize) {
        checks += 1;
        if let Err(violation) = check_deployment_guardrails(deployment, observation) {
            let rollback = rollback(
                deployment,
                &policy.verified_incumbent_digest,
                &format!("canary stop: {violation}"),
                &format!("observations through check {checks}: {observation:?}"),
            )
            .map_err(|e| record::MonitorError::RollbackRefused {
                reason: format!("{e:?}"),
            })?;
            return Ok(record::MonitorOutcome::Stopped { checks, rollback });
        }
    }
    Ok(record::MonitorOutcome::Completed { checks })
}
