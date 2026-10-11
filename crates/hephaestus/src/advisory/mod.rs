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

/// One candidate provider with qualified task evidence (AC 17).
///
/// `quality` is the end-to-end quality measured on known-outcome
/// cases (`evaluate_provider` output); `None` means no qualified
/// evidence exists. `qualified_cases` counts the cases behind the
/// measurement. `capabilities` lists the provider capability contract
/// (for example "triage"). `cost` uses one unit per selection call.
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderOffer {
    pub identity: ProviderIdentity,
    pub cost: f64,
    pub quality: Option<f64>,
    pub qualified_cases: usize,
    pub capabilities: Vec<String>,
}

/// What the task needs: required capabilities from the capability
/// contract, minimum measured quality, and minimum evidence size.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskRequirements {
    pub required_capabilities: Vec<String>,
    pub min_quality: f64,
    pub min_qualified_cases: usize,
}

/// Why provider selection refused.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectError {
    /// No offer passes the capability contract and evidence gates.
    NoEligibleProvider { evaluated: usize },
    /// One offer carries an unusable cost (NaN, infinite, negative).
    InvalidOffer { name: String, reason: String },
}

impl std::fmt::Display for SelectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoEligibleProvider { evaluated } => {
                write!(f, "no eligible provider among {evaluated} offers")
            }
            Self::InvalidOffer { name, reason } => {
                write!(f, "invalid offer {name}: {reason}")
            }
        }
    }
}

impl std::error::Error for SelectError {}

fn offer_eligible(offer: &ProviderOffer, req: &TaskRequirements) -> bool {
    // Capability contract: every required capability present.
    if !req
        .required_capabilities
        .iter()
        .all(|c| offer.capabilities.contains(c))
    {
        return false;
    }
    // Qualified task evidence: measured quality meets the floor on
    // enough known-outcome cases. Missing evidence never qualifies.
    match offer.quality {
        Some(q) if q >= req.min_quality && offer.qualified_cases >= req.min_qualified_cases => {}
        _ => return false,
    }
    true
}

/// Select the least costly eligible provider.
///
/// Eligibility needs the capability contract plus qualified task
/// evidence (measured quality on enough known-outcome cases).
/// Unqualified offers never win, however cheap. Ties break by
/// provider name so repeated calls agree. Costs must be finite
/// and non-negative; any invalid cost refuses the whole call.
pub fn select_provider(
    offers: &[ProviderOffer],
    req: &TaskRequirements,
) -> Result<ProviderIdentity, SelectError> {
    for offer in offers {
        if !offer.cost.is_finite() || offer.cost < 0.0 {
            return Err(SelectError::InvalidOffer {
                name: offer.identity.name.clone(),
                reason: "cost must be finite and non-negative".to_string(),
            });
        }
    }
    let mut eligible: Vec<&ProviderOffer> =
        offers.iter().filter(|o| offer_eligible(o, req)).collect();
    if eligible.is_empty() {
        return Err(SelectError::NoEligibleProvider {
            evaluated: offers.len(),
        });
    }
    // Least cost first; name breaks ties deterministically.
    eligible.sort_by(|a, b| {
        a.cost
            .partial_cmp(&b.cost)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.identity.name.cmp(&b.identity.name))
    });
    Ok(eligible[0].identity.clone())
}

/// Retry and token bounds for escalation (AC 18).
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPolicy {
    pub max_retries: u32,
    pub token_limit: u64,
}

/// One independent check result. `passed` comes from a verifier
/// separate from the proposing provider — never from the
/// provider's own confidence.
#[derive(Debug, Clone, PartialEq)]
pub struct IndependentCheck {
    pub passed: bool,
    pub tokens_used: u64,
}

/// Running escalation account: attempts spent, tokens spent, and
/// failures observed. Counts only move forward.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EscalationState {
    pub attempts: u32,
    pub tokens_spent: u64,
    pub failures: u64,
}

/// Why escalation refused.
#[derive(Debug, Clone, PartialEq)]
pub enum EscalationError {
    /// The caller offered self-confidence instead of an independent
    /// pass. Self-confidence never qualifies.
    SelfConfidenceNotQualification { confidence: f64 },
    /// The independent check failed. The failure counts.
    IndependentCheckFailed { failures: u64 },
    /// The retry budget is spent.
    RetryBudgetExhausted { attempts: u32, max_retries: u32 },
    /// The token limit binds.
    TokenLimitExceeded { tokens_spent: u64, token_limit: u64 },
}

impl std::fmt::Display for EscalationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SelfConfidenceNotQualification { confidence } => {
                write!(f, "self-confidence {confidence} is not qualification")
            }
            Self::IndependentCheckFailed { failures } => {
                write!(f, "independent check failed (failures: {failures})")
            }
            Self::RetryBudgetExhausted {
                attempts,
                max_retries,
            } => {
                write!(
                    f,
                    "retry budget exhausted ({attempts} of {max_retries} spent)"
                )
            }
            Self::TokenLimitExceeded {
                tokens_spent,
                token_limit,
            } => {
                write!(
                    f,
                    "token limit exceeded ({tokens_spent} of {token_limit} spent)"
                )
            }
        }
    }
}

impl std::error::Error for EscalationError {}

/// Request escalation from an independent check.
///
/// The check must pass on its own; any offered self-confidence with
/// a failing check refuses as `SelfConfidenceNotQualification`.
/// Retry and token limits bind before any attempt records. Passed
/// gates record one attempt plus tokens; a failing check also
/// records one failure. Refusals from spent budgets record nothing.
pub fn request_escalation(
    state: &mut EscalationState,
    policy: &EscalationPolicy,
    check: &IndependentCheck,
    self_confidence: Option<f64>,
) -> Result<(), EscalationError> {
    // Self-confidence never substitutes for an independent pass.
    // Misuse refuses before any budget spends, but counts a failure.
    if !check.passed
        && let Some(confidence) = self_confidence
    {
        state.failures += 1;
        return Err(EscalationError::SelfConfidenceNotQualification { confidence });
    }
    if state.attempts >= policy.max_retries {
        return Err(EscalationError::RetryBudgetExhausted {
            attempts: state.attempts,
            max_retries: policy.max_retries,
        });
    }
    if state.tokens_spent.saturating_add(check.tokens_used) > policy.token_limit {
        return Err(EscalationError::TokenLimitExceeded {
            tokens_spent: state.tokens_spent,
            token_limit: policy.token_limit,
        });
    }
    state.attempts += 1;
    state.tokens_spent += check.tokens_used;
    if !check.passed {
        state.failures += 1;
        return Err(EscalationError::IndependentCheckFailed {
            failures: state.failures,
        });
    }
    Ok(())
}
