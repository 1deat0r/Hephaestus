//! Scoped grant authorization (T-004, R-099 dispatch-time rules).
//!
//! Port of `reference/qualification.py::check_grant` plus the grant interval
//! rule. Fail-closed: trust comes from the [`TrustContext`], clock facts from
//! its `evaluated_at`, and every binding must match exactly. These checks
//! authorize *before* dispatch or effect — no worker grants itself anything.

use serde_json::Value;

use super::trust::TrustContext;

/// Everything a single grant check compares against.
pub struct GrantCheck<'a> {
    /// The record seeking authorization (task / plan / improvement).
    pub record: &'a Value,
    /// The mission the grant must bind to.
    pub mission: &'a Value,
    /// The resolved `authorization_grant` record.
    pub grant: &'a Value,
    /// Approval/intervention artifact digest (nullable per contract).
    pub artifact: Option<&'a str>,
    /// Capabilities the operation requires.
    pub capabilities: &'a [&'a str],
    /// Operation identity the grant must cover.
    pub operation: &'a str,
    /// Execution destination (nullable per contract).
    pub destination: Option<&'a str>,
}

fn record_id(record: &Value) -> String {
    record
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("<unknown>")
        .to_string()
}

fn error(code: &str, record: &Value) -> String {
    format!("{code}: {}", record_id(record))
}

fn ref_of(record: &Value) -> Value {
    serde_json::json!({
        "id": record.get("id").and_then(Value::as_str).unwrap_or(""),
        "version": record.get("record_version").and_then(Value::as_i64).unwrap_or(-1),
    })
}

fn subset(required: &[&str], allowed: &Value) -> bool {
    let Some(allowed) = allowed.as_array() else {
        return false;
    };
    let allowed: Vec<&str> = allowed.iter().filter_map(Value::as_str).collect();
    required.iter().all(|cap| allowed.contains(cap))
}

/// Authorize one operation against its grant. Returns reason-coded errors;
/// empty means the exact scoped grant currently holds.
pub fn authorize_grant(check: &GrantCheck<'_>, ctx: &TrustContext) -> Vec<String> {
    let GrantCheck {
        record,
        mission,
        grant,
        artifact,
        capabilities,
        operation,
        destination,
    } = check;
    let mut errors = Vec::new();

    // Trust: the grant must be externally trusted, not merely present.
    let trusted = record
        .get("grant_ref")
        .and_then(|r| {
            let id = r.get("id")?.as_str()?;
            let version = r.get("version")?.as_i64()?;
            Some(ctx.trusted_grant(id, version, grant))
        })
        .unwrap_or(false);
    if !trusted {
        errors.push(error("GRANT_NOT_TRUSTED", record));
        return errors;
    }

    let auth = mission.get("authorization");
    let mission_ref_ok =
        grant.get("mission_ref").cloned().unwrap_or(Value::Null) == ref_of(mission);
    let state_ok = grant.get("state").and_then(Value::as_str) == Some("approved");
    let grant_policy = grant.get("policy_version").and_then(Value::as_str);
    let mission_policy = auth
        .and_then(|a| a.get("policy_version"))
        .and_then(Value::as_str);
    let ctx_policy = mission
        .get("id")
        .and_then(Value::as_str)
        .and_then(|mission_id| ctx.current_policy(mission_id));
    let grant_artifact = grant.get("artifact_sha256").and_then(Value::as_str);
    let artifact_ok = grant_artifact == *artifact;
    let operation_ok = grant.get("operation_id").and_then(Value::as_str) == Some(operation);
    let grant_destination = grant.get("destination");
    let destination_ok = match *destination {
        Some(wanted) => grant_destination.is_some_and(|g| g.as_str() == Some(wanted)),
        None => grant_destination.is_some_and(Value::is_null),
    };
    let grant_caps_ok = subset(
        capabilities,
        grant.get("capabilities").unwrap_or(&Value::Null),
    );
    let mission_caps_ok = subset(
        capabilities,
        auth.and_then(|a| a.get("allowed_capabilities"))
            .unwrap_or(&Value::Null),
    );
    let mission_state_ok =
        auth.and_then(|a| a.get("state")).and_then(Value::as_str) == Some("approved");

    let binding_ok = mission_ref_ok
        && state_ok
        && grant_policy.is_some()
        && grant_policy == mission_policy
        && ctx_policy.is_some()
        && ctx_policy == grant_policy
        && artifact_ok
        && operation_ok
        && destination_ok
        && grant_caps_ok
        && mission_caps_ok
        && mission_state_ok;
    if !binding_ok {
        errors.push(error("GRANT_BINDING_MISMATCH", record));
    }

    if destination.is_some() {
        let allowed = auth
            .and_then(|a| a.get("allowed_destinations"))
            .and_then(Value::as_array);
        let granted = allowed
            .map(|list| list.iter().any(|d| d.as_str() == *destination))
            .unwrap_or(false);
        if !granted {
            errors.push(error("DESTINATION_NOT_GRANTED", record));
        }
    }

    // Budget cap against the grant (per-record cap, not reservations).
    let Some(budget) = record.get("budget") else {
        // Schema-valid records always carry a budget where required; a
        // missing one here fails closed rather than skipping the cost rule.
        errors.push(error("GRANT_COST_EXCEEDED", record));
        return errors;
    };
    let Some(max_cost) = grant.get("max_cost") else {
        errors.push(error("GRANT_COST_EXCEEDED", record));
        return errors;
    };
    let currency_ok = budget.get("currency") == max_cost.get("currency");
    let within_cap = match (
        budget.get("minor_units").and_then(Value::as_i64),
        max_cost.get("minor_units").and_then(Value::as_i64),
    ) {
        (Some(used), Some(cap)) => used <= cap,
        _ => false,
    };
    if !currency_ok || !within_cap {
        errors.push(error("GRANT_COST_EXCEEDED", record));
    }

    // Time window: issued <= now < expires, with an external clock.
    let now = ctx.evaluated_at();
    let issued = grant.get("issued_at").and_then(Value::as_str);
    let expires = grant.get("expires_at").and_then(Value::as_str);
    let window_ok = match (now, issued, expires) {
        (Some(now), Some(issued), Some(expires)) => match (
            time::OffsetDateTime::parse(issued, &time::format_description::well_known::Rfc3339),
            time::OffsetDateTime::parse(expires, &time::format_description::well_known::Rfc3339),
        ) {
            (Ok(issued), Ok(expires)) => issued <= now && now < expires,
            _ => false,
        },
        _ => false,
    };
    if !window_ok {
        errors.push(error("GRANT_EXPIRED_OR_TIME_UNKNOWN", record));
    }

    errors
}

/// `GRANT_INVALID_INTERVAL`: a grant whose window is empty or reversed can
/// never authorize anything. Returns the error or `None`.
pub fn check_grant_interval(grant: &Value) -> Option<String> {
    let issued = grant.get("issued_at").and_then(Value::as_str);
    let expires = grant.get("expires_at").and_then(Value::as_str);
    let valid = match (issued, expires) {
        (Some(issued), Some(expires)) => match (
            time::OffsetDateTime::parse(issued, &time::format_description::well_known::Rfc3339),
            time::OffsetDateTime::parse(expires, &time::format_description::well_known::Rfc3339),
        ) {
            (Ok(issued), Ok(expires)) => issued < expires,
            _ => false,
        },
        _ => false,
    };
    if valid {
        None
    } else {
        Some(error("GRANT_INVALID_INTERVAL", grant))
    }
}
