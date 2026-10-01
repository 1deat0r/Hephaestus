//! Yield accounting (T-040, R-105).
//!
//! Integration tests at the public seam: `account`.

use hephaestus::yieldaccount::account;
use hephaestus::yieldaccount::record::{Outcome, OutcomeKind};

fn o(kind: OutcomeKind, cost: u64, original: bool) -> Outcome {
    Outcome {
        kind,
        cost_minor_units: cost,
        original,
    }
}

#[test]
fn four_quantities_separated() {
    // R-105: positive, useful-negative, originality, and economics are
    // reported SEPARATELY — never merged into one universal score.
    let outcomes = vec![
        o(OutcomeKind::UsefulPositive, 100, true),
        o(OutcomeKind::UsefulNegative, 50, false),
        o(OutcomeKind::UsefulNegative, 30, true),
        o(OutcomeKind::Uninformative, 20, false),
    ];
    let r = account(&outcomes);
    assert_eq!(r.useful_positive, 1);
    assert_eq!(r.useful_negative, 2, "valid negatives counted as useful");
    assert_eq!(r.original_count, 2);
    assert_eq!(r.total_cost_minor_units, 200);
    // Economics: 200 / (1 positive + 2 useful negatives) = 66.
    assert_eq!(r.cost_per_useful_minor_units, 66);
}

#[test]
fn uninformative_excluded_from_useful_denominator() {
    let outcomes = vec![o(OutcomeKind::Uninformative, 40, false)];
    let r = account(&outcomes);
    assert_eq!(r.useful_positive, 0);
    assert_eq!(r.useful_negative, 0);
    // No useful outcomes: cost-per-useful is 0, no fabricated division.
    assert_eq!(r.cost_per_useful_minor_units, 0);
    // But the total cost stays in the complete denominator (R-103).
    assert_eq!(r.total_cost_minor_units, 40);
}

#[test]
fn empty_input_all_zeros() {
    let r = account(&[]);
    assert_eq!(r.useful_positive, 0);
    assert_eq!(r.useful_negative, 0);
    assert_eq!(r.original_count, 0);
    assert_eq!(r.cost_per_useful_minor_units, 0);
    assert_eq!(r.total_cost_minor_units, 0);
}

#[test]
fn twin_run_byte_identical() {
    let a = account(&[o(OutcomeKind::UsefulPositive, 10, true)]);
    let b = account(&[o(OutcomeKind::UsefulPositive, 10, true)]);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
