//! Yield-accounting records (T-040, R-105).

use serde::{Deserialize, Serialize};

/// The outcome class of one pilot mission result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeKind {
    /// Advanced the mission.
    UsefulPositive,
    /// A valid negative: refuted/inconclusive but informative — not
    /// waste (R-105: useful negative-result yield separated).
    UsefulNegative,
    /// Neither informed nor advanced.
    Uninformative,
}

/// One recorded outcome (computed from pilot records, never guessed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    pub kind: OutcomeKind,
    /// Resource cost in minor units (R-103 continuity).
    pub cost_minor_units: u64,
    /// Not derivable from prior art (discovery continuity).
    pub original: bool,
}

/// The separated yield report — four quantities, never merged into one
/// universal score (R-032 continuity).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YieldReport {
    pub useful_positive: usize,
    pub useful_negative: usize,
    pub original_count: usize,
    /// Cost per useful outcome in minor units; 0 when no useful
    /// outcomes (no fabricated denominator).
    pub cost_per_useful_minor_units: u64,
    /// Total cost retained for the complete denominator (R-103).
    pub total_cost_minor_units: u64,
}
