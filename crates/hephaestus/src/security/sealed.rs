//! Sealed-data access rules (T-004, R-101).
//!
//! Port of the `holdout_access` checks, bundle-level reset/duplication
//! guards, and confirmatory-access verification from
//! `reference/qualification.py`. Sealed feedback stays sealed: access must
//! belong to a live family, sit inside the declared test budget, postdate
//! registration, and — for confirmatory results — be exactly one externally
//! trusted access whose opening matches the recorded data-opened time.

use std::collections::HashSet;

use serde_json::Value;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

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

fn parse_time(value: Option<&str>) -> Option<OffsetDateTime> {
    value.and_then(|v| OffsetDateTime::parse(v, &Rfc3339).ok())
}

/// Inputs for one sealed-access record check.
pub struct HoldoutCheck<'a> {
    /// The `holdout_access` record under examination.
    pub record: &'a Value,
    /// The family resolved from `family_ref` (None = unresolvable).
    pub family: Option<&'a Value>,
    /// The plan resolved from `plan_ref` (None = unresolvable).
    pub plan: Option<&'a Value>,
}

/// Per-record sealed-access rules. Empty = the access is structurally
/// permitted; trust of the access itself is checked separately via
/// [`TrustContext::trusted_holdout_access`] and the confirmatory check.
pub fn check_holdout_access(check: &HoldoutCheck<'_>) -> Vec<String> {
    let record = check.record;
    let mut errors = Vec::new();

    let membership_ok = match (check.family, check.plan) {
        (Some(family), Some(plan)) => {
            let plan_ref = ref_of(plan);
            family
                .get("plan_refs")
                .and_then(Value::as_array)
                .is_some_and(|refs| refs.contains(&plan_ref))
        }
        _ => false,
    };
    if !membership_ok {
        errors.push(error("HOLDOUT_MEMBERSHIP_MISMATCH", record));
        return errors;
    }
    let family = check.family.expect("membership established");

    let retired = family.get("retired").and_then(Value::as_bool) == Some(true);
    let max_tests = family.get("max_confirmatory_tests").and_then(Value::as_i64);
    let query_index = record.get("query_index").and_then(Value::as_i64);
    let over_budget = match (query_index, max_tests) {
        (Some(q), Some(m)) => q > m,
        _ => true, // missing facts fail closed
    };
    let sealed =
        record.get("feedback").and_then(Value::as_str) == Some("sealed_until_campaign_end");
    if retired || over_budget || !sealed {
        errors.push(error("HOLDOUT_REUSE_FORBIDDEN", record));
    }

    if let Some(plan) = check.plan {
        let registered = plan
            .get("registration")
            .and_then(|r| r.get("registered_at"))
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty());
        let opened = record.get("opened_at").and_then(Value::as_str);
        let ok = match (registered, opened) {
            (Some(registered), Some(opened)) => {
                match (parse_time(Some(opened)), parse_time(Some(registered))) {
                    (Some(opened), Some(registered)) => opened >= registered,
                    _ => false, // unparseable times fail closed
                }
            }
            _ => false, // missing registration or opening fails closed
        };
        if !ok {
            errors.push(error("HOLDOUT_BEFORE_REGISTRATION", record));
        }
    }

    errors
}

/// Bundle-level sealed-data guards: no duplicated queries against a family,
/// no reused partition across non-exploratory families.
pub fn check_sealed_bundle(records: &[Value]) -> Vec<String> {
    let mut errors = Vec::new();

    let mut seen_queries: HashSet<(String, i64, i64)> = HashSet::new();
    let mut families: Vec<&Value> = Vec::new();
    for record in records {
        match record.get("kind").and_then(Value::as_str) {
            Some("holdout_access") => {
                let family_ref = record.get("family_ref");
                let family_id = family_ref
                    .and_then(|r| r.get("id"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let family_version = family_ref
                    .and_then(|r| r.get("version"))
                    .and_then(Value::as_i64);
                let query_index = record.get("query_index").and_then(Value::as_i64);
                if let (Some(version), Some(query)) = (family_version, query_index)
                    && !seen_queries.insert((family_id.to_string(), version, query))
                {
                    errors.push("HOLDOUT_QUERY_DUPLICATED: bundle".to_string());
                }
            }
            Some("experiment_family")
                if record.get("strategy").and_then(Value::as_str) != Some("exploratory_only") =>
            {
                families.push(record);
            }
            _ => {}
        }
    }

    let mut seen_partitions: HashSet<&str> = HashSet::new();
    for family in families {
        if let Some(partition) = family.get("data_partition_sha256").and_then(Value::as_str)
            && !seen_partitions.insert(partition)
        {
            errors.push("HOLDOUT_FAMILY_RESET_FORBIDDEN: bundle".to_string());
        }
    }

    errors
}

/// Confirmatory results must be opened through exactly one externally
/// trusted sealed access whose `opened_at` matches the recorded time.
/// `owner` is the experiment result claiming the access.
pub fn check_confirmatory_access(
    owner: &Value,
    plan: &Value,
    records: &[Value],
    ctx: &TrustContext,
) -> Vec<String> {
    let plan_ref = ref_of(plan);
    let opened = owner.get("data_opened_at").and_then(Value::as_str);
    let matching: Vec<&Value> = records
        .iter()
        .filter(|r| r.get("kind").and_then(Value::as_str) == Some("holdout_access"))
        .filter(|r| r.get("plan_ref") == Some(&plan_ref))
        .filter(|r| {
            let id = r.get("id").and_then(Value::as_str).unwrap_or("");
            let version = r
                .get("record_version")
                .and_then(Value::as_i64)
                .unwrap_or(-1);
            ctx.trusted_holdout_access(id, version, r)
        })
        .collect();
    let ok = matching.len() == 1 && matching[0].get("opened_at").and_then(Value::as_str) == opened;
    if ok {
        Vec::new()
    } else {
        vec![error("CONFIRMATORY_ACCESS_NOT_VERIFIED", owner)]
    }
}
