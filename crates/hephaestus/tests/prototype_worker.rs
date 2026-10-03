//! Prototype worker (T-020, R-043/AT-043 (composed artifacts verified
//! against global invariants — assemble; an unsatisfied required
//! interface fails the parent report with a named finding),
//! R-094/R-095, MASTER_SPEC §15).
//!
//! Integration tests at the public seam: `authorize`, `assemble`.

use hephaestus::prototype::record::{
    BuildReceipt, Component, ProtectedContext, PrototypeChange, Rejection, TestReceipt,
};
use hephaestus::prototype::{assemble, authorize};

fn s(v: &str) -> String {
    v.to_string()
}

fn protected() -> ProtectedContext {
    ProtectedContext {
        evaluator_digest: s("eval-digest-1"),
        baseline_digest: s("baseline-digest-1"),
        verified_artifacts: vec![s("cand-artifact-2")],
    }
}

/// A legitimate candidate-artifact change with a verified receipt.
fn candidate_change() -> PrototypeChange {
    PrototypeChange {
        target_artifact: s("candidate/cache_policy.py"),
        new_artifact_digest: s("cand-artifact-2"),
        claim_ids: vec![s("C1")],
        build: BuildReceipt {
            artifact_digest: s("cand-artifact-2"),
            command: s("make build"),
        },
        test: TestReceipt {
            artifact_digest: s("cand-artifact-2"),
            passed: true,
            record: s("10/10 local tests"),
        },
    }
}

// ---- Ticket 01: authorize ----

#[test]
fn verified_candidate_change_accepted() {
    let receipt = authorize(&candidate_change(), &protected()).expect("authorized");
    assert_eq!(receipt.artifact_digest, "cand-artifact-2");
    assert_eq!(receipt.claim_ids, vec!["C1"]);
    // Receipt binds the evaluator digest the change was authorized
    // against (change is void if the evaluator moves).
    assert_eq!(receipt.against_evaluator_digest, "eval-digest-1");
}

#[test]
fn protected_targets_rejected_by_name() {
    // Evaluator-targeted change.
    let mut change = candidate_change();
    change.target_artifact = s("protected/evaluator.rs");
    assert_eq!(
        authorize(&change, &protected()),
        Err(Rejection::ProtectedEvaluatorTarget)
    );
    // Baseline-targeted change.
    let mut change = candidate_change();
    change.target_artifact = s("protected/baseline.json");
    assert_eq!(
        authorize(&change, &protected()),
        Err(Rejection::ProtectedBaselineTarget)
    );
}

#[test]
fn worker_claim_of_unchanged_evaluator_is_not_trust() {
    // R-095: a change whose new digest EQUALS the evaluator digest is
    // refused even though the worker "claims" it is just a candidate.
    let mut change = candidate_change();
    change.new_artifact_digest = s("eval-digest-1");
    change.build.artifact_digest = s("eval-digest-1");
    change.test.artifact_digest = s("eval-digest-1");
    assert_eq!(
        authorize(&change, &protected()),
        Err(Rejection::ProtectedEvaluatorTarget)
    );
}

#[test]
fn unverified_receipt_rejected() {
    // R-095: digest NOT in the protected registry -> rejected, no
    // matter what the worker claims.
    let mut change = candidate_change();
    change.new_artifact_digest = s("never-verified-digest");
    change.build.artifact_digest = s("never-verified-digest");
    change.test.artifact_digest = s("never-verified-digest");
    assert_eq!(
        authorize(&change, &protected()),
        Err(Rejection::UnverifiedArtifactDigest)
    );
}

#[test]
fn failing_test_receipt_rejected() {
    let mut change = candidate_change();
    change.test.passed = false;
    assert_eq!(
        authorize(&change, &protected()),
        Err(Rejection::ReceiptFailure)
    );
}

// ---- Ticket 02: assemble ----

#[test]
fn interface_and_version_checks_with_named_missing_edges() {
    let components = vec![
        Component {
            name: s("worker"),
            version: s("1.0"),
            provides: vec![s("iface:candidate")],
            requires: vec![s("iface:evaluator"), s("iface:nonexistent")],
            resource_budget: 2,
            invariants: vec![],
        },
        Component {
            name: s("evaluator"),
            version: s("1.0"),
            provides: vec![s("iface:evaluator")],
            requires: vec![],
            resource_budget: 3,
            invariants: vec![],
        },
    ];
    let report = assemble(&components, 10, &[]);
    // iface:evaluator satisfied.
    let ok_iface = report
        .findings
        .iter()
        .find(|f| f.check == "interface:iface:evaluator")
        .expect("finding");
    assert!(ok_iface.ok);
    // iface:nonexistent missing, edge NAMED.
    let missing = report
        .findings
        .iter()
        .find(|f| f.check == "interface:iface:nonexistent")
        .expect("finding");
    assert!(!missing.ok);
    assert!(missing.detail.contains("nonexistent"));
    assert_eq!(missing.component, "worker");
}

#[test]
fn resource_aggregation_and_cost_accounting() {
    let components = vec![
        Component {
            name: s("a"),
            version: s("1.0"),
            provides: vec![],
            requires: vec![],
            resource_budget: 6,
            invariants: vec![],
        },
        Component {
            name: s("b"),
            version: s("1.0"),
            provides: vec![],
            requires: vec![],
            resource_budget: 5,
            invariants: vec![],
        },
    ];
    // Over limit: aggregation check fails and names the total.
    let report = assemble(&components, 10, &[]);
    let agg = report
        .findings
        .iter()
        .find(|f| f.check == "resource_aggregation")
        .expect("finding");
    assert!(!agg.ok);
    assert_eq!(report.total_resource_budget, 11);
    // Cost-shift rows account every component's share (§15:297).
    assert!(
        report
            .cost_shift_rows
            .iter()
            .any(|(n, b)| n == "a" && *b == 6)
    );
    assert!(
        report
            .cost_shift_rows
            .iter()
            .any(|(n, b)| n == "b" && *b == 5)
    );
    // Within limit: passes.
    let report_ok = assemble(&components, 11, &[]);
    assert!(
        report_ok
            .findings
            .iter()
            .find(|f| f.check == "resource_aggregation")
            .unwrap()
            .ok
    );
}

#[test]
fn guardrails_checked_and_named() {
    let components = vec![Component {
        name: s("worker"),
        version: s("1.0"),
        provides: vec![],
        requires: vec![],
        resource_budget: 1,
        invariants: vec![s("no-stale-reads")],
    }];
    // Upheld guardrail passes; missing one fails and names itself.
    let report = assemble(
        &components,
        10,
        &[
            (s("no-stale-reads"), vec![]),
            (s("no-network-egress"), vec![]),
        ],
    );
    let upheld = report
        .findings
        .iter()
        .find(|f| f.check == "guardrail:no-stale-reads")
        .expect("finding");
    assert!(upheld.ok);
    let missing = report
        .findings
        .iter()
        .find(|f| f.check == "guardrail:no-network-egress")
        .expect("finding");
    assert!(!missing.ok);
    assert!(!report.passes, "a failed guardrail fails the assembly");
}

#[test]
fn twin_run_byte_identical() {
    let components = vec![Component {
        name: s("worker"),
        version: s("1.0"),
        provides: vec![s("iface:a")],
        requires: vec![s("iface:a")],
        resource_budget: 1,
        invariants: vec![],
    }];
    let a = assemble(&components, 5, &[]);
    let b = assemble(&components, 5, &[]);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
