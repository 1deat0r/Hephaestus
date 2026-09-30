//! Receipt subject and plan binding verification (T-004, R-094).
//!
//! Port of the `receipt(...)` checks in `reference/qualification.py`: a
//! receipt must be externally trusted, bind the exact plan/hypothesis payload
//! (via [`super::digest`] digests), carry the subject's own payload digest,
//! cover the plan's endpoints/guardrails and required artifacts, and only
//! then count as evidence. Receipt references themselves are excluded from
//! the subject digest to avoid hash cycles — see `subject_digest`.

use serde_json::Value;

use super::trust::TrustContext;

fn record_id(record: &Value) -> String {
    record
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("<unknown>")
        .to_string()
}

fn error(code: &str, owner: &Value) -> String {
    format!("{code}: {}", record_id(owner))
}

fn ref_of(record: &Value) -> Value {
    serde_json::json!({
        "id": record.get("id").and_then(Value::as_str).unwrap_or(""),
        "version": record.get("record_version").and_then(Value::as_i64).unwrap_or(-1),
    })
}

fn str_array<'v>(value: &'v Value, field: &str) -> Vec<&'v str> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

/// Inputs for one receipt verification.
pub struct ReceiptCheck<'a> {
    /// The `verification_receipt` record (already resolved by kind).
    pub receipt: &'a Value,
    /// The receipt's identity as referenced by the owner record.
    pub receipt_ref_id: &'a str,
    pub receipt_ref_version: i64,
    /// The record that claims this receipt (result or dossier).
    pub owner: &'a Value,
    /// Required purpose: execution | analysis | reproduction | promotion.
    pub purpose: &'a str,
    /// The experiment plan the receipt must bind.
    pub plan: &'a Value,
    /// The hypothesis the receipt must bind.
    pub hypothesis: &'a Value,
    /// The mission resolved from the plan's `mission_ref` (None mirrors the
    /// Python reference when the reference is unresolvable).
    pub mission: Option<&'a Value>,
    /// Artifact hashes the receipt must include.
    pub required_artifacts: &'a [&'a str],
}

/// Verify a receipt binding. Returns reason-coded errors; empty = the
/// receipt is trusted, correctly bound, and covers what it claims.
pub fn verify_receipt(check: &ReceiptCheck<'_>, ctx: &TrustContext) -> Vec<String> {
    let ReceiptCheck {
        receipt,
        receipt_ref_id,
        receipt_ref_version,
        owner,
        purpose,
        plan,
        hypothesis,
        mission,
        required_artifacts,
    } = check;
    let mut errors = Vec::new();

    let trusted = ctx.trusted_receipt(receipt_ref_id, *receipt_ref_version, receipt);
    if !trusted {
        errors.push(error("RECEIPT_NOT_TRUSTED", owner));
        return errors;
    }

    // Expected plan bindings — exact payload equality, computed from the
    // plan/hypothesis/mission records rather than trusted from the receipt.
    let mission_ref = plan.get("mission_ref").cloned().unwrap_or(Value::Null);
    // Expected policy version comes from the MISSION record, never from the
    // receipt under examination (no self-certification).
    let policy_version = mission
        .and_then(|m| m.get("authorization"))
        .and_then(|a| a.get("policy_version"))
        .cloned();
    let expected_bindings = serde_json::json!({
        "mission_ref": mission_ref,
        "hypothesis_ref": ref_of(hypothesis),
        "experiment_plan_ref": ref_of(plan),
        "candidate_sha256": plan.get("intervention_artifact_sha256").cloned().unwrap_or(Value::Null),
        "evaluator_sha256": plan.get("protected_evaluator_sha256").cloned().unwrap_or(Value::Null),
        "analysis_implementation_sha256": plan
            .get("analysis")
            .and_then(|a| a.get("implementation_sha256"))
            .cloned()
            .unwrap_or(Value::Null),
        "evidence_snapshot_sha256": plan.get("evidence_snapshot_sha256").cloned().unwrap_or(Value::Null),
        "policy_version": policy_version,
        "registration_sha256": plan
            .get("registration")
            .and_then(|r| r.get("payload_sha256"))
            .cloned()
            .unwrap_or(Value::Null),
    });
    let bindings_ok = receipt.get("bindings") == Some(&expected_bindings)
        && receipt.get("purpose").and_then(Value::as_str) == Some(purpose)
        && receipt.get("outcome").and_then(Value::as_str) == Some("pass");
    if !bindings_ok {
        errors.push(error("RECEIPT_BINDING_MISMATCH", owner));
    }

    // Subject binding: digest of the owner excluding receipt references.
    let subject_ok = receipt.get("subject_ref") == Some(&ref_of(owner))
        && receipt
            .get("subject_payload_sha256")
            .and_then(Value::as_str)
            == super::digest::subject_digest(owner).ok().as_deref();
    if !subject_ok {
        errors.push(error("RECEIPT_SUBJECT_MISMATCH", owner));
    }

    // Coverage: plan endpoints and guardrails must be present.
    let plan_endpoints = str_array(
        plan.get("analysis").unwrap_or(&Value::Null),
        "primary_endpoints",
    );
    let receipt_endpoints = str_array(receipt, "endpoint_ids");
    let plan_guardrails = str_array(plan, "guardrail_ids");
    let receipt_guardrails = str_array(receipt, "guardrail_ids");
    let coverage_ok = plan_endpoints.iter().all(|e| receipt_endpoints.contains(e))
        && plan_guardrails
            .iter()
            .all(|g| receipt_guardrails.contains(g));
    if !coverage_ok {
        errors.push(error("RECEIPT_COVERAGE_MISSING", owner));
    }

    // Required artifacts must be listed, and every listed artifact must be
    // externally verified bytes.
    let receipt_artifacts = str_array(receipt, "artifact_hashes");
    if !required_artifacts
        .iter()
        .all(|a| receipt_artifacts.contains(a))
    {
        errors.push(error("RECEIPT_ARTIFACT_MISMATCH", owner));
    }
    for hash in &receipt_artifacts {
        if !ctx.has_verified_artifact(hash) {
            errors.push(error("ARTIFACT_NOT_VERIFIED", owner));
            break;
        }
    }

    errors
}
