//! Advanced search evaluation (T-031).
//!
//! Integration tests at the public seam: `matched_ablation`,
//! `evaluate_pair`.

use hephaestus::acceleration::record::{MechanismKind, PairMeasurement, PromotionVerdict};
use hephaestus::acceleration::{CandidateMechanism, evaluate_pair, matched_ablation};

fn s(v: &str) -> String {
    v.to_string()
}

fn mechanism(available: bool, disablable: bool) -> CandidateMechanism {
    CandidateMechanism {
        kind: MechanismKind::GraphRetrieval,
        id: s("graph-retrieval-1"),
        disablable,
        compute_budget: 100,
        tool_access: vec![s("search"), s("files")],
        infrastructure_available: available,
    }
}

// ---- Ticket 01: mechanisms + matched pairs ----

#[test]
fn six_mechanism_kinds_typed() {
    let all = [
        MechanismKind::GraphRetrieval,
        MechanismKind::VectorSearch,
        MechanismKind::GpuWorkers,
        MechanismKind::MonteCarloTreeSearch,
        MechanismKind::QualityDiversitySearch,
        MechanismKind::LearnedAllocation,
    ];
    assert_eq!(all.len(), 6);
}

#[test]
fn matched_ablation_enforces_identical_envelopes() {
    let m = mechanism(true, true);
    // Identical envelope: pair built.
    assert!(matched_ablation(&m, 100, &[s("search"), s("files")]).is_ok());
    // Budget differs: refused (not a matched ablation).
    assert!(matched_ablation(&m, 50, &[s("search"), s("files")]).is_err());
    // Tool access differs: refused.
    assert!(matched_ablation(&m, 100, &[s("search")]).is_err());
}

// ---- Ticket 02: promotion verdicts ----

#[test]
fn promoted_requires_value_guardrails_and_disablability() {
    let m = mechanism(true, true);
    let pair = matched_ablation(&m, 100, &[s("search"), s("files")]).unwrap();
    // Measured improvement + no violations + disablable -> Promoted.
    let good = PairMeasurement {
        with_value: 0.85,
        without_value: 0.60,
        guardrail_violations: 0,
    };
    assert_eq!(
        evaluate_pair(&pair, Some(&good)),
        PromotionVerdict::Promoted
    );
    // No improvement -> Retained (incumbent kept).
    let worse = PairMeasurement {
        with_value: 0.50,
        without_value: 0.60,
        guardrail_violations: 0,
    };
    assert_eq!(
        evaluate_pair(&pair, Some(&worse)),
        PromotionVerdict::Retained
    );
    // Guardrail violations -> Retained regardless of value.
    let violating = PairMeasurement {
        with_value: 0.95,
        without_value: 0.60,
        guardrail_violations: 2,
    };
    assert_eq!(
        evaluate_pair(&pair, Some(&violating)),
        PromotionVerdict::Retained
    );
    // Non-disablable mechanism -> never promoted (core stays functional).
    let locked = mechanism(true, false);
    let locked_pair = matched_ablation(&locked, 100, &[s("search"), s("files")]).unwrap();
    assert_eq!(
        evaluate_pair(&locked_pair, Some(&good)),
        PromotionVerdict::Retained
    );
    // Nothing recorded yet -> Retained (not promotable on no evidence).
    assert_eq!(evaluate_pair(&pair, None), PromotionVerdict::Retained);
}

#[test]
fn absent_infrastructure_is_not_evaluable_not_failure() {
    // No vector DB / GPU available: honest NotEvaluable.
    let absent = mechanism(false, true);
    let pair = matched_ablation(&absent, 100, &[s("search"), s("files")]).unwrap();
    let good = PairMeasurement {
        with_value: 0.9,
        without_value: 0.5,
        guardrail_violations: 0,
    };
    assert_eq!(
        evaluate_pair(&pair, Some(&good)),
        PromotionVerdict::NotEvaluable,
        "fabricated-looking measurements on absent infrastructure are ignored"
    );
    assert_eq!(evaluate_pair(&pair, None), PromotionVerdict::NotEvaluable);
}

#[test]
fn twin_run_byte_identical() {
    let m = mechanism(true, true);
    let a = matched_ablation(&m, 100, &[s("search"), s("files")]).unwrap();
    let b = matched_ablation(&m, 100, &[s("search"), s("files")]).unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
