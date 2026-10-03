//! Advisory provider contract layer (T-029, R-005/AT-005 (optional
//! backends disabled: the local reference providers keep the workflow
//! executable — NullProvider/FixtureProvider), R-053/AT-053 (calibration
//! and abstention evaluated from known-outcome fixture cases; a
//! provider with no calibration record gets no automatic authority),
//! R-090).
//!
//! Integration tests at the public seam: `evaluate_provider`,
//! `assert_control_authority`, NullProvider.

use hephaestus::advisory::record::{
    ConfidenceBucket, CostQuantity, KnownOutcomeCase, ProviderIdentity, Recommendation,
};
use hephaestus::advisory::{
    AdvisoryProvider, FixtureProvider, NullProvider, assert_control_authority, evaluate_provider,
};

fn s(v: &str) -> String {
    v.to_string()
}

fn case(id: &str, correct: bool) -> KnownOutcomeCase {
    KnownOutcomeCase {
        case_id: s(id),
        prioritize_was_correct: correct,
    }
}

// ---- Ticket 01: trait + NullProvider + identity ----

// AT-053: unknowns can abstain. With no calibration record the
// provider declines every case — no advisory decision is minted.
#[test]
fn null_provider_always_abstains_core_runs_without_it() {
    let p = NullProvider;
    // R-005: the provider is optional — NullProvider declines everything.
    assert!(p.propose(&case("c1", true)).is_none());
    assert!(p.propose(&case("c2", false)).is_none());
    // Its identity is NOT the official Jev model.
    assert!(!p.identity().official_jev);
}

#[test]
fn provider_identity_labels_official_vs_third_party() {
    // R-090: official Jev vs third-party implementations stay DISTINCT.
    let official = ProviderIdentity {
        name: s("jev-official"),
        digest: s("d1"),
        official_jev: true,
    };
    let third_party = ProviderIdentity {
        name: s("jev-like-community"),
        digest: s("d2"),
        official_jev: false,
    };
    assert_ne!(official.official_jev, third_party.official_jev);
}

// ---- Ticket 02: evaluation + authority ----

#[test]
fn calibration_abstention_rejection_measured_from_known_outcomes() {
    let provider = FixtureProvider {
        identity: ProviderIdentity {
            name: s("fixture"),
            digest: s("fx"),
            official_jev: false,
        },
        answers: [
            ("c1", (true, 0.9)), // correct, high confidence
            ("c2", (true, 0.2)), // WRONG (case not worth prioritizing), low
            ("c3", (true, 0.5)), // wrong (case correct=true, deprioritized), medium
        ]
        .into_iter()
        .map(|(k, v)| (s(k), v))
        .collect(),
    };
    let cases = vec![
        case("c1", true),
        case("c2", false),
        case("c3", true),
        case("c4", true), // provider ABSTAINS: rejection false negative
    ];
    let report = evaluate_provider(
        &provider,
        &cases,
        vec![CostQuantity {
            quantity: s("0.5"),
            unit: s("CPU-hours"),
        }],
    );
    // Abstention: 1 of 4.
    assert!((report.abstention_rate.unwrap() - 0.25).abs() < 1e-12);
    // Rejection false negative: c4 was resolvable but rejected.
    assert_eq!(report.rejection_false_negatives, 1);
    // Calibration buckets exist for the advised bands.
    assert!(!report.calibration.is_empty());
    let high = report
        .calibration
        .iter()
        .find(|b| b.band == ConfidenceBucket::High)
        .expect("high bucket");
    assert!((high.observed_correct_rate - 1.0).abs() < 1e-12);
    // Cost is a quantity, unit-labeled.
    assert_eq!(report.cost[0].unit, "CPU-hours");
    // End-to-end quality measured over advised cases (2 correct of 3).
    assert!((report.end_to_end_quality.unwrap() - 2.0 / 3.0).abs() < 1e-12);
}

// AT-053 negative case: a decision provider with no domain
// calibration record produces NOT MEASURED — no invented calibration,
// quality, or abstention numbers to drive automatic advisory use.
#[test]
fn empty_fixtures_yield_not_measured_not_zeros() {
    let p = NullProvider;
    let report = evaluate_provider(&p, &[], vec![]);
    // Honest absence: no invented calibration or quality numbers.
    assert!(report.calibration.is_empty());
    assert!(report.end_to_end_quality.is_none());
    assert!(report.abstention_rate.is_none());
}

// AT-053 required outcome: automatic advisory use is limited — the
// advisory plane cannot change gates, mint budget, or qualify methods,
// and the core runs without any provider at all.
#[test]
fn control_authority_attestation() {
    let att = assert_control_authority();
    assert!(att.advisory_cannot_change_gates);
    assert!(att.advisory_cannot_mint_budget);
    assert!(att.advisory_cannot_qualify_methods);
    assert!(att.core_runs_without_provider);
}

#[test]
fn recommendation_types_distinct() {
    // Prioritize / Deprioritize / Abstain are distinct variants.
    assert_ne!(
        Recommendation::Prioritize(s("x")),
        Recommendation::Deprioritize(s("x"))
    );
}

#[test]
fn twin_run_byte_identical() {
    let provider = FixtureProvider {
        identity: ProviderIdentity {
            name: s("fixture"),
            digest: s("fx"),
            official_jev: false,
        },
        answers: [("c1", (true, 0.9))]
            .into_iter()
            .map(|(k, v)| (s(k), v))
            .collect(),
    };
    let cases = vec![case("c1", true)];
    let a = evaluate_provider(&provider, &cases, vec![]);
    let b = evaluate_provider(&provider, &cases, vec![]);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
