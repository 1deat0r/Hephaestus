//! Evaluation suite contract layer (T-026, R-074/R-075).
//!
//! Typed baseline arms with per-arm envelopes; matched-envelope
//! enforcement (R-074: comparisons at matched budgets); single-ingredient
//! ablations; R-075 result fields kept separate. Model arms are typed and
//! enforced but never simulated — execution needs provider integrations
//! behind capability contracts.

pub mod record;

pub use record::{
    AblationKind, AblationSpec, ArmError, ArmKind, ArmResult, BaselineArm, Envelope, Mismatch,
    MismatchReport,
};

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The suite: registered arms (append-only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EvalSuite {
    arms: Vec<BaselineArm>,
}

/// Define (register) a baseline arm: duplicate ids refused; zero budgets
/// refused (a comparison at zero budget measures nothing).
pub fn define_baseline(suite: &mut EvalSuite, arm: BaselineArm) -> Result<(), ArmError> {
    if arm.envelope.compute_budget == 0 {
        return Err(ArmError::ZeroBudget);
    }
    if suite.arms.iter().any(|a| a.id == arm.id) {
        return Err(ArmError::DuplicateArmId);
    }
    suite.arms.push(arm);
    Ok(())
}

/// Enforce matched budgets and access across ALL arms (R-074, campaign:
/// same approved envelope). Every inequality is named per arm+resource.
pub fn check_matched(suite: &EvalSuite) -> Result<(), MismatchReport> {
    let mut report = MismatchReport::default();
    let Some(first) = suite.arms.first() else {
        return Ok(());
    };
    let ref_env = &first.envelope;
    let mut tool_sets: BTreeSet<BTreeSet<String>> = BTreeSet::new();
    for arm in &suite.arms {
        let env = &arm.envelope;
        if env.compute_budget != ref_env.compute_budget {
            report.mismatches.push(Mismatch {
                arm_id: arm.id.clone(),
                resource: "compute_budget".to_string(),
                detail: format!(
                    "{} vs reference {}",
                    env.compute_budget, ref_env.compute_budget
                ),
            });
        }
        if env.model_access != ref_env.model_access {
            report.mismatches.push(Mismatch {
                arm_id: arm.id.clone(),
                resource: "model_access".to_string(),
                detail: format!("{} vs reference {}", env.model_access, ref_env.model_access),
            });
        }
        tool_sets.insert(env.tool_access.iter().cloned().collect());
        if env.tuning_record.is_empty() {
            report.mismatches.push(Mismatch {
                arm_id: arm.id.clone(),
                resource: "tuning_record".to_string(),
                detail: "tuning/engineering time must be recorded".to_string(),
            });
        }
    }
    if tool_sets.len() > 1 {
        for arm in &suite.arms {
            let set: BTreeSet<String> = arm.envelope.tool_access.iter().cloned().collect();
            if set != *tool_sets.iter().next().expect("nonempty") {
                report.mismatches.push(Mismatch {
                    arm_id: arm.id.clone(),
                    resource: "tool_access".to_string(),
                    detail: "tool access set differs from reference".to_string(),
                });
            }
        }
    }
    if report.mismatches.is_empty() {
        Ok(())
    } else {
        Err(report)
    }
}

/// Build a single-ingredient ablation (campaign: removes the candidate's
/// claimed causal ingredient). An ablation of an ingredient the arm does
/// not have is refused — you cannot remove what is absent.
pub fn ablation(arm: &BaselineArm, kind: AblationKind) -> Result<AblationSpec, String> {
    let ingredient_present = match kind {
        AblationKind::RemoveOptionalDecisionModel => arm
            .envelope
            .tool_access
            .iter()
            .any(|t| t.contains("decision-model")),
        _ => true, // graph/review/decomposition are arm-structural
    };
    if !ingredient_present {
        return Err(format!(
            "arm {} has no decision-model ingredient to ablate",
            arm.id
        ));
    }
    Ok(AblationSpec {
        of_arm_id: arm.id.clone(),
        kind,
    })
}

pub use record::{BenchmarkError, BenchmarkSession, OriginatedHypothesis};

/// Campaign-evaluator admission (R-073/AT-073): the full benchmark
/// never runs without supplied hypotheses, and every admitted
/// hypothesis carries its own opportunity + mechanism lineage — the
/// generator is evaluated on independently originated inputs, not on
/// an empty or seeded-only run.
pub fn begin_benchmark(
    hypotheses: &[record::OriginatedHypothesis],
) -> Result<record::BenchmarkSession, record::BenchmarkError> {
    if hypotheses.is_empty() {
        return Err(record::BenchmarkError::NoSuppliedHypotheses);
    }
    for h in hypotheses {
        if h.opportunity_id.is_empty() || h.mechanism_id.is_empty() {
            return Err(record::BenchmarkError::NotIndependentlyOriginated {
                hypothesis_id: h.hypothesis_id.clone(),
            });
        }
    }
    Ok(record::BenchmarkSession {
        hypothesis_ids: hypotheses.iter().map(|h| h.hypothesis_id.clone()).collect(),
    })
}

/// One measured run (AC 21): correctness, promotion decision, and
/// resource use. All quantities use one unit per comparison; missing
/// measures never enter — runs carry complete data or refuse.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunMeasurement {
    pub correct: bool,
    pub promoted: bool,
    pub tokens: u64,
    pub cost: f64,
    pub latency_ms: u64,
    pub throughput: f64,
    pub peak_memory_bytes: u64,
}

/// One measured arm: its envelope plus its runs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArmMeasurement {
    pub arm_id: String,
    pub envelope: Envelope,
    pub runs: Vec<RunMeasurement>,
}

/// Baseline-vs-candidate comparison at matched envelopes (AC 21).
/// Correctness is the fraction correct. Tokens, cost, and latency
/// total. Throughput means. Peak memory maxes. False promotions
/// count candidate runs promoted while incorrect.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EfficiencyComparison {
    pub baseline_id: String,
    pub candidate_id: String,
    pub baseline_correctness: f64,
    pub candidate_correctness: f64,
    pub false_promotions: usize,
    pub baseline_tokens: u64,
    pub candidate_tokens: u64,
    pub baseline_cost: f64,
    pub candidate_cost: f64,
    pub baseline_latency_ms: u64,
    pub candidate_latency_ms: u64,
    pub baseline_throughput: f64,
    pub candidate_throughput: f64,
    pub baseline_peak_memory_bytes: u64,
    pub candidate_peak_memory_bytes: u64,
}

/// Why a comparison refused.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonError {
    /// The envelopes differ on the named resource.
    UnmatchedEnvelopes { resource: String, detail: String },
    /// One arm carries no runs.
    EmptyRuns { arm_id: String },
    /// One run carries an unusable measure.
    InvalidMeasurement { arm_id: String, reason: String },
}

impl std::fmt::Display for ComparisonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnmatchedEnvelopes { resource, detail } => {
                write!(f, "unmatched envelopes on {resource}: {detail}")
            }
            Self::EmptyRuns { arm_id } => {
                write!(f, "arm {arm_id} carries no runs")
            }
            Self::InvalidMeasurement { arm_id, reason } => {
                write!(f, "arm {arm_id} carries an invalid measure: {reason}")
            }
        }
    }
}

impl std::error::Error for ComparisonError {}

fn envelopes_match(a: &Envelope, b: &Envelope) -> Option<(String, String)> {
    if a.compute_budget != b.compute_budget {
        return Some((
            "compute_budget".to_string(),
            format!("{} vs {}", a.compute_budget, b.compute_budget),
        ));
    }
    let set_a: BTreeSet<&String> = a.tool_access.iter().collect();
    let set_b: BTreeSet<&String> = b.tool_access.iter().collect();
    if set_a != set_b {
        return Some((
            "tool_access".to_string(),
            "tool access sets differ".to_string(),
        ));
    }
    if a.model_access != b.model_access {
        return Some((
            "model_access".to_string(),
            format!("{} vs {}", a.model_access, b.model_access),
        ));
    }
    None
}

fn check_runs(arm: &ArmMeasurement) -> Result<(), ComparisonError> {
    if arm.runs.is_empty() {
        return Err(ComparisonError::EmptyRuns {
            arm_id: arm.arm_id.clone(),
        });
    }
    for run in &arm.runs {
        if !run.cost.is_finite() || run.cost < 0.0 {
            return Err(ComparisonError::InvalidMeasurement {
                arm_id: arm.arm_id.clone(),
                reason: "cost must be finite and non-negative".to_string(),
            });
        }
        if !run.throughput.is_finite() || run.throughput < 0.0 {
            return Err(ComparisonError::InvalidMeasurement {
                arm_id: arm.arm_id.clone(),
                reason: "throughput must be finite and non-negative".to_string(),
            });
        }
    }
    Ok(())
}

fn summarize(runs: &[RunMeasurement]) -> (f64, u64, f64, u64, f64, u64) {
    let n = runs.len() as f64;
    let correct = runs.iter().filter(|r| r.correct).count() as f64 / n;
    let tokens = runs.iter().map(|r| r.tokens).sum();
    let cost = runs.iter().map(|r| r.cost).sum();
    let latency = runs.iter().map(|r| r.latency_ms).sum();
    let throughput = runs.iter().map(|r| r.throughput).sum::<f64>() / n;
    let peak = runs.iter().map(|r| r.peak_memory_bytes).max().unwrap_or(0);
    (correct, tokens, cost, latency, throughput, peak)
}

/// Compare baseline and candidate at matched envelopes (AC 21).
///
/// Envelopes must match on budget, tool access, and model access;
/// both arms must carry runs with usable measures. Order of runs
/// never affects the outcome.
pub fn compare_qualified_efficiency(
    baseline: &ArmMeasurement,
    candidate: &ArmMeasurement,
) -> Result<EfficiencyComparison, ComparisonError> {
    if let Some((resource, detail)) = envelopes_match(&baseline.envelope, &candidate.envelope) {
        return Err(ComparisonError::UnmatchedEnvelopes { resource, detail });
    }
    check_runs(baseline)?;
    check_runs(candidate)?;
    let (b_correct, b_tokens, b_cost, b_lat, b_thr, b_peak) = summarize(&baseline.runs);
    let (c_correct, c_tokens, c_cost, c_lat, c_thr, c_peak) = summarize(&candidate.runs);
    let false_promotions = candidate
        .runs
        .iter()
        .filter(|r| r.promoted && !r.correct)
        .count();
    Ok(EfficiencyComparison {
        baseline_id: baseline.arm_id.clone(),
        candidate_id: candidate.arm_id.clone(),
        baseline_correctness: b_correct,
        candidate_correctness: c_correct,
        false_promotions,
        baseline_tokens: b_tokens,
        candidate_tokens: c_tokens,
        baseline_cost: b_cost,
        candidate_cost: c_cost,
        baseline_latency_ms: b_lat,
        candidate_latency_ms: c_lat,
        baseline_throughput: b_thr,
        candidate_throughput: c_thr,
        baseline_peak_memory_bytes: b_peak,
        candidate_peak_memory_bytes: c_peak,
    })
}
