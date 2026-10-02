//! Typed result interpreter (T-022, R-026, R-042 (inconclusive results
//! preserved; noninferiority explicit — lower-bound margin test),
//! MASTER_SPEC §14).
//!
//! Integration tests at the public seam: `interpret`, `check_guardrails`.

use hephaestus::evaluation::record::{
    EngineeringTarget, ExecutionValidity, Measurements, ScientificConclusion,
};
use hephaestus::evaluation::{InterpretInputs, check_guardrails, interpret};
use hephaestus::experiment::record::{AnalysisSpec, Controls, ExperimentPlan, StoppingRule};
use hephaestus::methods::ProtectedRegistry;
use hephaestus::methods::record::{ClippingPolicy, Estimand, MethodSpec, MissingnessPolicy};

fn s(v: &str) -> String {
    v.to_string()
}

fn plan() -> ExperimentPlan {
    ExperimentPlan {
        hypothesis_version: s("h1.0.0"),
        claim_ids: vec![s("C1")],
        operating_conditions: s("cold-cache single machine"),
        comparator: s("cold-cache baseline"),
        intervention_artifact: s("candidate"),
        measurement_procedure: s("recorded-procedure"),
        units: s("seconds"),
        primary_endpoints: vec![s("rebuild_seconds")],
        guardrails: vec![s("peak_memory_ratio")],
        sampling_unit: s("individual rebuild run"),
        analysis: AnalysisSpec {
            method: s("paired comparison"),
            decision_rule: s("lower bound above theta"),
        },
        stopping_rule: StoppingRule::FixedSampleCount(30),
        evaluator_digest: s("eval-digest-1"),
        resource_limit: s("2 CPU-hours"),
        authorization_scope: s("local-fixture"),
        controls: Controls::default(),
    }
}

fn qualified_registry() -> ProtectedRegistry {
    let registry = ProtectedRegistry::new();
    registry
        .register(MethodSpec {
            name: s("paired-repo-difference"),
            version: s("1.0.0"),
            estimand: Estimand {
                quantity: s("paired difference"),
                unit: s("repository-level"),
                denominator: s("all missions"),
                cost_fields: vec![],
            },
            assumptions: vec![],
            alpha_allocations: vec![0.025],
            missingness: MissingnessPolicy::FailedOrMissingIsFailure,
            clipping: ClippingPolicy {
                lower: None,
                upper: None,
                report_clipped_rates: true,
            },
            implementation_digest: s("impl-digest-1"),
        })
        .unwrap();
    registry
}

fn inputs<'a>(plan: &'a ExperimentPlan, registry: &'a ProtectedRegistry) -> InterpretInputs<'a> {
    InterpretInputs {
        plan,
        protected_evaluator_digest: "eval-digest-1",
        registry,
        method_name: "paired-repo-difference",
        method_version: "1.0.0",
    }
}

// ---- Ticket 01: interpret ----

#[test]
fn interval_entirely_above_threshold_supports() {
    let registry = qualified_registry();
    let p = plan();
    let m = Measurements {
        interval: Some((0.12, 0.30)), // entirely above theta=0.10
        threshold: 0.10,
        quality_margin: None,
        quality_lower_bound: None,
    };
    let r = interpret(&inputs(&p, &registry), &m);
    assert_eq!(r.execution, ExecutionValidity::Valid);
    assert_eq!(r.science, ScientificConclusion::Supported);
    assert_eq!(r.engineering, EngineeringTarget::Met);
    assert_eq!(r.claim, "C1");
    assert_eq!(r.conditions, "cold-cache single machine");
    // R-026 provenance.
    assert_eq!(r.provenance.units, "seconds");
    assert_eq!(r.provenance.comparator, "cold-cache baseline");
    assert!(r.provenance.threshold_source.contains("h1.0.0"));
}

#[test]
fn interval_entirely_below_contradicts_overlap_inconclusive() {
    let registry = qualified_registry();
    let p = plan();
    let below = Measurements {
        interval: Some((-0.20, -0.02)),
        threshold: 0.10,
        quality_margin: None,
        quality_lower_bound: None,
    };
    assert_eq!(
        interpret(&inputs(&p, &registry), &below).science,
        ScientificConclusion::Contradicted
    );
    let overlap = Measurements {
        interval: Some((0.05, 0.20)), // straddles theta=0.10
        threshold: 0.10,
        quality_margin: None,
        quality_lower_bound: None,
    };
    let r = interpret(&inputs(&p, &registry), &overlap);
    assert_eq!(r.science, ScientificConclusion::Inconclusive);
    assert_eq!(r.engineering, EngineeringTarget::Inconclusive);
}

#[test]
fn noninferiority_uses_lower_bound_vs_negative_margin() {
    let registry = qualified_registry();
    let p = plan();
    // Interval overlaps theta (science inconclusive), but the quality
    // lower bound exceeds -m: engineering MET (§14:281).
    let m = Measurements {
        interval: Some((0.05, 0.20)),
        threshold: 0.10,
        quality_margin: Some(0.05),
        quality_lower_bound: Some(-0.02), // -0.02 > -0.05
    };
    let r = interpret(&inputs(&p, &registry), &m);
    assert_eq!(r.engineering, EngineeringTarget::Met);
    // Lower bound below -m: NOT met (not merely nonsignificant).
    let m = Measurements {
        interval: Some((0.05, 0.20)),
        threshold: 0.10,
        quality_margin: Some(0.05),
        quality_lower_bound: Some(-0.10), // -0.10 < -0.05
    };
    assert_eq!(
        interpret(&inputs(&p, &registry), &m).engineering,
        EngineeringTarget::NotMet
    );
}

#[test]
fn invalid_execution_short_circuits_to_not_assessed() {
    let registry = qualified_registry();
    // Evaluator digest mismatch.
    let p = plan();
    let bad_digest = InterpretInputs {
        protected_evaluator_digest: "WRONG",
        ..inputs(&p, &registry)
    };
    let m = Measurements {
        interval: Some((0.12, 0.30)),
        threshold: 0.10,
        quality_margin: None,
        quality_lower_bound: None,
    };
    let r = interpret(&bad_digest, &m);
    assert_eq!(r.execution, ExecutionValidity::Invalid);
    assert_eq!(r.science, ScientificConclusion::NotAssessed);
    assert_eq!(r.engineering, EngineeringTarget::NotAssessed);
    assert_eq!(
        r.invalid_reason.as_deref(),
        Some("evaluator digest mismatch")
    );
    // Unqualified method.
    let p = plan();
    let unqualified = InterpretInputs {
        method_version: "9.9.9",
        ..inputs(&p, &registry)
    };
    let r = interpret(&unqualified, &m);
    assert_eq!(r.execution, ExecutionValidity::Invalid);
    assert_eq!(
        r.invalid_reason.as_deref(),
        Some("method not registry-qualified")
    );
    // Missing interval: valid execution, but nothing assessed.
    let r = interpret(&inputs(&plan(), &registry), &Measurements::default());
    assert_eq!(r.execution, ExecutionValidity::Valid);
    assert_eq!(r.science, ScientificConclusion::NotAssessed);
}

#[test]
fn no_proven_true_representation_exists() {
    // §14:282: the enum has no True variant — enforced by construction;
    // strongest states enumerated here.
    let all = [
        ScientificConclusion::Supported,
        ScientificConclusion::Contradicted,
        ScientificConclusion::Inconclusive,
        ScientificConclusion::NotAssessed,
    ];
    assert_eq!(all.len(), 4);
}

// ---- Ticket 02: guardrails ----

#[test]
fn guardrails_independent_of_science_dimension() {
    // A failing guardrail does NOT flip a supported science conclusion.
    let registry = qualified_registry();
    let p = plan();
    let m = Measurements {
        interval: Some((0.12, 0.30)),
        threshold: 0.10,
        quality_margin: None,
        quality_lower_bound: None,
    };
    let r = interpret(&inputs(&p, &registry), &m);
    assert_eq!(r.science, ScientificConclusion::Supported);
    // Guardrail: peak memory 1.25 vs limit 1.10 -> FAIL, recorded
    // independently.
    let report = check_guardrails(&[(s("peak_memory_ratio"), 1.25, 1.10)]);
    assert!(!report.all_pass);
    assert!(!report.outcomes[0].ok);
    // The science dimension is untouched by the guardrail failure.
    assert_eq!(r.science, ScientificConclusion::Supported);
    // A passing guardrail records ok.
    let ok = check_guardrails(&[(s("peak_memory_ratio"), 1.05, 1.10)]);
    assert!(ok.all_pass);
}

#[test]
fn twin_run_byte_identical() {
    let registry = qualified_registry();
    let p = plan();
    let m = Measurements {
        interval: Some((0.12, 0.30)),
        threshold: 0.10,
        quality_margin: None,
        quality_lower_bound: None,
    };
    let a = interpret(&inputs(&p, &registry), &m);
    let b = interpret(&inputs(&p, &registry), &m);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
