//! Advanced search evaluation (T-031; T-050 R-083 caching review).
//!
//! Integration tests at the public seam: `matched_ablation`,
//! `evaluate_pair`.

use hephaestus::acceleration::record::{MechanismKind, PairMeasurement, PromotionVerdict};
use hephaestus::acceleration::{
    CachingComparator, CachingComparison, CachingReview, CachingReviewError, CandidateMechanism,
    evaluate_pair, matched_ablation, review_caching_comparison,
};

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

// ---- Ticket 01: R-083 context-caching benchmark review ----

#[test]
fn at_083_full_reconstruction_alone_without_justification_is_refused() {
    // R-083 negative case: use only full reconstruction despite a
    // stronger exact-cache implementation — benchmark review must
    // request the stronger comparator or justify the exclusion.
    let comparison = CachingComparison {
        comparators_used: vec![CachingComparator::FullReconstruction],
        exclusion_justification: None,
    };
    let err = review_caching_comparison(&comparison)
        .expect_err("an unjustified straw-baseline-only review must be refused");
    assert!(
        matches!(
            err,
            CachingReviewError::MissingStrongerComparatorOrJustification
        ),
        "{err:?}"
    );

    // An empty justification is no justification.
    let comparison = CachingComparison {
        comparators_used: vec![CachingComparator::FullReconstruction],
        exclusion_justification: Some(s("")),
    };
    review_caching_comparison(&comparison)
        .expect_err("an empty exclusion justification must be refused");
}

#[test]
fn at_083_using_the_stronger_comparator_satisfies_review() {
    let comparison = CachingComparison {
        comparators_used: vec![
            CachingComparator::FullReconstruction,
            CachingComparator::ExactCache,
        ],
        exclusion_justification: None,
    };
    let outcome = review_caching_comparison(&comparison).expect("stronger comparator used");
    assert!(matches!(outcome, CachingReview::StrongerComparatorUsed));
}

#[test]
fn at_083_a_justified_exclusion_satisfies_review_and_carries_the_reason() {
    let comparison = CachingComparison {
        comparators_used: vec![CachingComparator::FullReconstruction],
        exclusion_justification: Some(s(
            "exact-cache prototype lacks the pinned implementation digest (recorded gap)",
        )),
    };
    let outcome = review_caching_comparison(&comparison).expect("justified exclusion");
    match outcome {
        CachingReview::ExclusionJustified { justification } => {
            assert!(justification.contains("pinned implementation digest"));
        }
        other => panic!("expected ExclusionJustified, got {other:?}"),
    }
}
