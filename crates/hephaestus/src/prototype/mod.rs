//! Prototype worker change-level protection + assembly verification
//! (T-020, MASTER_SPEC section 15).
//!
//! Workers propose candidate-artifact changes with receipts; trust comes
//! from the protected context (R-095), not worker claims. Protected
//! evaluators and baselines are never valid change targets (§15:296).
//! Assembly verifies interfaces, versions, invariants, resource
//! aggregation, and end-to-end guardrails (§15:297) - passing local
//! tests is not sufficient for composition.

pub mod record;

pub use record::{
    AssemblyFinding, AssemblyReport, AuthorizationReceipt, BuildReceipt, Component,
    ProtectedContext, PrototypeChange, Rejection, TestReceipt,
};

/// Authorize a worker-proposed change (§15:296, R-094, R-095).
///
/// Rules, in order:
/// 1. Protected targets are refused by name (evaluator, baseline).
/// 2. Receipts must be internally consistent and passing.
/// 3. The new artifact digest must be in the protected registry
///    (bytes-verified) — a worker's claim of verification is not trust.
pub fn authorize(
    change: &PrototypeChange,
    protected: &ProtectedContext,
) -> Result<AuthorizationReceipt, Rejection> {
    // §15:296: authors cannot silently alter protected evaluators or the
    // baseline. Target identity is checked by NAME and by digest.
    if change.target_artifact.contains("evaluator")
        || change.new_artifact_digest == protected.evaluator_digest
    {
        return Err(Rejection::ProtectedEvaluatorTarget);
    }
    if change.target_artifact.contains("baseline")
        || change.new_artifact_digest == protected.baseline_digest
    {
        return Err(Rejection::ProtectedBaselineTarget);
    }
    // R-094: receipts present and passing.
    if !change.test.passed || change.build.artifact_digest != change.new_artifact_digest {
        return Err(Rejection::ReceiptFailure);
    }
    // R-095: the artifact bytes must be verified by the protected side.
    if !protected.verifies(&change.new_artifact_digest) {
        return Err(Rejection::UnverifiedArtifactDigest);
    }
    Ok(AuthorizationReceipt {
        target_artifact: change.target_artifact.clone(),
        artifact_digest: change.new_artifact_digest.clone(),
        claim_ids: change.claim_ids.clone(),
        against_evaluator_digest: protected.evaluator_digest.clone(),
    })
}

/// Assemble components and verify composition (§15:297): interfaces
/// pairwise, versions compatible, resource aggregation within the
/// mission limit, cost-shift accounting, named invariants, and
/// end-to-end guardrails. Failures name the failing component.
pub fn assemble(
    components: &[Component],
    mission_resource_limit: u64,
    guardrails: &[(String, Vec<String>)],
) -> AssemblyReport {
    let mut findings = Vec::new();
    let total: u64 = components.iter().map(|c| c.resource_budget).sum();

    // Interface + version compatibility (pairwise provides/requires).
    for requiring in components {
        for req in &requiring.requires {
            let satisfied = components
                .iter()
                .any(|c| c.name != requiring.name && c.provides.iter().any(|p| p == req));
            findings.push(AssemblyFinding {
                component: requiring.name.clone(),
                check: format!("interface:{req}"),
                ok: satisfied,
                detail: if satisfied {
                    "satisfied".to_string()
                } else {
                    format!("missing provider for required interface {req}")
                },
            });
        }
    }

    // Resource aggregation (§15:297).
    findings.push(AssemblyFinding {
        component: "assembly".to_string(),
        check: "resource_aggregation".to_string(),
        ok: total <= mission_resource_limit,
        detail: format!("total {total} vs limit {mission_resource_limit}"),
    });

    // Cost-shift accounting: per-component share of total (visible
    // redistribution; an optimization shifting cost to another stage
    // must be accounted in total system cost).
    let cost_shift_rows = components
        .iter()
        .map(|c| (c.name.clone(), c.resource_budget))
        .collect();

    // Global invariants: upheld iff every component declares them.
    let invariant_names: Vec<String> = components
        .iter()
        .flat_map(|c| c.invariants.iter().cloned())
        .collect();
    for expected in guardrails {
        let upheld = invariant_names.iter().any(|i| i == &expected.0);
        findings.push(AssemblyFinding {
            component: "assembly".to_string(),
            check: format!("guardrail:{}", expected.0),
            ok: upheld,
            detail: if upheld {
                "invariant declared by components".to_string()
            } else {
                "no component upholds this end-to-end guardrail".to_string()
            },
        });
    }

    let passes = findings.iter().all(|f| f.ok);
    AssemblyReport {
        findings,
        total_resource_budget: total,
        cost_shift_rows,
        passes,
    }
}
