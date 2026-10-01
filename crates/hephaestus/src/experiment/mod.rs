//! Experiment Compiler (T-019, MASTER_SPEC section 13).
//!
//! Compiles the cheapest authorized experiment able to resolve the
//! relevant uncertainty: plans bind all §13:255 fields immutably before
//! confirmatory results are accessed. R-040 gates predeclared analysis +
//! stopping rule; R-096 gates completeness; R-094/R-095 bind the
//! evaluator digest from the protected context. Discrimination matrix
//! maps predictions to mechanism / alternative / artifact with no
//! invented probabilities. Blockers are precise, never imagined outcomes.

pub mod record;

pub use record::{
    AnalysisSpec, Blocker, CompletenessDenial, Controls, DigestMismatch, ExperimentPlan,
    StoppingRule, Verdict,
};

use serde::{Deserialize, Serialize};

/// What compile needs: hypothesis identity + the uncertainty to resolve.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompileRequest {
    pub hypothesis_version: String,
    pub claim_ids: Vec<String>,
    pub uncertainty: String,
    /// Evaluator digest from the PROTECTED context (R-095) - supplied by
    /// the caller's trusted path, never read from candidate artifacts.
    pub protected_evaluator_digest: String,
    pub resource_limit: String,
    pub authorization_scope: String,
}

/// The remaining plan inputs beyond the request: R-040 requires the
/// analysis spec and stopping rule (None fails compilation); the rest
/// feed §13:255 fields.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlanInputs {
    pub analysis: Option<AnalysisSpec>,
    pub stopping_rule: Option<StoppingRule>,
    pub endpoints: Vec<String>,
    pub guardrails: Vec<String>,
    pub sampling_unit: Option<String>,
    pub comparator: Option<String>,
    pub controls: Controls,
}

/// Compile an experiment plan (§13:251, §13:255).
///
/// R-040: analysis spec and stopping rule are REQUIRED - absent inputs
/// fail compilation. Returns precise blockers for unavailable experiments;
/// never an imagined outcome.
pub fn compile(request: &CompileRequest, inputs: PlanInputs) -> Result<ExperimentPlan, Blocker> {
    // §13:262: precise blockers instead of imagined outcomes. Scope and
    // budget blockers live here; completeness gaps (endpoints, controls,
    // unit, guardrails) are compilable but DENIED at validate (R-096) —
    // an incomplete plan exists, it just cannot qualify.
    if request.authorization_scope.is_empty() {
        return Err(Blocker::ForbiddenAction);
    }
    if request.resource_limit == "0" || request.resource_limit.is_empty() {
        return Err(Blocker::ResourceOverrun);
    }
    // R-040: predeclared analysis + valid stopping rule are plan fields.
    let analysis = inputs.analysis.ok_or(Blocker::MissingInstrument)?;
    let stopping_rule = inputs.stopping_rule.ok_or(Blocker::MissingInstrument)?;
    Ok(ExperimentPlan {
        hypothesis_version: request.hypothesis_version.clone(),
        claim_ids: request.claim_ids.clone(),
        operating_conditions: request.uncertainty.clone(),
        comparator: inputs.comparator.unwrap_or_else(|| "baseline".to_string()),
        intervention_artifact: "candidate".to_string(),
        measurement_procedure: "recorded-procedure".to_string(),
        units: "unspecified".to_string(),
        primary_endpoints: inputs.endpoints,
        guardrails: inputs.guardrails,
        sampling_unit: inputs.sampling_unit.unwrap_or_default(),
        analysis,
        stopping_rule,
        evaluator_digest: request.protected_evaluator_digest.clone(),
        resource_limit: request.resource_limit.clone(),
        authorization_scope: request.authorization_scope.clone(),
        controls: inputs.controls,
    })
}

/// Completeness validation (R-096/AT-096): qualification requires
/// complete primary endpoint, control, unit, and guardrail coverage -
/// each gap named.
pub fn validate(plan: &ExperimentPlan) -> Verdict {
    let mut denials = Vec::new();
    if plan.primary_endpoints.is_empty() {
        denials.push(CompletenessDenial::MissingPrimaryEndpoint);
    }
    if plan.controls.negative.is_none() && plan.controls.positive.is_none() {
        denials.push(CompletenessDenial::MissingControl);
    }
    if plan.sampling_unit.is_empty() {
        denials.push(CompletenessDenial::MissingSamplingUnit);
    }
    if plan.guardrails.is_empty() {
        denials.push(CompletenessDenial::MissingGuardrail);
    }
    if denials.is_empty() {
        Verdict::Complete
    } else {
        Verdict::Incomplete(denials)
    }
}

/// Digest check (R-094/R-095): the plan's evaluator digest must match
/// the protected context's. Mismatch -> rejected.
pub fn check_digest(plan: &ExperimentPlan, protected_digest: &str) -> Result<(), DigestMismatch> {
    if plan.evaluator_digest == protected_digest {
        Ok(())
    } else {
        Err(DigestMismatch {
            plan_digest: plan.evaluator_digest.clone(),
            protected_digest: protected_digest.to_string(),
        })
    }
}

/// One discrimination-matrix row (§13:257): a predicted observation
/// mapped to its target — the proposed mechanism, the strongest relevant
/// alternative, or plausible artifact/null behavior. Text only: exact
/// probabilities are optional and are never invented.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixRow {
    /// "proposed_mechanism" | "strongest_alternative" | "artifact_or_null"
    pub target: String,
    pub prediction: String,
    /// True when this row's prediction is indistinguishable from the
    /// mechanism row's — the test cannot establish the mechanism's
    /// distinct contribution.
    pub cannot_discriminate: bool,
}

/// Build the discrimination matrix (§13:257): three fixed rows in
/// target order; `cannot_discriminate` flags any alternative prediction
/// identical to the mechanism's.
pub fn discrimination_matrix(
    plan: &ExperimentPlan,
    mechanism_prediction: &str,
    alternative_prediction: &str,
    artifact_prediction: &str,
) -> Vec<MatrixRow> {
    let _ = plan;
    let rows = [
        ("proposed_mechanism", mechanism_prediction),
        ("strongest_alternative", alternative_prediction),
        ("artifact_or_null", artifact_prediction),
    ];
    let mechanism_text = mechanism_prediction.trim();
    rows.into_iter()
        .map(|(target, prediction)| MatrixRow {
            target: target.to_string(),
            prediction: prediction.to_string(),
            cannot_discriminate: target != "proposed_mechanism"
                && prediction.trim() == mechanism_text
                && !mechanism_text.is_empty(),
        })
        .collect()
}
