//! Yield accounting (T-040, R-105).
//!
//! Separate useful positive outcome yield, useful negative-result
//! yield, originality, and economics — computed from RECORDED outcomes
//! only, each quantity reported separately (no single universal score).

pub mod record;

pub use record::{Outcome, OutcomeKind, YieldReport};

/// Account a set of recorded outcomes into the separated report.
pub fn account(outcomes: &[Outcome]) -> YieldReport {
    let useful_positive = outcomes
        .iter()
        .filter(|o| o.kind == OutcomeKind::UsefulPositive)
        .count();
    let useful_negative = outcomes
        .iter()
        .filter(|o| o.kind == OutcomeKind::UsefulNegative)
        .count();
    let original_count = outcomes.iter().filter(|o| o.original).count();
    let total_cost_minor_units: u64 = outcomes.iter().map(|o| o.cost_minor_units).sum();
    let useful = useful_positive + useful_negative;
    let cost_per_useful_minor_units = if useful == 0 {
        0
    } else {
        total_cost_minor_units / useful as u64
    };
    YieldReport {
        useful_positive,
        useful_negative,
        original_count,
        cost_per_useful_minor_units,
        total_cost_minor_units,
    }
}
