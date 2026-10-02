//! Advanced search evaluation with matched ablations (T-031).
//!
//! Each candidate acceleration mechanism is evaluated SEPARATELY against
//! a matched ablation (identical envelope, mechanism removed); promotion
//! requires measured value within quality guardrails AND disablability
//! (the core path stays functional). Absent infrastructure yields an
//! honest NotEvaluable — never fabricated measurements.

pub mod record;

pub use record::{
    AblationPair, CandidateMechanism, EnvelopeMismatch, MechanismKind, PairMeasurement,
    PromotionVerdict,
};

use serde::{Deserialize, Serialize};

/// Errors from pair construction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PairError {
    EnvelopeMismatch(EnvelopeMismatch),
    /// The baseline and the mechanism arm are the same arm.
    NotADistinctPair,
}

/// Build a matched ablation pair: the baseline arm (mechanism removed)
/// must have the identical envelope except the mechanism itself.
/// Mismatched envelopes are refused — that would not be a matched
/// ablation.
pub fn matched_ablation(
    mechanism: &CandidateMechanism,
    baseline_compute_budget: u64,
    baseline_tool_access: &[String],
) -> Result<AblationPair, PairError> {
    if mechanism.compute_budget != baseline_compute_budget {
        return Err(PairError::EnvelopeMismatch(EnvelopeMismatch {
            field: "compute_budget".to_string(),
            detail: format!(
                "{} vs baseline {}",
                mechanism.compute_budget, baseline_compute_budget
            ),
        }));
    }
    if mechanism.tool_access.len() != baseline_tool_access.len()
        || !mechanism
            .tool_access
            .iter()
            .zip(baseline_tool_access.iter())
            .all(|(a, b)| a == b)
    {
        return Err(PairError::EnvelopeMismatch(EnvelopeMismatch {
            field: "tool_access".to_string(),
            detail: "tool access sets differ".to_string(),
        }));
    }
    if mechanism.compute_budget == 0 && baseline_compute_budget == 0 {
        return Err(PairError::NotADistinctPair);
    }
    Ok(AblationPair {
        mechanism: mechanism.clone(),
        metric: "declared-metric".to_string(),
    })
}

/// Evaluate a recorded pair measurement (roadmap: promote only when
/// matched ablations demonstrate value within quality guardrails).
pub fn evaluate_pair(
    pair: &AblationPair,
    measurement: Option<&PairMeasurement>,
) -> PromotionVerdict {
    // Absent infrastructure: honestly not evaluated.
    if !pair.mechanism.infrastructure_available {
        return PromotionVerdict::NotEvaluable;
    }
    let Some(m) = measurement else {
        // Infrastructure exists but nothing recorded: not promotable yet.
        return PromotionVerdict::Retained;
    };
    // Guardrails first: violations fail the promotion regardless of value.
    if m.guardrail_violations > 0 {
        return PromotionVerdict::Retained;
    }
    // Measured value: the mechanism must actually improve the metric.
    if m.with_value <= m.without_value {
        return PromotionVerdict::Retained;
    }
    // Disablability: the core path must stay functional (campaign exit).
    if !pair.mechanism.disablable {
        return PromotionVerdict::Retained;
    }
    PromotionVerdict::Promoted
}

pub use record::{CachingComparator, CachingComparison};

/// The outcome of a context-caching benchmark review (R-083): either
/// the stronger comparator ran, or its exclusion is on the record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CachingReview {
    StrongerComparatorUsed,
    ExclusionJustified { justification: String },
}

/// Why benchmark review refused a caching comparison (typed-rejection
/// convention): neither the stronger comparator nor a justification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CachingReviewError {
    MissingStrongerComparatorOrJustification,
}

/// Benchmark review for proposed context caching (R-083/AT-083):
/// requests the stronger exact-cache comparator OR a recorded
/// exclusion justification — full reconstruction alone never passes
/// review (§28 straw-baseline rule).
pub fn review_caching_comparison(
    comparison: &CachingComparison,
) -> Result<CachingReview, CachingReviewError> {
    if comparison
        .comparators_used
        .contains(&record::CachingComparator::ExactCache)
    {
        return Ok(CachingReview::StrongerComparatorUsed);
    }
    match comparison.exclusion_justification.as_deref() {
        Some(j) if !j.is_empty() => Ok(CachingReview::ExclusionJustified {
            justification: j.to_string(),
        }),
        _ => Err(CachingReviewError::MissingStrongerComparatorOrJustification),
    }
}
