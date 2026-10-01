//! Experiment Compiler (T-019, R-040, R-096, R-094/095, MASTER_SPEC §13).
//!
//! Red-first integration tests at the public seam: `compile`, `validate`,
//! `check_digest`, `discrimination_matrix`.

use hephaestus::experiment::record::{
    AnalysisSpec, Blocker, CompletenessDenial, Controls, StoppingRule, Verdict,
};
use hephaestus::experiment::{
    CompileRequest, check_digest, compile, discrimination_matrix, validate,
};

fn s(v: &str) -> String {
    v.to_string()
}

fn request() -> CompileRequest {
    CompileRequest {
        hypothesis_version: s("h1.0.0"),
        claim_ids: vec![s("C1"), s("C2")],
        uncertainty: s("does mtime invalidation reduce rebuild time"),
        protected_evaluator_digest: s("a1b2c3"),
        resource_limit: s("2 CPU-hours"),
        authorization_scope: s("local-fixture"),
    }
}

fn analysis() -> AnalysisSpec {
    AnalysisSpec {
        method: s("paired comparison"),
        decision_rule: s("improvement if median delta > 0 across repeats"),
    }
}

fn controls() -> Controls {
    Controls {
        positive: Some(s("warm full rebuild")),
        negative: Some(s("no-op intervention")),
        randomization: true,
        repeatability_checks: true,
        environmental_capture: true,
        missing_observation_plan: Some(s("rerun failed trials once; record gaps")),
    }
}

fn full_inputs() -> hephaestus::experiment::PlanInputs {
    hephaestus::experiment::PlanInputs {
        analysis: Some(analysis()),
        stopping_rule: Some(StoppingRule::FixedSampleCount(30)),
        endpoints: vec![s("rebuild_seconds")],
        guardrails: vec![s("no stale reads")],
        sampling_unit: Some(s("individual rebuild run")),
        comparator: Some(s("cold-cache baseline")),
        controls: controls(),
    }
}

// ---- Ticket 01: compile, validate, digest, blockers ----

#[test]
fn compile_binds_all_section_255_fields() {
    let plan = compile(&request(), full_inputs()).expect("compiles");
    assert_eq!(plan.hypothesis_version, "h1.0.0", "frozen version (R-097)");
    assert_eq!(plan.claim_ids, vec!["C1", "C2"]);
    assert!(!plan.comparator.is_empty());
    assert!(!plan.intervention_artifact.is_empty());
    assert!(!plan.measurement_procedure.is_empty());
    assert!(!plan.units.is_empty());
    assert_eq!(plan.primary_endpoints, vec!["rebuild_seconds"]);
    assert_eq!(plan.guardrails, vec!["no stale reads"]);
    assert_eq!(plan.sampling_unit, "individual rebuild run");
    assert_eq!(
        plan.analysis.decision_rule,
        "improvement if median delta > 0 across repeats"
    );
    assert_eq!(plan.stopping_rule, StoppingRule::FixedSampleCount(30));
    assert_eq!(plan.evaluator_digest, "a1b2c3");
    assert_eq!(plan.resource_limit, "2 CPU-hours");
    assert_eq!(plan.authorization_scope, "local-fixture");
    assert!(plan.controls.randomization);
}

#[test]
fn missing_analysis_or_stopping_rule_fails_compilation() {
    // R-040: predeclared analysis + valid stopping rule are required.
    let mut inputs = full_inputs();
    inputs.analysis = None;
    assert!(
        compile(&request(), inputs.clone()).is_err(),
        "analysis spec required"
    );
    let mut inputs = full_inputs();
    inputs.stopping_rule = None;
    assert!(
        compile(&request(), inputs).is_err(),
        "stopping rule required"
    );
}

#[test]
fn validate_names_completeness_denials() {
    // R-096: each gap named.
    let plan = compile(
        &request(),
        hephaestus::experiment::PlanInputs {
            analysis: Some(analysis()),
            stopping_rule: Some(StoppingRule::FixedSampleCount(5)),
            endpoints: vec![],
            guardrails: vec![],
            sampling_unit: None,
            comparator: None,
            controls: Controls::default(),
        },
    )
    .expect("compiles structurally");
    match validate(&plan) {
        Verdict::Incomplete(denials) => {
            assert!(denials.contains(&CompletenessDenial::MissingPrimaryEndpoint));
            assert!(denials.contains(&CompletenessDenial::MissingControl));
            assert!(denials.contains(&CompletenessDenial::MissingSamplingUnit));
            assert!(denials.contains(&CompletenessDenial::MissingGuardrail));
        }
        Verdict::Complete => panic!("empty plan must not qualify (R-096)"),
    }
    // Complete plan passes.
    let complete = compile(&request(), full_inputs()).unwrap();
    assert_eq!(validate(&complete), Verdict::Complete);
}

#[test]
fn digest_mismatch_rejected() {
    let plan = compile(&request(), full_inputs()).unwrap();
    assert!(check_digest(&plan, "a1b2c3").is_ok());
    let err = check_digest(&plan, "zzz").expect_err("mismatch rejected");
    assert_eq!(err.plan_digest, "a1b2c3");
    assert_eq!(err.protected_digest, "zzz");
}

#[test]
fn blockers_are_precise_never_imagined() {
    // §13:262: precise blocker instead of an imagined result.
    let no_scope = CompileRequest {
        authorization_scope: String::new(),
        ..request()
    };
    assert_eq!(
        compile(&no_scope, full_inputs()),
        Err(Blocker::ForbiddenAction)
    );
    let no_budget = CompileRequest {
        resource_limit: s("0"),
        ..request()
    };
    assert_eq!(
        compile(&no_budget, full_inputs()),
        Err(Blocker::ResourceOverrun)
    );
}

// ---- Ticket 02: discrimination matrix, alternatives ----

#[test]
fn matrix_maps_three_prediction_targets() {
    let plan = compile(&request(), full_inputs()).unwrap();
    let matrix = discrimination_matrix(
        &plan,
        "mtime lookups shrink the invalidation set",
        "warmed cache makes rebuilds fast regardless",
        "measurement noise on cold caches",
    );
    assert_eq!(matrix.len(), 3, "mechanism / alternative / artifact rows");
    assert_eq!(matrix[0].target, "proposed_mechanism");
    assert_eq!(matrix[1].target, "strongest_alternative");
    assert_eq!(matrix[2].target, "artifact_or_null");
    // No invented probabilities: prediction text only.
    assert!(matrix.iter().all(|r| !r.prediction.contains('%')));
}

#[test]
fn indistinguishable_predictions_flagged() {
    let plan = compile(&request(), full_inputs()).unwrap();
    let matrix = discrimination_matrix(
        &plan,
        "rebuild is faster",
        "rebuild is faster", // identical prediction: cannot discriminate
        "no change",
    );
    assert!(
        matrix.iter().any(|r| r.cannot_discriminate),
        "identical mechanism/alternative predictions are flagged (§13:257)"
    );
}

#[test]
fn twin_run_byte_identical() {
    let a = compile(&request(), full_inputs()).unwrap();
    let b = compile(&request(), full_inputs()).unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
