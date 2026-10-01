//! Independent evaluator + typed result interpreter (T-022, MASTER_SPEC
//! section 14).
//!
//! Interprets computed measurement intervals into typed results that
//! preserve the three separate result dimensions (§14:280), apply the
//! threshold semantics (§14:281), and never represent "proven true"
//! (§14:282). Guardrails are evaluated independently (roadmap line).
//! Integration: requires a frozen plan's evaluator digest (R-095) and a
//! registry-qualified method (R-100); otherwise execution is Invalid
//! with a named reason.

pub mod record;

pub use record::{
    EngineeringTarget, ExecutionValidity, GuardrailClass, GuardrailGap, GuardrailOutcome,
    GuardrailReport, Measurements, ScientificConclusion, ThresholdProvenance, TypedResult,
};

use crate::experiment::ExperimentPlan;
use crate::methods::ProtectedRegistry;

/// What the evaluator needs beyond measurements: the frozen plan, the
/// protected evaluator digest, and the qualified method registry.
pub struct InterpretInputs<'a> {
    pub plan: &'a ExperimentPlan,
    pub protected_evaluator_digest: &'a str,
    pub registry: &'a ProtectedRegistry,
    /// Qualified method name+version used to compute the interval.
    pub method_name: &'a str,
    pub method_version: &'a str,
}

/// Interpret measurements into a typed result (§14:280-282).
pub fn interpret(inputs: &InterpretInputs<'_>, m: &Measurements) -> TypedResult {
    let provenance = ThresholdProvenance {
        estimand: inputs.plan.analysis.method.clone(),
        units: inputs.plan.units.clone(),
        comparator: inputs.plan.comparator.clone(),
        threshold_source: format!(
            "frozen plan for hypothesis {}",
            inputs.plan.hypothesis_version
        ),
    };
    let claim = inputs.plan.claim_ids.first().cloned().unwrap_or_default();
    let conditions = inputs.plan.operating_conditions.clone();

    // Integration gates (R-095, R-100): evaluator digest and qualified
    // method. Failure is INVALID EXECUTION, not a weak conclusion.
    if inputs.plan.evaluator_digest != inputs.protected_evaluator_digest {
        return TypedResult {
            execution: ExecutionValidity::Invalid,
            science: ScientificConclusion::NotAssessed,
            engineering: EngineeringTarget::NotAssessed,
            claim,
            conditions,
            provenance,
            invalid_reason: Some("evaluator digest mismatch".to_string()),
        };
    }
    if inputs
        .registry
        .qualified(inputs.method_name, inputs.method_version)
        .is_none()
    {
        return TypedResult {
            execution: ExecutionValidity::Invalid,
            science: ScientificConclusion::NotAssessed,
            engineering: EngineeringTarget::NotAssessed,
            claim,
            conditions,
            provenance,
            invalid_reason: Some("method not registry-qualified".to_string()),
        };
    }

    // §14:281: interval vs threshold theta.
    let (science, engineering) = match m.interval {
        None => (
            ScientificConclusion::NotAssessed,
            EngineeringTarget::NotAssessed,
        ),
        Some((low, high)) => {
            let science = if low > m.threshold {
                ScientificConclusion::Supported
            } else if high < m.threshold {
                ScientificConclusion::Contradicted
            } else {
                ScientificConclusion::Inconclusive
            };
            // Engineering target: noninferiority if a quality margin is
            // declared — lower bound of the quality difference must
            // exceed -m (§14:281), not merely a nonsignificant
            // difference. Without a margin, target follows the interval
            // entirely-above-theta rule.
            let engineering = match (m.quality_margin, m.quality_lower_bound) {
                (Some(margin), Some(lb)) if lb > -margin => EngineeringTarget::Met,
                (Some(_), Some(_)) => EngineeringTarget::NotMet,
                _ => {
                    if low > m.threshold {
                        EngineeringTarget::Met
                    } else if high < m.threshold {
                        EngineeringTarget::NotMet
                    } else {
                        EngineeringTarget::Inconclusive
                    }
                }
            };
            (science, engineering)
        }
    };

    TypedResult {
        execution: ExecutionValidity::Valid,
        science,
        engineering,
        claim,
        conditions,
        provenance,
        invalid_reason: None,
    }
}

/// Independent guardrail evaluation: each declared guardrail's measured
/// value vs its limit; failures recorded WITHOUT touching the
/// science/engineering dimensions (interpret produces those).
pub fn check_guardrails(measurements: &[(String, f64, f64)]) -> GuardrailReport {
    let outcomes: Vec<GuardrailOutcome> = measurements
        .iter()
        .map(|(name, value, limit)| GuardrailOutcome {
            name: name.clone(),
            value: *value,
            limit: *limit,
            ok: value <= limit,
        })
        .collect();
    let all_pass = outcomes.iter().all(|o| o.ok);
    GuardrailReport { outcomes, all_pass }
}

/// R-104 presence gate: qualification requires ALL FOUR guardrail
/// classes to be DECLARED before results are interpreted
/// (pre-registration continuity). Returns one named gap per missing
/// class; an empty gap list means the declaration is complete. The
/// evaluation SEMANTICS stay in check_guardrails.
pub fn require_guardrails(declared: &[GuardrailClass]) -> Vec<GuardrailGap> {
    let required = [
        GuardrailClass::ScopedError,
        GuardrailClass::Correctness,
        GuardrailClass::PrecisionPower,
        GuardrailClass::Noninferiority,
    ];
    required
        .iter()
        .filter(|c| !declared.contains(c))
        .map(|c| GuardrailGap { missing: *c })
        .collect()
}
