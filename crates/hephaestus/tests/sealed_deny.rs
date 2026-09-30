//! Deny-first tests for sealed-data access rules (T-004).

use hephaestus::security::digest::record_digest;
use hephaestus::security::sealed::{
    HoldoutCheck, check_confirmatory_access, check_holdout_access, check_sealed_bundle,
};
use hephaestus::security::trust::TrustContext;
use serde_json::{Value, json};

fn leak(value: Value) -> &'static Value {
    Box::leak(Box::new(value))
}

fn access() -> Value {
    json!({
        "id": "ACCESS-1", "kind": "holdout_access", "record_version": 1,
        "family_ref": {"id": "FAMILY-1", "version": 1},
        "plan_ref": {"id": "PLAN-1", "version": 1},
        "opened_at": "2026-09-29T17:00:00+00:00",
        "query_index": 1,
        "feedback": "sealed_until_campaign_end",
        "issuer_id": "SYNTHETIC-DATA-BROKER"
    })
}

fn family() -> Value {
    json!({
        "id": "FAMILY-1", "kind": "experiment_family", "record_version": 1,
        "mission_ref": {"id": "MIS-1", "version": 1},
        "plan_refs": [{"id": "PLAN-1", "version": 1}],
        "selection_history_refs": [],
        "data_partition_sha256": "1111111111111111111111111111111111111111111111111111111111111111",
        "strategy": "bonferroni_fixed_family",
        "alpha": 0.05,
        "max_confirmatory_tests": 3,
        "retired": false
    })
}

fn plan() -> Value {
    json!({
        "id": "PLAN-1", "kind": "experiment_plan", "record_version": 1,
        "registration": {"registered_at": "2026-09-29T16:00:00+00:00"}
    })
}

fn consistent() -> (Value, Value, Value) {
    (access(), family(), plan())
}

#[test]
fn consistent_access_is_permitted() {
    let (record, family, plan) = consistent();
    let errors = check_holdout_access(&HoldoutCheck {
        record: &record,
        family: Some(&family),
        plan: Some(&plan),
    });
    assert!(errors.is_empty(), "expected no errors, got {errors:?}");
}

#[test]
fn unresolvable_family_denies() {
    let (record, _family, plan) = consistent();
    let errors = check_holdout_access(&HoldoutCheck {
        record: &record,
        family: None,
        plan: Some(&plan),
    });
    assert_eq!(errors, vec!["HOLDOUT_MEMBERSHIP_MISMATCH: ACCESS-1"]);
}

#[test]
fn plan_outside_family_denies() {
    let (record, family, plan) = consistent();
    let mut family = family;
    family["plan_refs"] = json!([{"id": "OTHER-PLAN", "version": 1}]);
    let errors = check_holdout_access(&HoldoutCheck {
        record: &record,
        family: Some(&family),
        plan: Some(&plan),
    });
    assert_eq!(errors, vec!["HOLDOUT_MEMBERSHIP_MISMATCH: ACCESS-1"]);
}

#[test]
fn retired_family_denies_reuse() {
    let (record, family, plan) = consistent();
    let mut family = family;
    family["retired"] = json!(true);
    let errors = check_holdout_access(&HoldoutCheck {
        record: &record,
        family: Some(&family),
        plan: Some(&plan),
    });
    assert_eq!(errors, vec!["HOLDOUT_REUSE_FORBIDDEN: ACCESS-1"]);
}

#[test]
fn query_beyond_budget_denies() {
    let (record, family, plan) = consistent();
    let mut record = record;
    record["query_index"] = json!(4); // family allows 3
    let errors = check_holdout_access(&HoldoutCheck {
        record: &record,
        family: Some(&family),
        plan: Some(&plan),
    });
    assert_eq!(errors, vec!["HOLDOUT_REUSE_FORBIDDEN: ACCESS-1"]);
}

#[test]
fn unsealed_feedback_denies() {
    let (record, family, plan) = consistent();
    let mut record = record;
    record["feedback"] = json!("revealed_early");
    let errors = check_holdout_access(&HoldoutCheck {
        record: &record,
        family: Some(&family),
        plan: Some(&plan),
    });
    assert_eq!(errors, vec!["HOLDOUT_REUSE_FORBIDDEN: ACCESS-1"]);
}

#[test]
fn access_before_registration_denies() {
    let (record, family, plan) = consistent();
    let mut record = record;
    record["opened_at"] = json!("2026-09-29T15:00:00+00:00"); // registered at 16:00
    let errors = check_holdout_access(&HoldoutCheck {
        record: &record,
        family: Some(&family),
        plan: Some(&plan),
    });
    assert_eq!(errors, vec!["HOLDOUT_BEFORE_REGISTRATION: ACCESS-1"]);
}

#[test]
fn missing_registration_denies() {
    let (record, family, plan) = consistent();
    let mut plan = plan;
    plan["registration"]["registered_at"] = json!(null);
    let errors = check_holdout_access(&HoldoutCheck {
        record: &record,
        family: Some(&family),
        plan: Some(&plan),
    });
    assert_eq!(errors, vec!["HOLDOUT_BEFORE_REGISTRATION: ACCESS-1"]);
}

#[test]
fn duplicate_family_query_denies() {
    let a = access();
    let mut b = access();
    b["id"] = json!("ACCESS-2");
    let errors = check_sealed_bundle(&[a, b]);
    assert_eq!(errors, vec!["HOLDOUT_QUERY_DUPLICATED: bundle"]);
}

#[test]
fn reused_partition_across_families_denies() {
    let f1 = family();
    let mut f2 = family();
    f2["id"] = json!("FAMILY-2");
    let errors = check_sealed_bundle(&[f1, f2]);
    assert_eq!(errors, vec!["HOLDOUT_FAMILY_RESET_FORBIDDEN: bundle"]);
}

#[test]
fn exploratory_families_share_no_partition_claim() {
    let mut f1 = family();
    f1["strategy"] = json!("exploratory_only");
    let mut f2 = family();
    f2["id"] = json!("FAMILY-2");
    f2["strategy"] = json!("exploratory_only");
    // Duplicate partition hashes among exploratory-only families are not
    // sealed confirmatory partitions.
    let errors = check_sealed_bundle(&[f1, f2]);
    assert!(errors.is_empty(), "got {errors:?}");
}

fn trusted_access_ctx(record: &Value) -> TrustContext {
    let mut ctx = TrustContext::empty();
    ctx.trust_holdout_access("ACCESS-1", 1);
    ctx.authenticate_record_hash("ACCESS-1", 1, record_digest(record).unwrap());
    ctx
}

#[test]
fn confirmatory_access_requires_external_trust() {
    let record = leak(access());
    let plan = leak(plan());
    let owner = leak(json!({
        "id": "RES-1", "kind": "experiment_result", "record_version": 1,
        "data_opened_at": "2026-09-29T17:00:00+00:00"
    }));
    // Present but untrusted: deny.
    let ctx = TrustContext::empty();
    let errors = check_confirmatory_access(owner, plan, std::slice::from_ref(record), &ctx);
    assert_eq!(errors, vec!["CONFIRMATORY_ACCESS_NOT_VERIFIED: RES-1"]);
    // Trusted and matching: allow.
    let ctx = trusted_access_ctx(record);
    let errors = check_confirmatory_access(owner, plan, std::slice::from_ref(record), &ctx);
    assert!(errors.is_empty(), "got {errors:?}");
}

#[test]
fn confirmatory_access_time_must_match_data_opened() {
    let record = leak(access());
    let plan = leak(plan());
    let owner = leak(json!({
        "id": "RES-1", "kind": "experiment_result", "record_version": 1,
        "data_opened_at": "2026-09-29T18:00:00+00:00"
    }));
    let ctx = trusted_access_ctx(record);
    let errors = check_confirmatory_access(owner, plan, std::slice::from_ref(record), &ctx);
    assert_eq!(errors, vec!["CONFIRMATORY_ACCESS_NOT_VERIFIED: RES-1"]);
}

#[test]
fn confirmatory_access_needs_exactly_one_match() {
    let record = leak(access());
    let plan = leak(plan());
    let owner = leak(json!({
        "id": "RES-1", "kind": "experiment_result", "record_version": 1,
        "data_opened_at": "2026-09-29T17:00:00+00:00"
    }));
    let ctx = trusted_access_ctx(record);
    // No matching access at all: deny.
    let errors = check_confirmatory_access(owner, plan, &[], &ctx);
    assert_eq!(errors, vec!["CONFIRMATORY_ACCESS_NOT_VERIFIED: RES-1"]);
}
