//! Fixture campaign driver (dev-roadmap ticket 01; Covers AC 1).
//!
//! Runs a 20-mission batch end-to-end through the real E2E chain
//! (`missionrun::run`) over subsamples of the world-derived trace
//! fixture, then computes pilot variance from the ACTUAL runs
//! (R-103: failures retained in denominators; here the fixture is
//! clean, so the receipt records zero explicitly — never omitted).
//!
//! Design (deterministic, no randomness, no clock):
//! - 10 repositories x 2 arms (`cand`, `base`) = 20 missions.
//! - Repository `k` runs the chain over the fixture trace with the
//!   first `k` heavy-stage lines removed (a real data subsample —
//!   the interval moves with the data, so paired variance is nonzero).
//! - Arm thetas are declared per mission: cand `0.25 + 0.01*k`,
//!   base `10.0`. The recorded value is the decision margin
//!   (interval lower bound minus declared theta), computed from the
//!   actual chain output — variance over real run products.
//! - Outcomes feed `pilot::record_outcome`; the report carries the
//!   pilot plan, the per-mission receipts, and the variance estimate.

use crate::missionrun;
use crate::pilot::record::{
    MissionOutcome, Partition, PilotPlan, Stratification, VarianceEstimate,
};
use crate::pilot::{plan_pilot, record_outcome};

/// Missions per batch: 10 repositories x 2 arms (R-103 denominators).
pub const BATCH_MISSIONS: usize = 20;
/// Repositories per batch.
pub const BATCH_REPOS: usize = 10;
/// Cand-arm base theta; repo `k` declares `CAND_THETA_BASE + 0.01*k`.
pub const CAND_THETA_BASE: f64 = 0.25;
/// Base-arm theta (mirrors the honest-negative leg of mission-run).
pub const BASE_THETA: f64 = 10.0;

/// One mission inside the batch: declared inputs + actual chain output.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MissionReceipt {
    pub index: usize,
    pub repo_id: String,
    pub arm_id: String,
    pub theta: f64,
    pub dropped_heavy_lines: usize,
    /// sha256 of the subsampled trace actually run (run receipt).
    pub receipt_sha256: String,
    /// Qualified interval from the actual chain run, seconds.
    pub interval: (f64, f64),
    /// Recorded pilot value: interval lower bound minus theta.
    pub value: f64,
}

/// The batch report: plan, receipts, outcomes, variance (R-103).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BatchReport {
    pub plan: PilotPlan,
    pub missions: Vec<MissionReceipt>,
    pub outcomes: Vec<MissionOutcome>,
    pub variance: VarianceEstimate,
    /// Total missions run (denominator receipt, R-103).
    pub total_missions: usize,
    /// Failed/blocked missions retained in the denominator (R-103).
    pub failures_retained: usize,
}

/// Subsample the trace: drop the first `drop_heavy` heavy-stage lines.
/// Pure string transform over caller-supplied data (no I/O).
fn subsample(trace_text: &str, drop_heavy: usize) -> String {
    let mut dropped = 0usize;
    let kept: Vec<&str> = trace_text
        .split_inclusive('\n')
        .filter(|line| {
            let is_heavy = line.starts_with("step=heavy ");
            if is_heavy && dropped < drop_heavy {
                dropped += 1;
                false
            } else {
                true
            }
        })
        .collect();
    kept.concat()
}

/// The preregistered pilot plan behind the batch (R-103).
pub fn batch_plan() -> PilotPlan {
    PilotPlan {
        preregistration_id: "prereg-fixture-campaign-1".to_string(),
        stratification: Stratification {
            size_bands: vec!["fixture".to_string()],
            topology_classes: vec!["flat".to_string()],
        },
        assignments: (0..BATCH_REPOS)
            .map(|k| crate::pilot::record::RepositoryAssignment {
                repo_id: format!("trace-repo-{k}"),
                partition: Partition::Pilot,
                episode_ids: vec![format!("ep-{k}-cand"), format!("ep-{k}-base")],
                size_band: "fixture".to_string(),
                topology_class: "flat".to_string(),
            })
            .collect(),
        batch_size: BATCH_MISSIONS,
        intent: "debugging batch — variance calibration only, not evidence".to_string(),
        budget: 20,
        clustering_unit: "repository".to_string(),
    }
}

/// Run the 20-mission batch through the real chain; variance comes
/// from the actual runs (R-103 denominators retained).
pub fn run_fixture_batch(trace_text: &str) -> Result<BatchReport, String> {
    let plan = plan_pilot(batch_plan(), &["prereg-fixture-campaign-1".to_string()])
        .map_err(|e| format!("campaign plan rejected: {e:?}"))?;
    let mut missions = Vec::with_capacity(BATCH_MISSIONS);
    let mut outcomes = Vec::with_capacity(BATCH_MISSIONS);
    for k in 0..BATCH_REPOS {
        let sub = subsample(trace_text, k);
        for arm in ["cand", "base"] {
            let theta = if arm == "cand" {
                CAND_THETA_BASE + 0.01 * k as f64
            } else {
                BASE_THETA
            };
            let receipt = missionrun::run(&sub, theta, None)
                .map_err(|e| format!("mission {k}/{arm}: {e}"))?;
            let value = receipt.interval.0 - theta;
            missions.push(MissionReceipt {
                index: missions.len(),
                repo_id: format!("trace-repo-{k}"),
                arm_id: arm.to_string(),
                theta,
                dropped_heavy_lines: k,
                receipt_sha256: receipt.receipt_sha256.clone(),
                interval: receipt.interval,
                value,
            });
            outcomes.push(MissionOutcome {
                repo_id: format!("trace-repo-{k}"),
                arm_id: arm.to_string(),
                value: Some(value),
                failed_or_blocked: false,
            });
        }
    }
    let variance = record_outcome(&plan, &outcomes);
    Ok(BatchReport {
        plan,
        missions,
        outcomes,
        failures_retained: variance.failures_retained,
        total_missions: variance.total_missions,
        variance,
    })
}
