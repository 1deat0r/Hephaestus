//! Prospective pilot campaigns (T-027, R-103, campaign pilot paragraphs).
//!
//! Preregistered pilot plans with repository-level partition separation,
//! free batch size (twenty is a possible debugging batch, NOT a
//! mandatory evidence threshold), variance estimation from recorded
//! outcomes, and a confirmation gate. Arithmetic on recorded outcomes
//! only — no fabricated variance.

pub mod record;

pub use record::{
    ConfirmationBlock, MissionOutcome, Partition, PilotPlan, PlanError, RepositoryAssignment,
    Stratification, VarianceEstimate,
};

use std::collections::BTreeMap;

/// Plan a pilot (R-103): partition conflicts named; preregistration
/// required; episodes pre-sampled; non-empty batch.
pub fn plan_pilot(
    plan: PilotPlan,
    known_preregistrations: &[String],
) -> Result<PilotPlan, PlanError> {
    if !known_preregistrations.contains(&plan.preregistration_id) {
        return Err(PlanError::UnregisteredPreregistration);
    }
    if plan.assignments.is_empty() {
        return Err(PlanError::EmptyBatch);
    }
    // Partition separation BY REPOSITORY (campaign).
    let mut seen: BTreeMap<&str, &Partition> = BTreeMap::new();
    for a in &plan.assignments {
        if let Some(prev) = seen.get(a.repo_id.as_str()) {
            if prev != &&a.partition {
                return Err(PlanError::PartitionConflict(a.repo_id.clone()));
            }
        } else {
            seen.insert(a.repo_id.as_str(), &a.partition);
        }
        // R-103: episodes sampled BEFORE confirmation.
        if a.partition == Partition::Confirmatory && a.episode_ids.is_empty() {
            return Err(PlanError::EpisodesNotPreSampled);
        }
    }
    Ok(plan)
}

/// Estimate variance from recorded pilot outcomes: per-arm mean +
/// population variance, paired-difference variance across repositories,
/// failures retained in the denominator. Empty input -> explicit None
/// fields, never zeros (no fabricated variance).
pub fn record_outcome(outcomes: &[MissionOutcome]) -> VarianceEstimate {
    if outcomes.is_empty() {
        return VarianceEstimate::default();
    }
    // Group by arm.
    let mut by_arm: BTreeMap<String, Vec<Option<f64>>> = BTreeMap::new();
    let mut failures = 0usize;
    for o in outcomes {
        if o.failed_or_blocked {
            failures += 1;
        }
        by_arm.entry(o.arm_id.clone()).or_default().push(o.value);
    }
    let mut per_arm = Vec::new();
    for (arm, vals) in &by_arm {
        let present: Vec<f64> = vals.iter().filter_map(|v| *v).collect();
        if present.is_empty() {
            per_arm.push((arm.clone(), None, None));
            continue;
        }
        let n = present.len() as f64;
        let mean = present.iter().sum::<f64>() / n;
        let var = present.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        per_arm.push((arm.clone(), Some(mean), Some(var)));
    }
    // Paired-difference variance across repositories: per repository,
    // difference between the first two arms' values where both present.
    let mut by_repo: BTreeMap<String, Vec<Option<f64>>> = BTreeMap::new();
    for o in outcomes {
        by_repo.entry(o.repo_id.clone()).or_default().push(o.value);
    }
    let diffs: Vec<f64> = by_repo
        .values()
        .filter_map(|pair| match pair.as_slice() {
            [Some(a), Some(b)] => Some(b - a),
            _ => None,
        })
        .collect();
    let paired = if diffs.len() < 2 {
        None
    } else {
        let n = diffs.len() as f64;
        let mean = diffs.iter().sum::<f64>() / n;
        Some(diffs.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / n)
    };
    VarianceEstimate {
        per_arm,
        paired_difference_variance: paired,
        failures_retained: failures,
        total_missions: outcomes.len(),
    }
}

/// Gate into confirmation (campaign: predeclare metrics and analysis
/// BEFORE the confirmatory campaign).
pub fn ready_for_confirmation(
    estimate: Option<&VarianceEstimate>,
    oracle_resolved: bool,
    analysis_qualified: bool,
    pilot_partition_only: bool,
) -> Result<(), ConfirmationBlock> {
    let estimate = estimate.ok_or(ConfirmationBlock::MissingVarianceEstimate)?;
    if estimate.total_missions == 0 {
        return Err(ConfirmationBlock::MissingVarianceEstimate);
    }
    if !oracle_resolved {
        return Err(ConfirmationBlock::UnresolvedOracle);
    }
    if !analysis_qualified {
        return Err(ConfirmationBlock::UnqualifiedAnalysis);
    }
    if !pilot_partition_only {
        return Err(ConfirmationBlock::PartitionContamination);
    }
    Ok(())
}
