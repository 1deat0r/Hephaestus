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
/// Visible inside the crate so the baseline comparison measures the
/// same subsamples the pilot batch runs (v1-release 02).
pub(crate) fn subsample(trace_text: &str, drop_heavy: usize) -> String {
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

fn fmt_num(value: Option<f64>) -> String {
    match value {
        Some(v) => format!("{v:.6}"),
        None => "none".to_string(),
    }
}

/// Render the batch report as a deterministic Markdown document
/// (v1-release 01). No clock, no I/O: same report in, same bytes
/// out. Provider spend is 0 with an explicit note - the fixture
/// chain runs no metered provider, and the report says so instead
/// of implying a measured zero-cost deployment.
pub fn render_pilot_report(report: &BatchReport) -> String {
    let mut out = String::new();
    out.push_str("# Pilot campaign report - fixture batch\n\n");
    out.push_str(&format!(
        "Preregistration: {}\n\nIntent: {}\n\nBatch: {} missions ({} repositories x 2 arms); clustering unit: {}; plan budget: {}.\n\n",
        report.plan.preregistration_id,
        report.plan.intent,
        report.plan.batch_size,
        report.plan.assignments.len(),
        report.variance.clustering_unit,
        report.plan.budget,
    ));
    out.push_str("Predeclared metrics: per-arm mean and variance,\npaired-difference variance across repositories, denominator\nreceipts with failures retained.\n\n");
    out.push_str("## Results\n\n");
    out.push_str("Theta shows 2 decimals and value 6 decimals for reading;\nthe receipt digest binds the full-precision run inputs, and\nvalue equals the interval lower bound minus theta.\n\n");
    out.push_str("| mission | repository | arm | dropped | theta | interval | value | receipt |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for m in &report.missions {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {:.2} | ({:.6}, {:.6}) | {:.6} | {} |\n",
            m.index,
            m.repo_id,
            m.arm_id,
            m.dropped_heavy_lines,
            m.theta,
            m.interval.0,
            m.interval.1,
            m.value,
            m.receipt_sha256,
        ));
    }
    out.push_str("\n## Variance (from actual runs)\n\n");
    for (arm, mean, var) in &report.variance.per_arm {
        out.push_str(&format!(
            "- arm {arm}: mean {}, variance {}\n",
            fmt_num(*mean),
            fmt_num(*var),
        ));
    }
    out.push_str(&format!(
        "- paired-difference variance: {}\n",
        fmt_num(report.variance.paired_difference_variance),
    ));
    out.push_str(&format!(
        "\nDenominators: total missions {}; failures retained: {}.\n",
        report.total_missions, report.failures_retained,
    ));
    out.push_str("\n## Cost\n\n");
    out.push_str(&format!(
        "- plan budget reserved: {} (fixture units).\n- provider spend: 0 - the fixture chain runs no metered provider;\n  this is absence of spend, not a measured zero-cost deployment.\n- compute and wall time: not metered in this batch (explicit gap).\n",
        report.plan.budget,
    ));
    out.push_str("\n## Failures\n\n");
    if report.failures_retained == 0 {
        out.push_str(
            "No mission failed or blocked in this batch. The count is\nrecorded, not omitted: failures retained in the denominator\nreads 0 above. Prospective live missions are out of scope for\nthis fixture report.\n",
        );
    } else {
        out.push_str(&format!(
            "{} mission(s) failed or blocked and stay in the denominator:\n\n",
            report.failures_retained,
        ));
        for o in &report.outcomes {
            if o.failed_or_blocked {
                out.push_str(&format!("- {} {}\n", o.repo_id, o.arm_id));
            }
        }
    }
    out
}

/// Frozen campaign manifest (frozen-campaign 01). The `frozen`
/// section binds the inputs a future qualification check needs;
/// it replays byte-identical. `provenance` carries run-local
/// toolchain facts, labeled as such. `scope` marks fixture
/// bounds on the face of the file: frozen inputs, not evidence.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FrozenManifest {
    pub preregistration_id: String,
    pub scope: String,
    pub package_version: String,
    pub trace_sha256: String,
    pub pilot_report_digest: String,
    pub baseline_report_digest: String,
    pub frozen: FrozenInputs,
    pub provenance: ProvenanceFacts,
}

/// The replay-stable core: plan plus input digests.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FrozenInputs {
    pub plan: crate::pilot::record::PilotPlan,
    pub trace_sha256: String,
    pub pilot_report_digest: String,
    pub baseline_report_digest: String,
}

/// Run-local facts: honest provenance, never frozen content.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProvenanceFacts {
    pub toolchain: String,
    pub os: String,
    pub arch: String,
}

/// Bind the fixture campaign inputs into a manifest. Substring
/// marker checks refuse files that do not read as the campaign
/// reports; a crafted file carrying the markers would pass, so the
/// check is provenance screening, never authentication. Digests
/// always bind (sha256 hex of the input bytes).
pub fn freeze_campaign(
    trace_text: &str,
    pilot_text: &str,
    baseline_text: &str,
) -> Result<FrozenManifest, String> {
    if !pilot_text.contains("prereg-fixture-campaign-1") {
        return Err("pilot input is not a pilot campaign report".to_string());
    }
    if !baseline_text.contains("Baseline comparison report") {
        return Err("baseline input is not a baseline comparison report".to_string());
    }
    let plan = batch_plan();
    let trace_sha256 = crate::knowledge::sha256_hex(trace_text.as_bytes());
    let pilot_digest = crate::knowledge::sha256_hex(pilot_text.as_bytes());
    let baseline_digest = crate::knowledge::sha256_hex(baseline_text.as_bytes());
    let toolchain = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "gap: rustc version unreadable".to_string());
    Ok(FrozenManifest {
        preregistration_id: plan.preregistration_id.clone(),
        scope: "fixture (not confirmatory evidence)".to_string(),
        package_version: env!("CARGO_PKG_VERSION").to_string(),
        trace_sha256: trace_sha256.clone(),
        pilot_report_digest: pilot_digest.clone(),
        baseline_report_digest: baseline_digest.clone(),
        frozen: FrozenInputs {
            plan,
            trace_sha256,
            pilot_report_digest: pilot_digest,
            baseline_report_digest: baseline_digest,
        },
        provenance: ProvenanceFacts {
            toolchain,
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        },
    })
}

/// Render the manifest as pretty JSON in struct field order.
/// Same-machine runs emit identical bytes; portable checks compare
/// the parsed frozen section instead of file bytes.
pub fn render_frozen_manifest(manifest: &FrozenManifest) -> Result<String, String> {
    serde_json::to_string_pretty(manifest).map_err(|e| format!("manifest serialize: {e}"))
}
