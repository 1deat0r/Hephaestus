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
