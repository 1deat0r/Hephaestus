//! Scenario-class coverage for the hidden evaluator (T-003): true, false,
//! confounded, impossible, ambiguous, and changed-boundary worlds each
//! produce their designated verdict.

use synthetic_evaluator::{
    Candidate, Form, Regime, Requirement, Scenario, Verdict, World, evaluate,
};

fn world(scenario: Scenario) -> World {
    World {
        id: "W-1".to_string(),
        scenario,
        domain: [10.0, 100.0],
        noise_sigma: 0.3,
        seed: 42,
        truth_form: Form::Inverse,
        truth_a: 20.0,
        truth_b: 500.0,
        causal_effect_of_input: true,
        regimes: Vec::new(),
        requirement: None,
    }
}

fn verdict(candidate: &Candidate, world: &World) -> Verdict {
    let obs = world.sample_observations();
    evaluate(candidate, world, &obs).verdict
}

#[test]
fn true_mechanism_confirms_exact_candidate() {
    let w = world(Scenario::TrueMechanism);
    let exact = Candidate {
        form: Form::Inverse,
        a: 20.0,
        b: 500.0,
    };
    assert_eq!(verdict(&exact, &w), Verdict::Confirmed);
    // Small parameter drift inside the declared tolerance still confirms.
    let drifted = Candidate {
        form: Form::Inverse,
        a: 20.0005,
        b: 500.0,
    };
    assert_eq!(verdict(&drifted, &w), Verdict::Confirmed);
}

#[test]
fn true_mechanism_rejects_lookalike_and_far_params() {
    let w = world(Scenario::TrueMechanism);
    // Wrong form: a line fitted across an inverse curve separates clearly.
    let line = Candidate {
        form: Form::Linear,
        a: 30.0,
        b: 0.4,
    };
    assert_eq!(verdict(&line, &w), Verdict::Rejected);
    // Right form, materially wrong parameters.
    let shifted = Candidate {
        form: Form::Inverse,
        a: 25.0,
        b: 500.0,
    };
    assert_eq!(verdict(&shifted, &w), Verdict::Rejected);
}

#[test]
fn false_mechanism_rejects_beautiful_but_wrong_fit() {
    let mut w = world(Scenario::FalseMechanism);
    w.truth_form = Form::Linear;
    w.truth_a = 20.0;
    w.truth_b = 0.4;
    // The true generator confirms.
    let truth = Candidate {
        form: Form::Linear,
        a: 20.0,
        b: 0.4,
    };
    assert_eq!(verdict(&truth, &w), Verdict::Confirmed);
    // The tempting inverse explanation fits the endpoints well but is not
    // the mechanism — a good-looking fit must not survive.
    let false_mech = Candidate {
        form: Form::Inverse,
        a: 64.0,
        b: -400.0,
    };
    assert_eq!(verdict(&false_mech, &w), Verdict::Rejected);
}

#[test]
fn confounder_rejects_causal_claim_despite_perfect_fit() {
    let mut w = world(Scenario::Confounder);
    w.truth_form = Form::Linear;
    w.truth_a = 10.0;
    w.truth_b = 0.05;
    w.causal_effect_of_input = false;
    // This candidate IS the data-generating curve — and must still be
    // rejected because the causal claim is false.
    let causal = Candidate {
        form: Form::Linear,
        a: 10.0,
        b: 0.05,
    };
    let obs = w.sample_observations();
    let assessment = evaluate(&causal, &w, &obs);
    assert_eq!(assessment.verdict, Verdict::Rejected);
    assert!(
        assessment.reason.contains("confound"),
        "reason must name the confounder: {}",
        assessment.reason
    );
    // A causal inverse claim is equally rejected.
    let inverse = Candidate {
        form: Form::Inverse,
        a: 8.0,
        b: 30.0,
    };
    assert_eq!(verdict(&inverse, &w), Verdict::Rejected);
}

#[test]
fn impossible_world_reports_infeasible_not_mechanism_failure() {
    let mut w = world(Scenario::ImpossibleConstraint);
    w.truth_form = Form::Constant;
    w.truth_a = 12.0;
    w.truth_b = 0.0;
    w.requirement = Some(Requirement { output_max: 10.0 });
    // Even the exact truth candidate cannot satisfy the target: the verdict
    // is infeasibility, never a mechanism conclusion from a missed target.
    let exact = Candidate {
        form: Form::Constant,
        a: 12.0,
        b: 0.0,
    };
    let assessment = evaluate(&exact, &w, &w.sample_observations());
    assert_eq!(assessment.verdict, Verdict::Infeasible);
    // A candidate that itself cannot reach the target is also infeasible.
    let high = Candidate {
        form: Form::Linear,
        a: 13.0,
        b: 0.1,
    };
    assert_eq!(verdict(&high, &w), Verdict::Infeasible);
}

#[test]
fn ambiguous_world_is_inconclusive_not_rejected() {
    let mut w = world(Scenario::AmbiguousEvidence);
    w.domain = [20.0, 30.0];
    w.truth_form = Form::Linear;
    w.truth_a = 10.0;
    w.truth_b = 0.1;
    w.noise_sigma = 0.6;
    // The exact mechanism still confirms structurally.
    let exact = Candidate {
        form: Form::Linear,
        a: 10.0,
        b: 0.1,
    };
    assert_eq!(verdict(&exact, &w), Verdict::Confirmed);
    // An alternative form that is observationally equivalent on this narrow
    // range under this much noise: inconclusive, not rejected.
    let ambiguous = Candidate {
        form: Form::Inverse,
        a: 15.0,
        b: -60.0,
    };
    assert_eq!(verdict(&ambiguous, &w), Verdict::Inconclusive);
}

#[test]
fn changed_boundary_reports_out_of_scope() {
    let mut w = world(Scenario::ChangedBoundary);
    w.regimes = vec![Regime {
        break_at: 55.0,
        form: Form::Linear,
        a: 30.0,
        b: 0.2,
    }];
    // Holds in the first regime only.
    let pre_break = Candidate {
        form: Form::Inverse,
        a: 20.0,
        b: 500.0,
    };
    assert_eq!(verdict(&pre_break, &w), Verdict::OutOfScope);
    // Holds in the second regime only.
    let post_break = Candidate {
        form: Form::Linear,
        a: 30.0,
        b: 0.2,
    };
    assert_eq!(verdict(&post_break, &w), Verdict::OutOfScope);
    // Matches neither regime.
    let neither = Candidate {
        form: Form::Constant,
        a: 40.0,
        b: 0.0,
    };
    assert_eq!(verdict(&neither, &w), Verdict::Rejected);
}

#[test]
fn empty_observations_never_confirm() {
    let w = world(Scenario::TrueMechanism);
    let exact = Candidate {
        form: Form::Inverse,
        a: 20.0,
        b: 500.0,
    };
    let assessment = evaluate(&exact, &w, &[]);
    assert_eq!(assessment.verdict, Verdict::Rejected);
}
