//! Advanced search evaluation with matched ablations (T-031).
//!
//! Each candidate acceleration mechanism is evaluated SEPARATELY against
//! a matched ablation (identical envelope, mechanism removed); promotion
//! requires measured value within quality guardrails AND disablability
//! (the core path stays functional). Absent infrastructure yields an
//! honest NotEvaluable — never fabricated measurements.

pub mod record;

pub use record::{
    AblationPair, CandidateMechanism, EnvelopeMismatch, MechanismKind, PairMeasurement,
    PromotionVerdict,
};

use serde::{Deserialize, Serialize};

/// Errors from pair construction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PairError {
    EnvelopeMismatch(EnvelopeMismatch),
    /// The baseline and the mechanism arm are the same arm.
    NotADistinctPair,
}

/// Build a matched ablation pair: the baseline arm (mechanism removed)
/// must have the identical envelope except the mechanism itself.
/// Mismatched envelopes are refused — that would not be a matched
/// ablation.
pub fn matched_ablation(
    mechanism: &CandidateMechanism,
    baseline_compute_budget: u64,
    baseline_tool_access: &[String],
) -> Result<AblationPair, PairError> {
    if mechanism.compute_budget != baseline_compute_budget {
        return Err(PairError::EnvelopeMismatch(EnvelopeMismatch {
            field: "compute_budget".to_string(),
            detail: format!(
                "{} vs baseline {}",
                mechanism.compute_budget, baseline_compute_budget
            ),
        }));
    }
    if mechanism.tool_access.len() != baseline_tool_access.len()
        || !mechanism
            .tool_access
            .iter()
            .zip(baseline_tool_access.iter())
            .all(|(a, b)| a == b)
    {
        return Err(PairError::EnvelopeMismatch(EnvelopeMismatch {
            field: "tool_access".to_string(),
            detail: "tool access sets differ".to_string(),
        }));
    }
    if mechanism.compute_budget == 0 && baseline_compute_budget == 0 {
        return Err(PairError::NotADistinctPair);
    }
    Ok(AblationPair {
        mechanism: mechanism.clone(),
        metric: "declared-metric".to_string(),
    })
}

/// Evaluate a recorded pair measurement (roadmap: promote only when
/// matched ablations demonstrate value within quality guardrails).
pub fn evaluate_pair(
    pair: &AblationPair,
    measurement: Option<&PairMeasurement>,
) -> PromotionVerdict {
    // Absent infrastructure: honestly not evaluated.
    if !pair.mechanism.infrastructure_available {
        return PromotionVerdict::NotEvaluable;
    }
    let Some(m) = measurement else {
        // Infrastructure exists but nothing recorded: not promotable yet.
        return PromotionVerdict::Retained;
    };
    // Guardrails first: violations fail the promotion regardless of value.
    if m.guardrail_violations > 0 {
        return PromotionVerdict::Retained;
    }
    // Measured value: the mechanism must actually improve the metric.
    if m.with_value <= m.without_value {
        return PromotionVerdict::Retained;
    }
    // Disablability: the core path must stay functional (campaign exit).
    if !pair.mechanism.disablable {
        return PromotionVerdict::Retained;
    }
    PromotionVerdict::Promoted
}

pub use record::{CachingComparator, CachingComparison};

/// The outcome of a context-caching benchmark review (R-083): either
/// the stronger comparator ran, or its exclusion is on the record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CachingReview {
    StrongerComparatorUsed,
    ExclusionJustified { justification: String },
}

/// Why benchmark review refused a caching comparison (typed-rejection
/// convention): neither the stronger comparator nor a justification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CachingReviewError {
    MissingStrongerComparatorOrJustification,
}

/// Benchmark review for proposed context caching (R-083/AT-083):
/// requests the stronger exact-cache comparator OR a recorded
/// exclusion justification — full reconstruction alone never passes
/// review (§28 straw-baseline rule).
pub fn review_caching_comparison(
    comparison: &CachingComparison,
) -> Result<CachingReview, CachingReviewError> {
    if comparison
        .comparators_used
        .contains(&record::CachingComparator::ExactCache)
    {
        return Ok(CachingReview::StrongerComparatorUsed);
    }
    match comparison.exclusion_justification.as_deref() {
        Some(j) if !j.is_empty() => Ok(CachingReview::ExclusionJustified {
            justification: j.to_string(),
        }),
        _ => Err(CachingReviewError::MissingStrongerComparatorOrJustification),
    }
}

/// One honestly measured deterministic run (v1-release 02 S1).
/// Correctness is the positive-margin rate on the fixture trace.
/// Tokens are fixed-tokenizer counts over the bytes actually run -
/// never provider metering. Cost is 0.0 by absence (no metered
/// provider exists in this path). Latency is local wall time.
/// Throughput derives from measured tokens and time. Peak memory
/// is the kernel VmHWM high-water mark sampled after the run.
pub struct MeasuredRun {
    pub run: crate::evalsuite::RunMeasurement,
    pub receipt_sha256: String,
}

fn peak_rss_bytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
            return Some(kb.saturating_mul(1024));
        }
    }
    None
}

/// Run base and cand arms over repo subsamples of the trace and
/// measure every run (v1-release 02 S1). The base arm is the
/// honest-negative control (theta 10.0): its near-zero correctness
/// is the expected control outcome, not a value finding. No
/// promotion gate runs, so every `promoted` is false by absence.
pub fn measure_deterministic_arms(
    trace_text: &str,
    envelope: crate::evalsuite::Envelope,
) -> Result<
    (
        crate::evalsuite::ArmMeasurement,
        crate::evalsuite::ArmMeasurement,
        Vec<String>,
        Vec<String>,
    ),
    String,
> {
    let mut base_runs = Vec::new();
    let mut cand_runs = Vec::new();
    for k in 0..crate::campaign::BATCH_REPOS {
        let sub = crate::campaign::subsample(trace_text, k);
        for (arm, theta, runs) in [
            ("base", crate::campaign::BASE_THETA, &mut base_runs),
            (
                "cand",
                crate::campaign::CAND_THETA_BASE + 0.01 * k as f64,
                &mut cand_runs,
            ),
        ] {
            let tokens = crate::knowledge::tokenize(&sub).len() as u64;
            let start = std::time::Instant::now();
            let receipt = crate::missionrun::run(&sub, theta, None)
                .map_err(|e| format!("{arm} repo {k}: {e}"))?;
            let latency_ns = start.elapsed().as_nanos().max(1) as f64;
            let value = receipt.interval.0 - theta;
            runs.push(MeasuredRun {
                run: crate::evalsuite::RunMeasurement {
                    correct: value > 0.0,
                    promoted: false,
                    tokens,
                    cost: 0.0,
                    latency_ms: (latency_ns / 1_000_000.0).ceil() as u64,
                    throughput: tokens as f64 * 1_000_000_000.0 / latency_ns,
                    peak_memory_bytes: peak_rss_bytes().unwrap_or(0),
                },
                receipt_sha256: receipt.receipt_sha256,
            });
        }
    }
    let to_arm = |id: &str, runs: Vec<MeasuredRun>| crate::evalsuite::ArmMeasurement {
        arm_id: id.to_string(),
        envelope: envelope.clone(),
        runs: runs.into_iter().map(|m| m.run).collect(),
    };
    let base_receipts: Vec<String> = base_runs.iter().map(|m| m.receipt_sha256.clone()).collect();
    let cand_receipts: Vec<String> = cand_runs.iter().map(|m| m.receipt_sha256.clone()).collect();
    Ok((
        to_arm("base", base_runs),
        to_arm("cand", cand_runs),
        base_receipts,
        cand_receipts,
    ))
}

/// Deterministic comparison report over measured arms plus the
/// explicit model-arm absence record (v1-release 02). Measured
/// latency, throughput, and memory vary run to run; correctness, tokens,
/// cost, and promotion counts replay exactly.
pub fn render_baseline_report(
    comparison: &crate::evalsuite::EfficiencyComparison,
    base: &crate::evalsuite::ArmMeasurement,
    cand: &crate::evalsuite::ArmMeasurement,
    base_receipts: &[String],
    cand_receipts: &[String],
) -> String {
    let mut out = String::new();
    out.push_str("# Baseline comparison report - deterministic arms\n\n");
    out.push_str("Arms compare at matched envelopes: identical compute budget,\ntool access, and model access. The base arm is the honest-negative\ncontrol; its low correctness is expected, not a finding.\nNo promotion gate ran: false promotions reads 0 by absence.\nCorrectness, tokens, cost, and promotion counts replay exactly;\nlatency, throughput, and memory are measured per run and vary.\n\n");
    out.push_str(&format!(
        "baseline {}: correctness {:.6}, tokens: {}, cost {:.1}, latency total {} ms, throughput mean {:.3} tok/s, peak memory max {} bytes\n",
        comparison.baseline_id,
        comparison.baseline_correctness,
        comparison.baseline_tokens,
        comparison.baseline_cost,
        comparison.baseline_latency_ms,
        comparison.baseline_throughput,
        comparison.baseline_peak_memory_bytes,
    ));
    out.push_str(&format!(
        "candidate {}: correctness {:.6}, tokens: {}, cost {:.1}, latency total {} ms, throughput mean {:.3} tok/s, peak memory max {} bytes\n",
        comparison.candidate_id,
        comparison.candidate_correctness,
        comparison.candidate_tokens,
        comparison.candidate_cost,
        comparison.candidate_latency_ms,
        comparison.candidate_throughput,
        comparison.candidate_peak_memory_bytes,
    ));
    out.push_str(&format!(
        "\nfalse promotions: {} (no gate ran; count by absence).\n",
        comparison.false_promotions
    ));
    out.push_str("\n## Method notes\n\n");
    out.push_str("- correctness: positive-margin rate on the fixture trace (actual runs).\n- tokens: fixed-tokenizer counts over bytes actually run, not provider metering;\n  counting runs outside the timed section, so throughput divides tokens by chain time only.\n- provider spend: 0 - no metered provider exists in this path (absence, not a finding).\n- latency: local wall time per run in nanoseconds, each run rounded up to whole ms, then summed.\n- throughput: measured tokens per measured second from true nanoseconds;\n  the latency total sums ceiled ms instead, so the two bases differ by rounding.\n- peak memory: process-wide kernel VmHWM high-water sampled after each run, maxed\n  (0 where unreadable); the mark never decreases, so per-run values are max-so-far\n  with no per-run isolation, and the max equals the final sample.\n");
    out.push_str(&format!(
        "\n## Runs\n\nbase runs: {}; candidate runs: {}.\n",
        base.runs.len(),
        cand.runs.len()
    ));
    out.push_str("\n## Run receipts\n\nEach digest binds the subsampled input bytes only (not theta or arm),\nso base and cand runs over one subsample share the digest by construction.\n\n");
    for (i, r) in base_receipts.iter().enumerate() {
        out.push_str(&format!("- base {i}: {r}\n"));
    }
    for (i, r) in cand_receipts.iter().enumerate() {
        out.push_str(&format!("- cand {i}: {r}\n"));
    }
    out.push_str("\n## Model arms: absent\n\nNo consented provider contract exists, so no model arm ran.\nConsequence: this report backs no model-vs-deterministic claim.\nRerun with consented arms to extend it; this file then regenerates.\n");
    out
}
