//! Deny-first tests for receipt subject/plan bindings (T-004).

use hephaestus::security::digest::{record_digest, subject_digest};
use hephaestus::security::receipt::{ReceiptCheck, verify_receipt};
use hephaestus::security::trust::TrustContext;
use serde_json::{Value, json};

fn artifact(ch: char) -> &'static str {
    Box::leak(ch.to_string().repeat(64).into_boxed_str())
}

fn leak(value: Value) -> &'static Value {
    Box::leak(Box::new(value))
}

fn mission() -> &'static Value {
    leak(json!({
        "id": "MIS-1", "kind": "mission", "record_version": 1,
        "authorization": {"policy_version": "PV-1"}
    }))
}

fn hypothesis() -> &'static Value {
    leak(json!({
        "id": "HYP-1", "kind": "hypothesis", "record_version": 1,
        "mission_ref": {"id": "MIS-1", "version": 1}
    }))
}

fn plan() -> &'static Value {
    leak(json!({
        "id": "PLAN-1", "kind": "experiment_plan", "record_version": 1,
        "mission_ref": {"id": "MIS-1", "version": 1},
        "intervention_artifact_sha256": artifact('c'),
        "protected_evaluator_sha256": artifact('d'),
        "evidence_snapshot_sha256": artifact('e'),
        "analysis": {"implementation_sha256": artifact('f'), "primary_endpoints": ["PRED-1"]},
        "registration": {"payload_sha256": artifact('9')},
        "guardrail_ids": ["G-1"]
    }))
}

fn owner() -> &'static Value {
    leak(json!({
        "id": "RES-1", "kind": "experiment_result", "record_version": 1,
        "execution_receipt_ref": {"id": "RCPT-1", "version": 1},
        "hypothesis_ref": {"id": "HYP-1", "version": 1}
    }))
}

fn expected_bindings() -> Value {
    json!({
        "mission_ref": {"id": "MIS-1", "version": 1},
        "hypothesis_ref": {"id": "HYP-1", "version": 1},
        "experiment_plan_ref": {"id": "PLAN-1", "version": 1},
        "candidate_sha256": artifact('c'),
        "evaluator_sha256": artifact('d'),
        "analysis_implementation_sha256": artifact('f'),
        "evidence_snapshot_sha256": artifact('e'),
        "policy_version": "PV-1",
        "registration_sha256": artifact('9')
    })
}

fn receipt_with(bindings: Value) -> Value {
    json!({
        "id": "RCPT-1", "kind": "verification_receipt", "record_version": 1,
        "purpose": "execution",
        "outcome": "pass",
        "bindings": bindings,
        "endpoint_ids": ["PRED-1"],
        "guardrail_ids": ["G-1"],
        "artifact_hashes": [artifact('1')],
        "subject_ref": {"id": "RES-1", "version": 1},
        "subject_payload_sha256": subject_digest(owner()).unwrap(),
        "independence_basis": null,
        "issuer_id": "SYNTHETIC-RUNNER"
    })
}

fn trusted_ctx(receipt: &Value) -> TrustContext {
    let mut ctx = TrustContext::empty();
    ctx.trust_receipt("RCPT-1", 1);
    ctx.authenticate_record_hash("RCPT-1", 1, record_digest(receipt).unwrap());
    ctx.verify_artifact(artifact('1').to_string());
    ctx
}

fn check<'a>(receipt: &'a Value, owner: &'a Value, required: &'a [&'a str]) -> ReceiptCheck<'a> {
    ReceiptCheck {
        receipt,
        receipt_ref_id: "RCPT-1",
        receipt_ref_version: 1,
        owner,
        purpose: "execution",
        plan: plan(),
        hypothesis: hypothesis(),
        mission: Some(mission()),
        required_artifacts: required,
    }
}

fn required_one() -> &'static [&'static str] {
    Box::leak(vec![artifact('1')].into_boxed_slice())
}

#[test]
fn consistent_receipt_verifies() {
    let receipt = receipt_with(expected_bindings());
    let ctx = trusted_ctx(&receipt);
    let errors = verify_receipt(&check(&receipt, owner(), required_one()), &ctx);
    assert!(errors.is_empty(), "expected no errors, got {errors:?}");
}

#[test]
fn untrusted_receipt_denies_and_stops() {
    let receipt = receipt_with(expected_bindings());
    let ctx = TrustContext::empty();
    let errors = verify_receipt(&check(&receipt, owner(), required_one()), &ctx);
    assert_eq!(errors, vec!["RECEIPT_NOT_TRUSTED: RES-1"]);
}

#[test]
fn altered_bindings_denied_even_with_fresh_trust() {
    let mut bindings = expected_bindings();
    bindings["registration_sha256"] = json!(artifact('0'));
    let receipt = receipt_with(bindings);
    let ctx = trusted_ctx(&receipt);
    let errors = verify_receipt(&check(&receipt, owner(), required_one()), &ctx);
    assert!(errors.contains(&"RECEIPT_BINDING_MISMATCH: RES-1".to_string()));
}

#[test]
fn policy_comes_from_mission_not_from_receipt() {
    // The receipt claims PV-1; the mission actually runs PV-2. A circular
    // implementation (expectation derived from the receipt) would pass here.
    let receipt = receipt_with(expected_bindings());
    let ctx = trusted_ctx(&receipt);
    let mutated_mission = leak(json!({
        "id": "MIS-1", "kind": "mission", "record_version": 1,
        "authorization": {"policy_version": "PV-2"}
    }));
    let mut c = check(&receipt, owner(), required_one());
    c.mission = Some(mutated_mission);
    let errors = verify_receipt(&c, &ctx);
    assert!(errors.contains(&"RECEIPT_BINDING_MISMATCH: RES-1".to_string()));
}

#[test]
fn wrong_purpose_denies() {
    let receipt = receipt_with(expected_bindings());
    let ctx = trusted_ctx(&receipt);
    let mut c = check(&receipt, owner(), required_one());
    c.purpose = "promotion";
    let errors = verify_receipt(&c, &ctx);
    assert!(errors.contains(&"RECEIPT_BINDING_MISMATCH: RES-1".to_string()));
}

#[test]
fn non_pass_outcome_denies() {
    let mut receipt = receipt_with(expected_bindings());
    receipt["outcome"] = json!("fail");
    let ctx = trusted_ctx(&receipt);
    let errors = verify_receipt(&check(&receipt, owner(), required_one()), &ctx);
    assert!(errors.contains(&"RECEIPT_BINDING_MISMATCH: RES-1".to_string()));
}

#[test]
fn wrong_subject_reference_denies() {
    let mut receipt = receipt_with(expected_bindings());
    receipt["subject_ref"] = json!({"id": "RES-OTHER", "version": 1});
    let ctx = trusted_ctx(&receipt);
    let errors = verify_receipt(&check(&receipt, owner(), required_one()), &ctx);
    assert!(errors.contains(&"RECEIPT_SUBJECT_MISMATCH: RES-1".to_string()));
}

#[test]
fn subject_digest_without_receipt_exclusion_denies() {
    // record_digest (no receipt-reference exclusion) must not satisfy the
    // subject binding — that would break the cycle-exclusion rule.
    let mut receipt = receipt_with(expected_bindings());
    receipt["subject_payload_sha256"] = json!(record_digest(owner()).unwrap());
    let ctx = trusted_ctx(&receipt);
    let errors = verify_receipt(&check(&receipt, owner(), required_one()), &ctx);
    assert!(errors.contains(&"RECEIPT_SUBJECT_MISMATCH: RES-1".to_string()));
}

#[test]
fn missing_endpoint_coverage_denies() {
    let mut receipt = receipt_with(expected_bindings());
    receipt["endpoint_ids"] = json!([]);
    let ctx = trusted_ctx(&receipt);
    let errors = verify_receipt(&check(&receipt, owner(), required_one()), &ctx);
    assert!(errors.contains(&"RECEIPT_COVERAGE_MISSING: RES-1".to_string()));
}

#[test]
fn missing_guardrail_coverage_denies() {
    let mut receipt = receipt_with(expected_bindings());
    receipt["guardrail_ids"] = json!([]);
    let ctx = trusted_ctx(&receipt);
    let errors = verify_receipt(&check(&receipt, owner(), required_one()), &ctx);
    assert!(errors.contains(&"RECEIPT_COVERAGE_MISSING: RES-1".to_string()));
}

#[test]
fn missing_required_artifact_denies() {
    let mut receipt = receipt_with(expected_bindings());
    // Receipt lists a different (verified) artifact; the required one is
    // absent from its list.
    receipt["artifact_hashes"] = json!([artifact('7')]);
    let mut ctx = trusted_ctx(&receipt);
    ctx.verify_artifact(artifact('7').to_string());
    let errors = verify_receipt(&check(&receipt, owner(), required_one()), &ctx);
    assert!(errors.contains(&"RECEIPT_ARTIFACT_MISMATCH: RES-1".to_string()));
}

#[test]
fn unverified_artifact_bytes_denies() {
    let mut receipt = receipt_with(expected_bindings());
    receipt["artifact_hashes"] = json!([artifact('7')]);
    // Receipt is trusted and freshly authenticated; only artifact
    // verification is missing.
    let ctx = trusted_ctx(&receipt);
    let errors = verify_receipt(&check(&receipt, owner(), required_one()), &ctx);
    assert!(errors.contains(&"ARTIFACT_NOT_VERIFIED: RES-1".to_string()));
}
