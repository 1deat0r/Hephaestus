//! Parity tests against `tests/test_reference_contracts.py` (T-002).
//!
//! Each test mirrors one Python reference test: mutate the schema-valid
//! example bundle the same way, then assert the same reason-coded prefix.
//! Qualification/trust-context codes are NOT mirrored here — that port needs
//! the external trust boundary of T-004 (see `semantic.rs` module docs).

use hephaestus::contracts::ContractRecord;
use hephaestus::semantic::validate_bundle;
use serde_json::{Value, json};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/software-mission.json"
);

fn base() -> Value {
    let text = std::fs::read_to_string(EXAMPLE).expect("fixture readable");
    serde_json::from_str(&text).expect("fixture is valid JSON")
}

fn record_mut<'a>(bundle: &'a mut Value, kind: &str) -> &'a mut Value {
    bundle["records"]
        .as_array_mut()
        .expect("records")
        .iter_mut()
        .find(|r| r["kind"] == Value::String(kind.to_string()))
        .unwrap_or_else(|| panic!("fixture record of kind {kind}"))
}

fn has(errors: &[String], prefix: &str) {
    assert!(
        errors.iter().any(|e| e.starts_with(prefix)),
        "expected an error starting with {prefix:?}, got {errors:?}"
    );
}

#[test]
fn test_example_schema_and_semantics() {
    let bundle = base();
    for record in bundle["records"].as_array().unwrap() {
        ContractRecord::from_value(record.clone()).expect("fixture must parse as typed record");
    }
    assert_eq!(validate_bundle(&bundle), Vec::<String>::new());
}

#[test]
fn test_duplicate_version_rejected() {
    let mut bundle = base();
    let mission = bundle["records"][0].clone();
    bundle["records"].as_array_mut().unwrap().push(mission);
    has(&validate_bundle(&bundle), "DUPLICATE_VERSION");
}

#[test]
fn test_missing_reference_rejected() {
    let mut bundle = base();
    record_mut(&mut bundle, "hypothesis")["mission_ref"]["version"] = 99.into();
    has(&validate_bundle(&bundle), "MISSING_REFERENCE");
}

#[test]
fn test_test_ready_needs_discriminator() {
    let mut bundle = base();
    let h = record_mut(&mut bundle, "hypothesis");
    h["state"] = json!("test_ready");
    h["falsifiers"] = json!([]);
    has(&validate_bundle(&bundle), "NO_CREDIBLE_DISCRIMINATOR");
}

#[test]
fn test_test_ready_cannot_be_blocked() {
    let mut bundle = base();
    record_mut(&mut bundle, "hypothesis")["state"] = json!("test_ready");
    has(&validate_bundle(&bundle), "TESTABILITY_BLOCKED");
}

#[test]
fn test_unknown_prediction_rejected() {
    let mut bundle = base();
    record_mut(&mut bundle, "experiment_plan")["prediction_ids"]
        .as_array_mut()
        .unwrap()
        .push(json!("PRED-MISSING"));
    has(&validate_bundle(&bundle), "UNKNOWN_PLAN_PREDICTION");
}

#[test]
fn test_confirmatory_registration_required() {
    let mut bundle = base();
    let p = record_mut(&mut bundle, "experiment_plan");
    p["status"] = json!("ready");
    p["analysis"]["data_partition"] = json!("confirmatory");
    has(&validate_bundle(&bundle), "CONFIRMATION_NOT_REGISTERED");
}

#[test]
fn test_fixed_sample_requires_n() {
    let mut bundle = base();
    record_mut(&mut bundle, "experiment_plan")["status"] = json!("ready");
    has(&validate_bundle(&bundle), "SAMPLE_SIZE_UNSET");
}

#[test]
// R-040/AT-040: optional stopping (repeated looks) requires a
// separately approved anytime-valid method; without it the plan is
// rejected at validation, before data access.
fn test_optional_stopping_requires_method() {
    let mut bundle = base();
    let p = record_mut(&mut bundle, "experiment_plan");
    p["status"] = json!("ready");
    p["analysis"]["design"] = json!("sequential");
    has(&validate_bundle(&bundle), "UNAPPROVED_OPTIONAL_STOPPING");
}

#[test]
fn test_incomplete_execution_cannot_support() {
    let mut bundle = base();
    record_mut(&mut bundle, "experiment_result")["scientific_conclusion"] = json!("supported");
    has(&validate_bundle(&bundle), "INVALID_EXECUTION_CONCLUSION");
}

#[test]
fn test_valid_result_requires_receipts() {
    let mut bundle = base();
    record_mut(&mut bundle, "experiment_result")["execution_validity"] = json!("valid");
    has(&validate_bundle(&bundle), "RESULT_RECEIPTS_MISSING");
}

#[test]
fn test_data_access_cannot_precede_registration() {
    let mut bundle = base();
    let p = record_mut(&mut bundle, "experiment_plan");
    p["analysis"]["data_partition"] = json!("confirmatory");
    p["registration"]["registered_at"] = json!("2026-09-30T05:15:00+13:00");
    record_mut(&mut bundle, "experiment_result")["data_opened_at"] =
        json!("2026-09-30T04:15:00+13:00");
    has(&validate_bundle(&bundle), "DATA_BEFORE_REGISTRATION");
}

#[test]
fn test_non_idempotent_blind_retry_rejected() {
    let mut bundle = base();
    record_mut(&mut bundle, "task")["effect_class"] = json!("external_non_idempotent");
    has(&validate_bundle(&bundle), "UNSAFE_RETRY");
}

#[test]
fn test_unknown_effect_not_dispatched() {
    let mut bundle = base();
    let t = record_mut(&mut bundle, "task");
    t["effect_class"] = json!("unknown");
    t["state"] = json!("ready");
    has(&validate_bundle(&bundle), "UNKNOWN_EFFECT_DISPATCH");
}

#[test]
fn test_unapproved_mission_not_dispatched() {
    let mut bundle = base();
    record_mut(&mut bundle, "task")["state"] = json!("ready");
    has(&validate_bundle(&bundle), "AUTHORIZATION_NOT_APPROVED");
}

#[test]
fn test_capability_not_implicitly_granted() {
    let mut bundle = base();
    let t = record_mut(&mut bundle, "task");
    t["state"] = json!("ready");
    t["required_capabilities"] = json!(["public_disclosure"]);
    has(&validate_bundle(&bundle), "CAPABILITY_NOT_GRANTED");
}

#[test]
fn test_task_cap_respects_mission() {
    let mut bundle = base();
    record_mut(&mut bundle, "task")["budget"]["minor_units"] = 999999.into();
    has(&validate_bundle(&bundle), "TASK_CAP_EXCEEDS_MISSION");
}

#[test]
fn test_task_cycle_rejected() {
    let mut bundle = base();
    let task_id = bundle["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "task")
        .map(|r| r["id"].as_str().unwrap().to_string())
        .expect("task in fixture");
    record_mut(&mut bundle, "task")["dependency_refs"] = json!([{"id": task_id, "version": 1}]);
    has(&validate_bundle(&bundle), "TASK_DAG_CYCLE");
}

#[test]
fn test_model_decision_cannot_be_authoritative() {
    let mut bundle = base();
    record_mut(&mut bundle, "decision")["authoritative"] = json!(true);
    // Python asserts this at the schema layer; the typed contract enforces
    // the same `const: false` through contract validation.
    let decision = bundle["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "decision")
        .expect("decision")
        .clone();
    let err = ContractRecord::from_value(decision).expect_err("const false violated");
    assert!(!err.violations().is_empty(), "got {err}");
}

#[test]
fn test_probability_vector_consistency() {
    let mut bundle = base();
    record_mut(&mut bundle, "decision")["probabilities"] = json!([0.9, 0.9]);
    has(&validate_bundle(&bundle), "INVALID_PROBABILITY_VECTOR");
}

#[test]
fn test_abstention_has_no_selected_choice() {
    let mut bundle = base();
    record_mut(&mut bundle, "decision")["selected_option"] = json!("ready");
    has(&validate_bundle(&bundle), "ABSTENTION_CONFLICT");
}

#[test]
fn test_model_calibration_needs_receipt() {
    let mut bundle = base();
    record_mut(&mut bundle, "decision")["calibration_status"] =
        json!("qualified_for_declared_domain");
    has(&validate_bundle(&bundle), "CALIBRATION_RECEIPT_MISSING");
}

#[test]
fn test_promotion_needs_real_qualifying_result() {
    let mut bundle = base();
    record_mut(&mut bundle, "dossier")["status"] = json!("validated_candidate");
    has(&validate_bundle(&bundle), "PROMOTION_EVIDENCE_MISSING");
}

#[test]
fn test_promotion_requires_reproduction() {
    let mut bundle = base();
    record_mut(&mut bundle, "dossier")["status"] = json!("validated_candidate");
    has(&validate_bundle(&bundle), "REPRODUCTION_MISSING");
}

#[test]
fn test_promotion_does_not_ignore_novelty_scope() {
    let mut bundle = base();
    record_mut(&mut bundle, "dossier")["status"] = json!("validated_candidate");
    has(
        &validate_bundle(&bundle),
        "NOVELTY_UNRESOLVED_FOR_PROMOTION",
    );
}

#[test]
fn test_nonfinite_data_rejected() {
    // The Python reference walks raw dicts, where float('nan') can exist, and
    // reports NONFINITE_NUMBER. serde_json cannot represent NaN/Infinity in a
    // Value at all (JSON literals are rejected at parse time), so the equivalent
    // protection is: NaN never enters the system.
    assert!(serde_json::from_str::<Value>(r#"{"threshold": NaN}"#).is_err());
    assert!(serde_json::from_str::<Value>(r#"{"threshold": Infinity}"#).is_err());
    let mut bundle = base();
    // Negative control: the same bundle stays clean when parseable.
    record_mut(&mut bundle, "hypothesis");
    assert_eq!(validate_bundle(&bundle), Vec::<String>::new());
}

#[test]
fn test_reversed_interval_rejected() {
    let mut bundle = base();
    record_mut(&mut bundle, "experiment_result")["findings"] = json!([{
        "prediction_id": "PRED-TIME",
        "estimate": 0.2,
        "unit": "fraction",
        "lower_bound": 0.4,
        "upper_bound": 0.1,
        "interpretation": "Synthetic invalid bounds"
    }]);
    has(&validate_bundle(&bundle), "REVERSED_INTERVAL");
}

#[test]
fn test_compiled_blocked_example_is_not_deleted() {
    let bundle = base();
    let h = &bundle["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "hypothesis")
        .expect("hypothesis");
    assert_eq!(h["state"], "compiled");
    assert_eq!(h["testability"]["status"], "blocked");
    assert_eq!(validate_bundle(&bundle), Vec::<String>::new());
}
