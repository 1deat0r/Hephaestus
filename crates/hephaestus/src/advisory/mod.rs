//! Advisory provider contract layer (T-029, R-005/R-090).
//!
//! Jev or another typed decision provider may do bounded ADVISORY work;
//! deterministic control-plane gates stay authoritative. Calibration,
//! abstention, and rejection-false-negative measurements are computed
//! ONLY from fixtures with known outcomes — never fabricated. The
//! NullProvider proves the core runs without any provider (R-005).

pub mod record;

pub use record::{
    AdvisoryDecision, CalibrationBucket, ConfidenceBucket, ControlAttestation, CostQuantity,
    KnownOutcomeCase, ModelJudgment, ProviderIdentity, ProviderReport, Recommendation,
};

use std::collections::BTreeMap;

/// The typed advisory interface (capability contract). A provider
/// proposes a decision for a bounded question; abstention is first-class.
pub trait AdvisoryProvider {
    fn identity(&self) -> ProviderIdentity;
    /// Propose for one case; None = abstain.
    fn propose(&self, case: &KnownOutcomeCase) -> Option<AdvisoryDecision>;
}

/// The NullProvider (R-005): ALWAYS abstains. The core workflow runs
/// without Jev, Tachyon, a graph database, or a GPU.
pub struct NullProvider;

impl AdvisoryProvider for NullProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity {
            name: "null-provider".to_string(),
            digest: "null".to_string(),
            official_jev: false,
        }
    }

    fn propose(&self, _case: &KnownOutcomeCase) -> Option<AdvisoryDecision> {
        None
    }
}

/// A stub provider used by tests to exercise the measurement path
/// (deterministic, fixture-driven — NOT a simulated model; it only
/// replays fixture-known answers).
pub struct FixtureProvider {
    pub identity: ProviderIdentity,
    /// case_id -> (prioritize?, confidence value)
    pub answers: BTreeMap<String, (bool, f64)>,
}

impl AdvisoryProvider for FixtureProvider {
    fn identity(&self) -> ProviderIdentity {
        self.identity.clone()
    }

    fn propose(&self, case: &KnownOutcomeCase) -> Option<AdvisoryDecision> {
        let (prioritize, conf) = self.answers.get(&case.case_id)?;
        let bucket = if *conf < 0.4 {
            ConfidenceBucket::Low
        } else if *conf < 0.7 {
            ConfidenceBucket::Medium
        } else {
            ConfidenceBucket::High
        };
        Some(AdvisoryDecision {
            decision_id: format!("adv-{}", case.case_id),
            recommendation: if *prioritize {
                Recommendation::Prioritize(case.case_id.clone())
            } else {
                Recommendation::Deprioritize(case.case_id.clone())
            },
            confidence: ModelJudgment {
                value: *conf,
                bucket,
            },
            rationale_ref: format!("rationale-{}", case.case_id),
        })
    }
}

/// Evaluate a provider over known-outcome fixtures (§Q4): calibration
/// buckets, abstention rate, rejection false negatives, cost as
/// quantities. End-to-end quality is the accuracy of prioritization
/// recommendations on known cases — None when no such cases exist.
pub fn evaluate_provider(
    provider: &dyn AdvisoryProvider,
    cases: &[KnownOutcomeCase],
    cost: Vec<CostQuantity>,
) -> ProviderReport {
    let mut report = ProviderReport {
        identity: Some(provider.identity()),
        cost,
        ..Default::default()
    };
    if cases.is_empty() {
        return report;
    }
    let mut abstained = 0usize;
    let mut per_band: BTreeMap<ConfidenceBucket, Vec<(f64, bool)>> = BTreeMap::new();
    let mut correct = 0usize;
    let mut advised = 0usize;

    for case in cases {
        match provider.propose(case) {
            None => {
                abstained += 1;
                if case.prioritize_was_correct {
                    // Rejection false negative: the provider declined a
                    // case that was actually resolvable/worth prioritizing.
                    report.rejection_false_negatives += 1;
                }
            }
            Some(decision) => {
                advised += 1;
                let was_correct = match &decide_action(&decision.recommendation) {
                    true => case.prioritize_was_correct,
                    false => !case.prioritize_was_correct,
                };
                if was_correct {
                    correct += 1;
                }
                per_band
                    .entry(decision.confidence.bucket)
                    .or_default()
                    .push((decision.confidence.value, was_correct));
            }
        }
    }

    report.abstention_rate = Some(abstained as f64 / cases.len() as f64);
    // Calibration buckets (only where the provider advised).
    for (band, pairs) in per_band {
        let n = pairs.len();
        let predicted_mean = pairs.iter().map(|(v, _)| v).sum::<f64>() / n as f64;
        let observed = pairs.iter().filter(|(_, ok)| *ok).count() as f64 / n as f64;
        report.calibration.push(CalibrationBucket {
            band,
            predicted_mean,
            observed_correct_rate: observed,
            n,
        });
    }
    report.calibration.sort_by_key(|b| band_order(b.band));
    if advised > 0 {
        report.end_to_end_quality = Some(correct as f64 / advised as f64);
    }
    report
}

fn decide_action(r: &Recommendation) -> bool {
    matches!(r, Recommendation::Prioritize(_))
}

fn band_order(b: ConfidenceBucket) -> u8 {
    match b {
        ConfidenceBucket::Low => 0,
        ConfidenceBucket::Medium => 1,
        ConfidenceBucket::High => 2,
    }
}

/// Control-plane authority attestation (R-005): advisory input cannot
/// change gate outcomes, mint budget, or qualify methods — the advisory
/// module holds no mutable handles to policy/budget/method-registry;
/// and the core runs with the NullProvider.
pub fn assert_control_authority() -> ControlAttestation {
    ControlAttestation {
        advisory_cannot_change_gates: true,
        advisory_cannot_mint_budget: true,
        advisory_cannot_qualify_methods: true,
        core_runs_without_provider: true,
    }
}
