//! Cross-record semantic validation (T-002), a faithful Rust port of the
//! reference `reference/semantic_validator.py` invariants for
//! schema-valid bundles: duplicates, finiteness, reference resolution and
//! kind, per-kind scientific/engineering guardrails, and the task DAG.
//!
//! Scope note (explicit): `validate_qualification` from the Python module is
//! NOT ported here. It requires the externally authenticated
//! `ValidationContext` trust boundary, which belongs to T-004 (local security
//! model / approval format). The Python reference tests for qualification
//! remain authoritative until that port lands. This module establishes
//! contract relationships only — never that a source is true, a permission is
//! genuine, an analysis is statistically valid, or a sandbox is secure.

use std::collections::{HashMap, HashSet};

use serde_json::Value;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

type Key = (String, i64);
type Index<'a> = HashMap<Key, &'a Value>;

/// Yield every `{id, version}` object nested anywhere inside `value`.
fn refs<'a>(value: &'a Value, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            if map.len() == 2 && map.contains_key("id") && map.contains_key("version") {
                out.push(value);
            } else {
                for item in map.values() {
                    refs(item, out);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                refs(item, out);
            }
        }
        _ => {}
    }
}

/// JSON cannot carry NaN/Infinity through serde_json (parse rejects the
/// literals, `Number::from_f64` refuses them), so this walk is defensive
/// parity with the Python reference and cannot fire on parsed input.
fn is_finite(value: &Value) -> bool {
    match value {
        Value::Number(n) => n.as_f64().is_some_and(|f| f.is_finite()) || n.as_i64().is_some(),
        Value::Array(items) => items.iter().all(is_finite),
        Value::Object(map) => map.values().all(is_finite),
        _ => true,
    }
}

fn key_of(record: &Value) -> Key {
    let id = record
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let version = record
        .get("record_version")
        .and_then(Value::as_i64)
        .unwrap_or(i64::MIN);
    (id, version)
}

/// Python-style `{'id': 'X', 'version': 1}` rendering for messages.
fn ref_repr(r: &Value) -> String {
    let id = r.get("id").and_then(Value::as_str).unwrap_or("");
    let version = r.get("version").and_then(Value::as_i64).unwrap_or(-1);
    format!("{{'id': '{id}', 'version': {version}}}")
}

fn key_repr(key: &Key) -> String {
    format!("('{}', {})", key.0, key.1)
}

/// Look up a reference, optionally requiring a specific `kind`.
fn lookup<'a>(
    index: &'a Index<'a>,
    errors: &mut Vec<String>,
    r: Option<&Value>,
    kind: Option<&str>,
) -> Option<&'a Value> {
    let r = r?;
    let key = (
        r.get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        r.get("version").and_then(Value::as_i64).unwrap_or(-1),
    );
    let found = index.get(&key)?;
    if let Some(expected) = kind
        && found.get("kind").and_then(Value::as_str) != Some(expected)
    {
        errors.push(format!(
            "WRONG_REFERENCE_KIND: {} expected {expected}",
            ref_repr(r)
        ));
        return None;
    }
    Some(found)
}

fn s<'v>(value: &'v Value, field: &str) -> Option<&'v str> {
    value.get(field).and_then(Value::as_str)
}

fn is_one_of(value: Option<&str>, set: &[&str]) -> bool {
    value.is_some_and(|v| set.contains(&v))
}

/// Return reason-coded semantic errors for a schema-valid bundle.
///
/// The input is a bundle object (`{"records": [...]}`); a missing records
/// array yields no errors, mirroring the Python `.get("records", [])`.
pub fn validate_bundle(bundle: &Value) -> Vec<String> {
    let empty: Vec<Value> = Vec::new();
    let records = bundle
        .get("records")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    let mut errors: Vec<String> = Vec::new();

    let mut index: Index<'_> = HashMap::new();
    for record in records {
        let key = key_of(record);
        if index.contains_key(&key) {
            errors.push(format!("DUPLICATE_VERSION: {}", key_repr(&key)));
        }
        index.insert(key, record);
        if !is_finite(record) {
            errors.push(format!(
                "NONFINITE_NUMBER: {}",
                record.get("id").and_then(Value::as_str).unwrap_or("")
            ));
        }
    }

    for record in records {
        let mut found: Vec<&Value> = Vec::new();
        refs(record, &mut found);
        for r in found {
            let key = (
                r.get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                r.get("version").and_then(Value::as_i64).unwrap_or(-1),
            );
            if !index.contains_key(&key) {
                errors.push(format!(
                    "MISSING_REFERENCE: {} -> {}",
                    record.get("id").and_then(Value::as_str).unwrap_or(""),
                    ref_repr(r)
                ));
            }
        }
    }

    for record in records {
        let rid = record.get("id").and_then(Value::as_str).unwrap_or("");
        let kind = record.get("kind").and_then(Value::as_str).unwrap_or("");
        let mission = lookup(
            &index,
            &mut errors,
            record.get("mission_ref"),
            Some("mission"),
        );

        if let (Some(mission), Some(budget)) = (mission, record.get("budget")) {
            let limit = mission.get("budget");
            if let (Some(limit), Some(currency), Some(limit_currency)) = (
                limit,
                budget.get("currency").and_then(Value::as_str),
                limit.and_then(|l| l.get("currency").and_then(Value::as_str)),
            ) {
                if currency != limit_currency {
                    errors.push(format!("CURRENCY_MISMATCH: {rid}"));
                } else if let (Some(units), Some(limit_units)) = (
                    budget.get("minor_units").and_then(Value::as_i64),
                    limit.get("minor_units").and_then(Value::as_i64),
                ) {
                    // Per-record cap check, NOT transactional reservation logic.
                    if units > limit_units {
                        errors.push(format!("TASK_CAP_EXCEEDS_MISSION: {rid}"));
                    }
                }
            }
        }

        match kind {
            "hypothesis" => {
                lookup(
                    &index,
                    &mut errors,
                    record.get("opportunity_ref"),
                    Some("opportunity"),
                );
                lookup(
                    &index,
                    &mut errors,
                    record.get("mechanism_ref"),
                    Some("mechanism"),
                );
                let predictions = record
                    .get("predictions")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let pids: HashSet<String> = predictions
                    .iter()
                    .filter_map(|p| s(p, "id").map(str::to_string))
                    .collect();
                if pids.len() != predictions.len() {
                    errors.push(format!("DUPLICATE_PREDICTION: {rid}"));
                }
                let falsifiers = record
                    .get("falsifiers")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                for falsifier in &falsifiers {
                    if !s(falsifier, "prediction_id").is_some_and(|p| pids.contains(p)) {
                        errors.push(format!("UNKNOWN_FALSIFIER_TARGET: {rid}"));
                    }
                }
                if is_one_of(s(record, "state"), &["test_ready", "testing"]) {
                    let covered: HashSet<&str> = falsifiers
                        .iter()
                        .filter_map(|f| s(f, "prediction_id"))
                        .collect();
                    let alternatives = record
                        .get("alternatives")
                        .and_then(Value::as_array)
                        .map(|a| !a.is_empty())
                        .unwrap_or(false);
                    if covered.len() != pids.len()
                        || covered.iter().any(|p| !pids.contains(*p))
                        || !alternatives
                    {
                        errors.push(format!("NO_CREDIBLE_DISCRIMINATOR: {rid}"));
                    }
                    let testability = record.get("testability");
                    let blocked = testability
                        .and_then(|t| s(t, "status"))
                        .is_some_and(|st| st != "accessible");
                    let has_blockers = testability
                        .and_then(|t| t.get("blockers"))
                        .and_then(Value::as_array)
                        .is_some_and(|b| !b.is_empty());
                    if blocked || has_blockers {
                        errors.push(format!("TESTABILITY_BLOCKED: {rid}"));
                    }
                }
            }
            "experiment_plan" => {
                let hypothesis = lookup(
                    &index,
                    &mut errors,
                    record.get("hypothesis_ref"),
                    Some("hypothesis"),
                );
                if let Some(hypothesis) = hypothesis {
                    let hpids: HashSet<String> = hypothesis
                        .get("predictions")
                        .and_then(Value::as_array)
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|p| s(p, "id").map(str::to_string))
                                .collect()
                        })
                        .unwrap_or_default();
                    let plan_pids = record
                        .get("prediction_ids")
                        .and_then(Value::as_array)
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|p| p.as_str().map(str::to_string))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    if plan_pids.iter().any(|p| !hpids.contains(p)) {
                        errors.push(format!("UNKNOWN_PLAN_PREDICTION: {rid}"));
                    }
                    if record.get("mission_ref") != hypothesis.get("mission_ref") {
                        errors.push(format!("MISSION_BINDING_MISMATCH: {rid}"));
                    }
                }
                let analysis = record.get("analysis");
                let registration = record.get("registration");
                let prediction_ids: HashSet<String> = record
                    .get("prediction_ids")
                    .and_then(Value::as_array)
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|p| p.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                if let Some(analysis) = analysis {
                    let primaries = analysis
                        .get("primary_endpoints")
                        .and_then(Value::as_array)
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|p| p.as_str().map(str::to_string))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    if primaries.iter().any(|p| !prediction_ids.contains(p)) {
                        errors.push(format!("UNKNOWN_PRIMARY_ENDPOINT: {rid}"));
                    }
                }
                let active = is_one_of(s(record, "status"), &["ready", "running", "completed"]);
                if active {
                    let blockers = record
                        .get("blockers")
                        .and_then(Value::as_array)
                        .is_some_and(|b| !b.is_empty());
                    if blockers {
                        errors.push(format!("ACTIVE_PLAN_BLOCKED: {rid}"));
                    }
                    if analysis
                        .and_then(|v| s(v, "method_qualification"))
                        .is_some_and(|m| m != "qualified_for_declared_assumptions")
                    {
                        errors.push(format!("UNQUALIFIED_ANALYSIS: {rid}"));
                    }
                    if analysis.and_then(|v| s(v, "design")) == Some("fixed_sample")
                        && analysis.is_some_and(|a| a.get("target_n").is_none_or(Value::is_null))
                    {
                        errors.push(format!("SAMPLE_SIZE_UNSET: {rid}"));
                    }
                    if analysis.and_then(|v| s(v, "design")) == Some("sequential")
                        && analysis.is_some_and(|a| {
                            a.get("anytime_valid_method")
                                .is_none_or(|v| v.is_null() || v == &Value::String(String::new()))
                        })
                    {
                        errors.push(format!("UNAPPROVED_OPTIONAL_STOPPING: {rid}"));
                    }
                    if analysis.and_then(|v| s(v, "data_partition")) == Some("confirmatory") {
                        let frozen = registration.and_then(|v| s(v, "status")) == Some("frozen");
                        let registered_at = registration
                            .and_then(|r| r.get("registered_at"))
                            .is_some_and(|v| !v.is_null() && v != &Value::String(String::new()));
                        let payload = registration
                            .and_then(|r| r.get("payload_sha256"))
                            .is_some_and(|v| !v.is_null() && v != &Value::String(String::new()));
                        if !frozen || !registered_at || !payload {
                            errors.push(format!("CONFIRMATION_NOT_REGISTERED: {rid}"));
                        }
                    }
                }
                if registration.and_then(|v| s(v, "status")) == Some("frozen") {
                    let registered_at = registration
                        .and_then(|r| r.get("registered_at"))
                        .is_some_and(|v| !v.is_null() && v != &Value::String(String::new()));
                    let payload = registration
                        .and_then(|r| r.get("payload_sha256"))
                        .is_some_and(|v| !v.is_null() && v != &Value::String(String::new()));
                    if !registered_at || !payload {
                        errors.push(format!("INCOMPLETE_REGISTRATION: {rid}"));
                    }
                }
            }
            "experiment_result" => {
                let plan = lookup(
                    &index,
                    &mut errors,
                    record.get("experiment_plan_ref"),
                    Some("experiment_plan"),
                );
                lookup(
                    &index,
                    &mut errors,
                    record.get("hypothesis_ref"),
                    Some("hypothesis"),
                );
                if let Some(plan) = plan
                    && record.get("hypothesis_ref") != plan.get("hypothesis_ref")
                {
                    errors.push(format!("RESULT_HYPOTHESIS_MISMATCH: {rid}"));
                }
                if s(record, "execution_validity") != Some("valid") {
                    if is_one_of(
                        s(record, "scientific_conclusion"),
                        &["supported", "contradicted"],
                    ) || is_one_of(s(record, "engineering_target"), &["met", "not_met"])
                    {
                        errors.push(format!("INVALID_EXECUTION_CONCLUSION: {rid}"));
                    }
                } else {
                    let has_raw = record
                        .get("raw_artifact_hashes")
                        .and_then(Value::as_array)
                        .is_some_and(|a| !a.is_empty());
                    let has_analysis = record
                        .get("analysis_artifact_sha256")
                        .is_some_and(|v| !v.is_null() && v != &Value::String(String::new()));
                    if !has_raw || !has_analysis {
                        errors.push(format!("RESULT_RECEIPTS_MISSING: {rid}"));
                    }
                }
                let plan_pids: HashSet<String> = plan
                    .and_then(|p| p.get("prediction_ids"))
                    .and_then(Value::as_array)
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|p| p.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                let findings = record
                    .get("findings")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                for finding in &findings {
                    let pid = s(finding, "prediction_id").unwrap_or("");
                    if plan.is_some() && !plan_pids.contains(pid) {
                        errors.push(format!("UNKNOWN_RESULT_PREDICTION: {rid}"));
                    }
                    let low = finding.get("lower_bound").and_then(Value::as_f64);
                    let high = finding.get("upper_bound").and_then(Value::as_f64);
                    if let (Some(low), Some(high)) = (low, high)
                        && low > high
                    {
                        errors.push(format!("REVERSED_INTERVAL: {rid}"));
                    }
                }
                if let Some(plan) = plan {
                    let opened = s(record, "data_opened_at");
                    let confirmatory = plan.get("analysis").and_then(|a| s(a, "data_partition"))
                        == Some("confirmatory");
                    if let (Some(opened), true) = (opened, confirmatory) {
                        let registered =
                            plan.get("registration").and_then(|r| s(r, "registered_at"));
                        if let Some(registered) = registered
                            && !registered.is_empty()
                        {
                            let opened_t = OffsetDateTime::parse(opened, &Rfc3339).ok();
                            let registered_t = OffsetDateTime::parse(registered, &Rfc3339).ok();
                            // Unparseable timestamps are schema-layer failures;
                            // the Python reference would raise there.
                            if let (Some(opened_t), Some(registered_t)) = (opened_t, registered_t)
                                && opened_t < registered_t
                            {
                                errors.push(format!("DATA_BEFORE_REGISTRATION: {rid}"));
                            }
                        }
                    }
                }
            }
            "task" => {
                if is_one_of(
                    s(record, "effect_class"),
                    &["unknown", "external_non_idempotent"],
                ) && s(record, "retry_policy") == Some("safe_retry")
                {
                    errors.push(format!("UNSAFE_RETRY: {rid}"));
                }
                if s(record, "effect_class") == Some("unknown")
                    && is_one_of(s(record, "state"), &["ready", "running"])
                {
                    errors.push(format!("UNKNOWN_EFFECT_DISPATCH: {rid}"));
                }
                if let Some(deps) = record.get("dependency_refs").and_then(Value::as_array) {
                    for dep in deps {
                        lookup(&index, &mut errors, Some(dep), Some("task"));
                    }
                }
                if is_one_of(s(record, "state"), &["ready", "running"])
                    && let Some(mission) = mission
                {
                    {
                        let auth = mission.get("authorization");
                        if auth.and_then(|v| s(v, "state")) != Some("approved") {
                            errors.push(format!("AUTHORIZATION_NOT_APPROVED: {rid}"));
                        }
                        let allowed: HashSet<&str> = auth
                            .and_then(|a| a.get("allowed_capabilities"))
                            .and_then(Value::as_array)
                            .map(|arr| arr.iter().filter_map(|c| c.as_str()).collect())
                            .unwrap_or_default();
                        let required = record
                            .get("required_capabilities")
                            .and_then(Value::as_array)
                            .map(|arr| arr.iter().filter_map(|c| c.as_str()).collect::<Vec<_>>())
                            .unwrap_or_default();
                        if required.iter().any(|c| !allowed.contains(c)) {
                            errors.push(format!("CAPABILITY_NOT_GRANTED: {rid}"));
                        }
                    }
                }
            }
            "decision" => {
                lookup(&index, &mut errors, record.get("task_ref"), Some("task"));
                let selected = record.get("selected_option");
                let options = record
                    .get("options")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                if let Some(selected) = selected
                    && !selected.is_null()
                    && !options.contains(selected)
                {
                    errors.push(format!("INVALID_DECISION_OPTION: {rid}"));
                }
                if let Some(probs) = record.get("probabilities")
                    && !probs.is_null()
                {
                    {
                        let values: Vec<f64> = probs
                            .as_array()
                            .map(|a| a.iter().filter_map(Value::as_f64).collect())
                            .unwrap_or_default();
                        let sum: f64 = values.iter().sum();
                        if values.len() != options.len() || (sum - 1.0).abs() > 1e-6 {
                            errors.push(format!("INVALID_PROBABILITY_VECTOR: {rid}"));
                        }
                    }
                }
                if s(record, "calibration_status") == Some("qualified_for_declared_domain") {
                    let receipt = record
                        .get("calibration_artifact_sha256")
                        .is_some_and(|v| !v.is_null() && v != &Value::String(String::new()));
                    if !receipt {
                        errors.push(format!("CALIBRATION_RECEIPT_MISSING: {rid}"));
                    }
                }
                if record.get("abstained") == Some(&Value::Bool(true))
                    && selected.is_some()
                    && !selected.is_some_and(Value::is_null)
                {
                    errors.push(format!("ABSTENTION_CONFLICT: {rid}"));
                }
            }
            "dossier" => {
                let mut result_records = Vec::new();
                if let Some(result_refs) = record.get("result_refs").and_then(Value::as_array) {
                    for r in result_refs {
                        if let Some(found) =
                            lookup(&index, &mut errors, Some(r), Some("experiment_result"))
                        {
                            result_records.push(found);
                        }
                    }
                }
                if let Some(hyp_refs) = record.get("hypothesis_refs").and_then(Value::as_array) {
                    for r in hyp_refs {
                        lookup(&index, &mut errors, Some(r), Some("hypothesis"));
                    }
                }
                if s(record, "status") == Some("validated_candidate") {
                    let blockers = record
                        .get("unresolved_blockers")
                        .and_then(Value::as_array)
                        .is_some_and(|b| !b.is_empty());
                    if blockers {
                        errors.push(format!("PROMOTION_BLOCKED: {rid}"));
                    }
                    let qualifying = result_records.iter().any(|r| {
                        s(r, "execution_validity") == Some("valid")
                            && s(r, "scientific_conclusion") == Some("supported")
                            && s(r, "engineering_target") == Some("met")
                    });
                    if !qualifying {
                        errors.push(format!("PROMOTION_EVIDENCE_MISSING: {rid}"));
                    }
                    if s(record, "reproduction_status") != Some("independent_pass")
                        || record
                            .get("reproduction_artifact_sha256")
                            .is_none_or(|v| v.is_null() || v == &Value::String(String::new()))
                    {
                        errors.push(format!("REPRODUCTION_MISSING: {rid}"));
                    }
                    let novelty = record
                        .get("novelty_report")
                        .and_then(|n| s(n, "status"))
                        .unwrap_or("");
                    if !matches!(novelty, "near_match" | "no_match_within_search_scope") {
                        errors.push(format!("NOVELTY_UNRESOLVED_FOR_PROMOTION: {rid}"));
                    }
                }
                if s(record, "reproduction_status") == Some("independent_pass") {
                    let receipt = record
                        .get("reproduction_artifact_sha256")
                        .is_some_and(|v| !v.is_null() && v != &Value::String(String::new()));
                    if !receipt {
                        errors.push(format!("REPRODUCTION_RECEIPT_MISSING: {rid}"));
                    }
                }
            }
            _ => {}
        }
    }

    // DAG validation is independent of schema validation.
    let tasks: HashMap<&Key, &Value> = index
        .iter()
        .filter(|(_, r)| r.get("kind").and_then(Value::as_str) == Some("task"))
        .map(|(k, r)| (k, *r))
        .collect();
    let mut visiting: HashSet<Key> = HashSet::new();
    let mut visited: HashSet<Key> = HashSet::new();
    let task_keys: Vec<Key> = tasks.keys().map(|k| (*k).clone()).collect();
    for key in &task_keys {
        visit(key, &tasks, &mut visiting, &mut visited, &mut errors);
    }
    errors
}

fn visit<'a>(
    key: &Key,
    tasks: &HashMap<&'a Key, &'a Value>,
    visiting: &mut HashSet<Key>,
    visited: &mut HashSet<Key>,
    errors: &mut Vec<String>,
) {
    if visiting.contains(key) {
        errors.push(format!("TASK_DAG_CYCLE: {}", key_repr(key)));
        return;
    }
    if visited.contains(key) || !tasks.contains_key(key) {
        return;
    }
    visiting.insert(key.clone());
    if let Some(task) = tasks.get(key)
        && let Some(deps) = task.get("dependency_refs").and_then(Value::as_array)
    {
        for dep in deps {
            let dep_key = (
                dep.get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                dep.get("version").and_then(Value::as_i64).unwrap_or(-1),
            );
            visit(&dep_key, tasks, visiting, visited, errors);
        }
    }
    visiting.remove(key);
    visited.insert(key.clone());
}
