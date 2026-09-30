//! Deny-first tests for scoped grant authorization (T-004).

use hephaestus::security::digest::record_digest;
use hephaestus::security::grant::{GrantCheck, authorize_grant, check_grant_interval};
use hephaestus::security::trust::TrustContext;
use serde_json::json;
use time::OffsetDateTime;

const T0: &str = "2026-09-30T00:00:00+00:00";
const T1: &str = "2026-10-01T00:00:00+00:00";

fn mission() -> serde_json::Value {
    json!({
        "id": "MIS-1",
        "kind": "mission",
        "record_version": 1,
        "authorization": {
            "state": "approved",
            "policy_version": "PV-1",
            "allowed_capabilities": ["run_sandboxed"],
            "allowed_destinations": ["local"]
        },
        "budget": {"currency": "USD", "minor_units": 1000}
    })
}

fn grant() -> serde_json::Value {
    json!({
        "id": "GRANT-1",
        "kind": "authorization_grant",
        "record_version": 1,
        "mission_ref": {"id": "MIS-1", "version": 1},
        "state": "approved",
        "policy_version": "PV-1",
        "artifact_sha256": "a".repeat(64),
        "operation_id": "OP-1",
        "destination": "local",
        "capabilities": ["run_sandboxed"],
        "max_cost": {"currency": "USD", "minor_units": 500},
        "issued_at": T0,
        "expires_at": T1
    })
}

fn record() -> serde_json::Value {
    json!({
        "id": "TASK-1",
        "kind": "task",
        "record_version": 1,
        "grant_ref": {"id": "GRANT-1", "version": 1},
        "budget": {"currency": "USD", "minor_units": 400},
        "required_capabilities": ["run_sandboxed"],
        "operation_id": "OP-1",
        "destination": "local"
    })
}

/// Fully consistent setup: trust + bindings + clock all present. Positive
/// control for the deny matrix.
fn trusted_ctx() -> TrustContext {
    let mut ctx = TrustContext::empty();
    ctx.trust_grant("GRANT-1", 1);
    ctx.authenticate_record_hash("GRANT-1", 1, record_digest(&grant()).unwrap());
    ctx.set_current_policy("MIS-1", "PV-1".to_string());
    ctx.set_evaluated_at(
        OffsetDateTime::parse(T0, &time::format_description::well_known::Rfc3339).unwrap(),
    );
    ctx
}

/// 'static artifact spellings for the borrow-checked test structs.
fn artifact(ch: char) -> &'static str {
    Box::leak(ch.to_string().repeat(64).into_boxed_str())
}

fn check<'a>(
    record: &'a serde_json::Value,
    mission: &'a serde_json::Value,
    grant: &'a serde_json::Value,
) -> GrantCheck<'a> {
    GrantCheck {
        record,
        mission,
        grant,
        artifact: Some(artifact('a')),
        capabilities: &["run_sandboxed"],
        operation: "OP-1",
        destination: Some("local"),
    }
}

#[test]
fn consistent_grant_authorizes() {
    let (record, mission, grant) = (record(), mission(), grant());
    let ctx = trusted_ctx();
    let c = check(&record, &mission, &grant);
    let errors = authorize_grant(&c, &ctx);
    assert!(errors.is_empty(), "expected no errors, got {errors:?}");
}

#[test]
fn untrusted_grant_denies() {
    let (record, mission, grant) = (record(), mission(), grant());
    let ctx = TrustContext::empty(); // no allowlist, no authenticated hash
    let errors = authorize_grant(&check(&record, &mission, &grant), &ctx);
    assert_eq!(errors, vec!["GRANT_NOT_TRUSTED: TASK-1"]);
}

#[test]
fn missing_grant_ref_denies() {
    let (mut record, mission, grant) = (record(), mission(), grant());
    record["grant_ref"] = serde_json::Value::Null;
    let ctx = trusted_ctx();
    let errors = authorize_grant(&check(&record, &mission, &grant), &ctx);
    assert_eq!(errors, vec!["GRANT_NOT_TRUSTED: TASK-1"]);
}

#[test]
fn allowlist_without_authenticated_hash_denies() {
    let (record, mission, grant) = (record(), mission(), grant());
    let mut ctx = TrustContext::empty();
    ctx.trust_grant("GRANT-1", 1); // membership only
    ctx.set_current_policy("MIS-1", "PV-1".to_string());
    ctx.set_evaluated_at(
        OffsetDateTime::parse(T0, &time::format_description::well_known::Rfc3339).unwrap(),
    );
    let errors = authorize_grant(&check(&record, &mission, &grant), &ctx);
    assert_eq!(errors, vec!["GRANT_NOT_TRUSTED: TASK-1"]);
}

#[test]
fn wrong_mission_binding_denies() {
    let (record, mut mission, grant) = (record(), mission(), grant());
    mission["id"] = json!("MIS-OTHER");
    let ctx = trusted_ctx();
    let errors = authorize_grant(&check(&record, &mission, &grant), &ctx);
    assert!(errors.contains(&"GRANT_BINDING_MISMATCH: TASK-1".to_string()));
}

#[test]
fn unapproved_state_denies() {
    let (record, mission, mut grant) = (record(), mission(), grant());
    grant["state"] = json!("proposed");
    // Keep the authenticated hash honest for the mutated record so only the
    // binding rule (not trust) can catch this.
    let mut ctx = trusted_ctx();
    ctx.authenticate_record_hash("GRANT-1", 1, record_digest(&grant).unwrap());
    let errors = authorize_grant(&check(&record, &mission, &grant), &ctx);
    assert!(errors.contains(&"GRANT_BINDING_MISMATCH: TASK-1".to_string()));
}

#[test]
fn stale_ctx_policy_denies() {
    let (record, mission, grant) = (record(), mission(), grant());
    let mut ctx = trusted_ctx();
    ctx.set_current_policy("MIS-1", "PV-2".to_string());
    let errors = authorize_grant(&check(&record, &mission, &grant), &ctx);
    assert!(errors.contains(&"GRANT_BINDING_MISMATCH: TASK-1".to_string()));
}

#[test]
fn artifact_mismatch_denies() {
    let (record, mission, grant) = (record(), mission(), grant());
    let ctx = trusted_ctx();
    let mut c = check(&record, &mission, &grant);
    c.artifact = Some(artifact('b'));
    let errors = authorize_grant(&c, &ctx);
    assert!(errors.contains(&"GRANT_BINDING_MISMATCH: TASK-1".to_string()));
}

#[test]
fn operation_mismatch_denies() {
    let (record, mission, grant) = (record(), mission(), grant());
    let ctx = trusted_ctx();
    let mut c = check(&record, &mission, &grant);
    c.operation = "OP-OTHER";
    let errors = authorize_grant(&c, &ctx);
    assert!(errors.contains(&"GRANT_BINDING_MISMATCH: TASK-1".to_string()));
}

#[test]
fn ungranted_capability_denies() {
    let (record, mission, mut grant) = (record(), mission(), grant());
    grant["capabilities"] = json!(["other_capability"]);
    let mut ctx = trusted_ctx();
    ctx.authenticate_record_hash("GRANT-1", 1, record_digest(&grant).unwrap());
    let errors = authorize_grant(&check(&record, &mission, &grant), &ctx);
    assert!(errors.contains(&"GRANT_BINDING_MISMATCH: TASK-1".to_string()));
}

#[test]
fn destination_outside_allowlist_denies() {
    let (mut record, mission, grant) = (record(), mission(), grant());
    record["destination"] = json!("remote-host");
    let ctx = trusted_ctx();
    let mut c = check(&record, &mission, &grant);
    c.destination = Some("remote-host");
    let errors = authorize_grant(&c, &ctx);
    assert!(errors.contains(&"DESTINATION_NOT_GRANTED: TASK-1".to_string()));
}

#[test]
fn cost_above_grant_cap_denies() {
    let (mut record, mission, grant) = (record(), mission(), grant());
    record["budget"]["minor_units"] = json!(900);
    let ctx = trusted_ctx();
    let errors = authorize_grant(&check(&record, &mission, &grant), &ctx);
    assert!(errors.contains(&"GRANT_COST_EXCEEDED: TASK-1".to_string()));
}

#[test]
fn currency_mismatch_denies() {
    let (mut record, mission, grant) = (record(), mission(), grant());
    record["budget"]["currency"] = json!("EUR");
    let ctx = trusted_ctx();
    let errors = authorize_grant(&check(&record, &mission, &grant), &ctx);
    assert!(errors.contains(&"GRANT_COST_EXCEEDED: TASK-1".to_string()));
}

#[test]
fn missing_budget_denies() {
    let (mut record, mission, grant) = (record(), mission(), grant());
    record.as_object_mut().unwrap().remove("budget");
    let ctx = trusted_ctx();
    let errors = authorize_grant(&check(&record, &mission, &grant), &ctx);
    assert!(errors.contains(&"GRANT_COST_EXCEEDED: TASK-1".to_string()));
}

#[test]
fn expired_grant_denies() {
    let (record, mission, grant) = (record(), mission(), grant());
    let mut ctx = trusted_ctx();
    let past = OffsetDateTime::parse(
        "2026-10-02T00:00:00+00:00",
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    ctx.set_evaluated_at(past);
    let errors = authorize_grant(&check(&record, &mission, &grant), &ctx);
    assert!(errors.contains(&"GRANT_EXPIRED_OR_TIME_UNKNOWN: TASK-1".to_string()));
}

#[test]
fn not_yet_issued_grant_denies() {
    let (record, mission, grant) = (record(), mission(), grant());
    let mut ctx = trusted_ctx();
    let early = OffsetDateTime::parse(
        "2026-09-29T00:00:00+00:00",
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    ctx.set_evaluated_at(early);
    let errors = authorize_grant(&check(&record, &mission, &grant), &ctx);
    assert!(errors.contains(&"GRANT_EXPIRED_OR_TIME_UNKNOWN: TASK-1".to_string()));
}

#[test]
fn missing_clock_denies_time_bounded_authorization() {
    let (record, mission, grant) = (record(), mission(), grant());
    let mut ctx = TrustContext::empty();
    ctx.trust_grant("GRANT-1", 1);
    ctx.authenticate_record_hash("GRANT-1", 1, record_digest(&grant).unwrap());
    ctx.set_current_policy("MIS-1", "PV-1".to_string());
    // No evaluated_at: an unknown clock must deny a time-bounded grant.
    let errors = authorize_grant(&check(&record, &mission, &grant), &ctx);
    assert!(errors.contains(&"GRANT_EXPIRED_OR_TIME_UNKNOWN: TASK-1".to_string()));
}

#[test]
fn reversed_grant_interval_is_invalid() {
    let mut g = grant();
    g["issued_at"] = json!(T1);
    g["expires_at"] = json!(T0);
    assert_eq!(
        check_grant_interval(&g),
        Some("GRANT_INVALID_INTERVAL: GRANT-1".to_string())
    );
    assert_eq!(check_grant_interval(&grant()), None);
}
