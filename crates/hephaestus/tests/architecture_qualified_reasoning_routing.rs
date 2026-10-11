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
