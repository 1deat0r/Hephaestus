//! Hidden ground-truth evaluation (T-003).
//!
//! The protected comparator: compares a candidate against the world's hidden
//! truth and the observation stream. [`Assessment`]s are for protected
//! consumers only — reasons may reference ground truth and must never be
//! shown to invention workers (they receive [`crate::Observation`]s, nothing
//! else).
//!
//! Verdicts:
//! * [`Verdict::Confirmed`] — candidate matches truth structurally.
//! * [`Verdict::Rejected`] — evidence distinguishes the candidate from truth
//!   (or its causal claim is false).
//! * [`Verdict::Inconclusive`] — candidate and truth are observationally
//!   equivalent inside the corpus's declared ambiguity band; the fixture is
//!   ambiguous, not the candidate vindicated.
//! * [`Verdict::Infeasible`] — the output requirement cannot be met; per the
//!   package rules no mechanism conclusion follows from a missed target.
//! * [`Verdict::OutOfScope`] — boundary conditions changed; the candidate
//!   holds only in one regime.

use crate::{Candidate, Observation, World};

/// Structural parameter agreement for a Confirmed verdict (corpus design
/// parameter, documented rather than tuned silently).
pub const PARAM_TOL: f64 = 1e-3;
/// Declared observational-equivalence band: a candidate whose RMSE is within
/// this of the truth's RMSE on the fixture stream is indistinguishable, not
/// refuted. Fixture worlds must be designed so false mechanisms fall outside
/// it and ambiguous mechanisms fall inside it.
pub const AMBIGUITY_BAND: f64 = 0.35;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Confirmed,
    Rejected,
    Inconclusive,
    Infeasible,
    OutOfScope,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assessment {
    pub verdict: Verdict,
    /// Protected-side explanation; may reference hidden truth.
    pub reason: String,
}

fn rmse(candidate: &Candidate, observations: &[Observation]) -> f64 {
    if observations.is_empty() {
        return f64::INFINITY;
    }
    let sum: f64 = observations
        .iter()
        .map(|o| {
            let d = candidate.predict(o.input) - o.output;
            d * d
        })
        .sum();
    (sum / observations.len() as f64).sqrt()
}

fn truth_rmse(world: &World, observations: &[Observation]) -> f64 {
    if observations.is_empty() {
        return f64::INFINITY;
    }
    let sum: f64 = observations
        .iter()
        .map(|o| {
            let (form, a, b) = world.truth_at(o.input);
            let d = form.predict(a, b, o.input) - o.output;
            d * d
        })
        .sum();
    (sum / observations.len() as f64).sqrt()
}

/// Minimum of a single form over [lo, hi] (piecewise-monotone forms, so the
/// segment endpoints decide).
fn min_over(form: crate::Form, a: f64, b: f64, lo: f64, hi: f64) -> f64 {
    form.predict(a, b, lo).min(form.predict(a, b, hi))
}

fn min_truth_output(world: &World) -> f64 {
    let (lo, hi) = (world.domain[0], world.domain[1]);
    let mut bounds: Vec<f64> = vec![lo, hi];
    for regime in &world.regimes {
        if regime.break_at > lo && regime.break_at < hi {
            bounds.push(regime.break_at);
        }
    }
    bounds.sort_by(f64::total_cmp);
    let mut min = f64::INFINITY;
    for window in bounds.windows(2) {
        let (seg_lo, seg_hi) = (window[0], window[1]);
        let (form, a, b) = world.truth_at((seg_lo + seg_hi) / 2.0);
        min = min.min(min_over(form, a, b, seg_lo, seg_hi));
    }
    min
}

fn params_match(candidate: &Candidate, form: crate::Form, a: f64, b: f64) -> bool {
    candidate.form == form
        && (candidate.a - a).abs() <= PARAM_TOL
        && (candidate.b - b).abs() <= PARAM_TOL
}

/// Evaluate a candidate against hidden truth and the observation stream.
pub fn evaluate(candidate: &Candidate, world: &World, observations: &[Observation]) -> Assessment {
    let rejected_no_data = |why: &str| Assessment {
        verdict: Verdict::Rejected,
        reason: why.to_string(),
    };
    if observations.is_empty() {
        return rejected_no_data("no observations to evaluate against");
    }

    let (lo, hi) = (world.domain[0], world.domain[1]);

    //1. Candidate's own feasibility against the output requirement.
    if let Some(req) = world.requirement {
        if min_over(candidate.form, candidate.a, candidate.b, lo, hi) > req.output_max + 1e-9 {
            return Assessment {
                verdict: Verdict::Infeasible,
                reason: format!(
                    "candidate minimum output exceeds required maximum {}",
                    req.output_max
                ),
            };
        }
        // 2. The world itself can never satisfy the requirement: no
        //    mechanism conclusion is drawn from the missed target.
        if min_truth_output(world) > req.output_max + 1e-9 {
            return Assessment {
                verdict: Verdict::Infeasible,
                reason: format!(
                    "world output never reaches required maximum {}",
                    req.output_max
                ),
            };
        }
    }

    // 3. Confounder worlds: a candidate claiming input causality is rejected
    //    even when it fits the confounded curve beautifully.
    if !world.causal_effect_of_input && candidate.form.claims_input_effect(candidate.b) {
        return Assessment {
            verdict: Verdict::Rejected,
            reason: "input has no causal effect on output; the observed correlation is confounded"
                .to_string(),
        };
    }

    // 4. Changed boundary conditions: piecewise truth vs single candidate.
    if !world.regimes.is_empty() {
        let mut pieces: Vec<(f64, f64, crate::Form, f64, f64)> = Vec::new();
        let first_break = world.regimes[0].break_at;
        pieces.push((
            lo,
            first_break,
            world.truth_form,
            world.truth_a,
            world.truth_b,
        ));
        for (i, regime) in world.regimes.iter().enumerate() {
            let end = world.regimes.get(i + 1).map(|r| r.break_at).unwrap_or(hi);
            pieces.push((regime.break_at, end, regime.form, regime.a, regime.b));
        }
        let active: Vec<_> = pieces
            .iter()
            .filter(|(start, end, ..)| {
                observations
                    .iter()
                    .any(|o| o.input >= *start && o.input < *end)
            })
            .collect();
        if active.is_empty() {
            return rejected_no_data("observations fall outside every regime");
        }
        let matched = active
            .iter()
            .filter(|(_, _, form, a, b)| params_match(candidate, *form, *a, *b))
            .count();
        if matched == active.len() {
            return Assessment {
                verdict: Verdict::Confirmed,
                reason: "candidate matches every regime covered by observations".to_string(),
            };
        }
        if matched >= 1 {
            return Assessment {
                verdict: Verdict::OutOfScope,
                reason: "boundary conditions changed; candidate holds only in one regime"
                    .to_string(),
            };
        }
        return rejected_no_data("candidate matches no active regime");
    }

    // 5. Structural agreement with single-regime truth.
    if params_match(candidate, world.truth_form, world.truth_a, world.truth_b) {
        return Assessment {
            verdict: Verdict::Confirmed,
            reason: "candidate matches hidden mechanism form and parameters".to_string(),
        };
    }

    // 6. Evidence-based separation (or declared ambiguity).
    let candidate_rmse = rmse(candidate, observations);
    let truth_rmse = truth_rmse(world, observations);
    if candidate_rmse <= truth_rmse + AMBIGUITY_BAND {
        Assessment {
            verdict: Verdict::Inconclusive,
            reason: format!(
                "observationally equivalent within ambiguity band (candidate rmse {candidate_rmse:.4}, truth rmse {truth_rmse:.4})"
            ),
        }
    } else {
        Assessment {
            verdict: Verdict::Rejected,
            reason: format!(
                "evidence separates candidate from truth (candidate rmse {candidate_rmse:.4}, truth rmse {truth_rmse:.4})"
            ),
        }
    }
}
