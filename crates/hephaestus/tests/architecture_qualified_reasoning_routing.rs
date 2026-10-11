//! AE-06 S1 — route deterministic operations (R-005, R-051, R-055).
//!
//! Parsing, arithmetic, authorization, hashing, and statistical
//! decisions always route to deterministic execution. Model calls
//! never serve them; advisory tasks alone may route to a provider.

use hephaestus::backend::{OperationKind, Route, RouteError, route_operation};

#[test]
fn s1_deterministic_ops_route_local() {
    // AC 16 positive: every deterministic operation routes local.
    for op in [
        OperationKind::Parsing,
        OperationKind::Arithmetic,
        OperationKind::Authorization,
        OperationKind::Hashing,
        OperationKind::StatisticalDecision,
    ] {
        assert_eq!(route_operation(&op), Route::Local, "op: {op:?}");
        assert_eq!(
            route_operation(&op),
            Route::Local,
            "twin call stable: {op:?}"
        );
    }
}

#[test]
fn s1_deterministic_ops_refuse_model_route() {
    // AC 16 refusal: requesting the advisory route for a deterministic
    // operation fails with the operation named.
    match OperationKind::Authorization.request_route(Route::Advisory) {
        Err(RouteError::ModelRefusedForDeterministicOp { operation }) => {
            assert_eq!(operation, "authorization");
        }
        other => panic!("expected ModelRefusedForDeterministicOp, got: {other:?}"),
    }
    // Local requests always pass, advisory ops included.
    assert_eq!(
        OperationKind::Hashing.request_route(Route::Local),
        Ok(Route::Local)
    );
}

#[test]
fn s1_deterministic_advisory_may_route_to_provider() {
    // AC 16 positive: genuinely advisory tasks may route to a
    // provider; the router never forces them local.
    let op = OperationKind::AdvisoryTask {
        task: "rank opportunities".to_string(),
    };
    assert_eq!(route_operation(&op), Route::Advisory);
    assert_eq!(op.request_route(Route::Advisory), Ok(Route::Advisory));
}

/// AE-06 S2 — select qualified providers (R-005, R-055, R-106).
/// The router selects the least costly eligible provider using
/// qualified task evidence. Optional providers stay behind
/// capability contracts: no evidence means no selection.
use hephaestus::advisory::record::ProviderIdentity;
use hephaestus::advisory::{ProviderOffer, SelectError, TaskRequirements, select_provider};

fn identity(name: &str) -> ProviderIdentity {
    ProviderIdentity {
        name: name.to_string(),
        digest: format!("digest-{name}"),
        official_jev: false,
    }
}

fn offer(
    name: &str,
    cost: f64,
    quality: Option<f64>,
    cases: usize,
    caps: &[&str],
) -> ProviderOffer {
    ProviderOffer {
        identity: identity(name),
        cost,
        quality,
        qualified_cases: cases,
        capabilities: caps.iter().map(|c| c.to_string()).collect(),
    }
}

fn req(caps: &[&str], min_quality: f64, min_cases: usize) -> TaskRequirements {
    TaskRequirements {
        required_capabilities: caps.iter().map(|c| c.to_string()).collect(),
        min_quality,
        min_qualified_cases: min_cases,
    }
}

#[test]
fn s2_routing_selects_cheapest_eligible() {
    // AC 17 positive: least costly eligible provider wins.
    let offers = vec![
        offer("expensive-good", 10.0, Some(0.9), 20, &["triage"]),
        offer("cheap-good", 2.0, Some(0.8), 20, &["triage"]),
        offer("mid-good", 5.0, Some(0.85), 20, &["triage"]),
    ];
    let selected = select_provider(&offers, &req(&["triage"], 0.7, 10)).expect("eligible exists");
    assert_eq!(selected.name, "cheap-good");
}

#[test]
fn s2_routing_refuses_unqualified() {
    // AC 17 refusal: no evidence means no selection, even when cheap.
    let offers = vec![
        offer("cheap-unknown", 1.0, None, 0, &["triage"]),
        offer("weak-evidence", 2.0, Some(0.4), 20, &["triage"]),
        offer("thin-evidence", 2.0, Some(0.9), 2, &["triage"]),
        offer("missing-cap", 2.0, Some(0.9), 20, &["other"]),
    ];
    match select_provider(&offers, &req(&["triage"], 0.7, 10)) {
        Err(SelectError::NoEligibleProvider { .. }) => {}
        other => panic!("expected NoEligibleProvider, got: {other:?}"),
    }
    // Invalid cost is refused with the reason named.
    let bad = vec![offer("bad-cost", f64::NAN, Some(0.9), 20, &["triage"])];
    assert!(matches!(
        select_provider(&bad, &req(&["triage"], 0.7, 10)),
        Err(SelectError::InvalidOffer { .. })
    ));
}

#[test]
fn s2_routing_tie_breaks_deterministically() {
    // AC 17 regression: equal costs break ties by name; twin calls
    // agree; an unqualified cheap offer never beats a qualified one.
    let offers = vec![
        offer("beta", 3.0, Some(0.8), 10, &["triage"]),
        offer("alpha", 3.0, Some(0.8), 10, &["triage"]),
        offer("cheap-unknown", 0.5, None, 0, &["triage"]),
    ];
    let r = req(&["triage"], 0.7, 10);
    let first = select_provider(&offers, &r).expect("eligible exists");
    let second = select_provider(&offers, &r).expect("eligible exists");
    assert_eq!(first.name, "alpha");
    assert_eq!(first, second);
}

/// AE-06 S3 — bound escalation and alternatives (R-005, R-055).
/// Escalation starts from independent checks. Retry and token limits
/// bind. Failures count. Self-confidence never qualifies.
use hephaestus::advisory::{
    EscalationError, EscalationPolicy, EscalationState, IndependentCheck, request_escalation,
};

fn policy() -> EscalationPolicy {
    EscalationPolicy {
        max_retries: 2,
        token_limit: 100,
    }
}

fn passing_check(tokens: u64) -> IndependentCheck {
    IndependentCheck {
        passed: true,
        tokens_used: tokens,
    }
}

fn failing_check(tokens: u64) -> IndependentCheck {
    IndependentCheck {
        passed: false,
        tokens_used: tokens,
    }
}

#[test]
fn s3_escalation_from_passing_check_within_limits() {
    // AC 18 positive: a passing independent check escalates and the
    // state records the attempt, tokens, and no failure.
    let mut state = EscalationState::default();
    request_escalation(&mut state, &policy(), &passing_check(30), None).expect("within limits");
    assert_eq!(state.attempts, 1);
    assert_eq!(state.tokens_spent, 30);
    assert_eq!(state.failures, 0);
}

#[test]
fn s3_escalation_refusals_named() {
    // AC 18 refusal: self-confidence alone never qualifies.
    let mut state = EscalationState::default();
    assert!(matches!(
        request_escalation(&mut state, &policy(), &failing_check(10), Some(0.99)),
        Err(EscalationError::SelfConfidenceNotQualification { .. })
    ));
    // A failed independent check counts a failure and refuses.
    assert!(matches!(
        request_escalation(&mut state, &policy(), &failing_check(10), None),
        Err(EscalationError::IndependentCheckFailed { .. })
    ));
    assert_eq!(state.failures, 2);
    // Retry budget exhausted after two attempts.
    let _ = request_escalation(&mut state, &policy(), &passing_check(10), None);
    assert!(matches!(
        request_escalation(&mut state, &policy(), &passing_check(10), None),
        Err(EscalationError::RetryBudgetExhausted { .. })
    ));
    // Token limit binds even with retries left.
    let mut rich = EscalationState::default();
    assert!(matches!(
        request_escalation(&mut rich, &policy(), &passing_check(101), None),
        Err(EscalationError::TokenLimitExceeded { .. })
    ));
}

#[test]
fn s3_escalation_counts_and_replays_deterministically() {
    // AC 18 regression: identical sequences reach identical states;
    // failures accumulate across mixed outcomes.
    let mut a = EscalationState::default();
    let mut b = EscalationState::default();
    for state in [&mut a, &mut b] {
        let _ = request_escalation(state, &policy(), &failing_check(10), None);
        let _ = request_escalation(state, &policy(), &passing_check(20), None);
    }
    assert_eq!(a, b);
    assert_eq!(a.attempts, 2);
    assert_eq!(a.failures, 1);
    assert_eq!(a.tokens_spent, 30);
}
