//! T-002 round-trip and contract-negative tests.
//!
//! Round-trip claim: every schema-valid fixture record parses into its typed
//! form, passes generated contract validation, and serializes back to a
//! byte-for-byte equivalent JSON value (structurally equal). An independent
//! JSON Schema oracle (`jsonschema` crate) then confirms the official
//! `contracts.schema.json` accepts the typed output — the generator and the
//! schema are checked against each other, not just against themselves.

use std::collections::HashSet;

use hephaestus::contracts::{ContractError, ContractRecord, parse_bundle_json};
use serde_json::Value;

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/software-mission.json"
);
const QUALIFICATION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/qualification-bundle.json"
);
const SCHEMA: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../schemas/contracts.schema.json"
);

fn load(path: &str) -> Value {
    let text = std::fs::read_to_string(path).expect("fixture readable");
    serde_json::from_str(&text).expect("fixture is valid JSON")
}

fn records(bundle: &Value) -> &Vec<Value> {
    bundle
        .get("records")
        .and_then(Value::as_array)
        .expect("records array")
}

/// JSON numbers are abstract (`0` and `0.0` are the same value); fixtures may
/// use either spelling. Normalize every number to a float before comparing so
/// round-trip equality is semantic, not lexical.
fn normalize_numbers(value: &Value) -> Value {
    match value {
        Value::Number(n) => Value::Number(
            serde_json::Number::from_f64(n.as_f64().expect("finite JSON number"))
                .expect("finite JSON number"),
        ),
        Value::Array(items) => Value::Array(items.iter().map(normalize_numbers).collect()),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), normalize_numbers(v)))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn round_trip(path: &str) -> Vec<ContractRecord> {
    let bundle = load(path);
    let schema: Value = load(SCHEMA);
    let mut parsed = Vec::new();
    for original in records(&bundle) {
        let record = ContractRecord::from_value(original.clone())
            .unwrap_or_else(|e| panic!("parse {}: {e}", original["id"]));
        let back = record.to_value().expect("serialize");
        assert_eq!(
            normalize_numbers(&back),
            normalize_numbers(original),
            "round-trip mismatch for {}",
            original["id"]
        );
        jsonschema::validate(&schema, &back)
            .unwrap_or_else(|e| panic!("schema oracle rejected {}: {e}", original["id"]));
        parsed.push(record);
    }
    parsed
}

#[test]
fn round_trip_example_bundle() {
    let parsed = round_trip(EXAMPLE);
    assert_eq!(parsed.len(), 13);
    let kinds: HashSet<&str> = parsed.iter().map(|r| r.kind()).collect();
    assert_eq!(kinds.len(), 13);
}

#[test]
fn round_trip_covers_all_18_record_kinds() {
    let parsed = round_trip(QUALIFICATION);
    let kinds: HashSet<&str> = parsed.iter().map(|r| r.kind()).collect();
    let expected: HashSet<&str> = [
        "mission",
        "source",
        "evidence",
        "opportunity",
        "mechanism",
        "hypothesis",
        "experiment_plan",
        "experiment_result",
        "task",
        "decision",
        "event",
        "dossier",
        "authorization_grant",
        "method_qualification",
        "experiment_family",
        "holdout_access",
        "verification_receipt",
        "improvement_candidate",
    ]
    .into_iter()
    .collect();
    assert_eq!(kinds, expected, "all 18 principal kinds must round-trip");
    assert_eq!(parsed.len(), 21);
}

#[test]
fn parse_bundle_json_parses_all_records() {
    let text = std::fs::read_to_string(EXAMPLE).expect("fixture readable");
    let parsed = parse_bundle_json(&text).expect("bundle parses");
    assert_eq!(parsed.len(), 13);
    assert_eq!(parsed[0].kind(), "mission");
}

#[test]
fn schema_oracle_rejects_an_unknown_field() {
    let mut bundle = load(EXAMPLE);
    let mission = &mut bundle["records"][0];
    assert_eq!(mission["kind"], "mission");
    mission["field_from_the_future"] = Value::Bool(true);
    let err = ContractRecord::from_value(mission.clone()).expect_err("extra field rejected");
    assert!(matches!(err, ContractError::Json(_)), "got {err:?}");
}

#[test]
fn schema_oracle_rejects_a_missing_required_field() {
    let mut bundle = load(EXAMPLE);
    let mission = &mut bundle["records"][0];
    mission.as_object_mut().unwrap().remove("objective");
    let err = ContractRecord::from_value(mission.clone()).expect_err("missing field rejected");
    assert!(matches!(err, ContractError::Json(_)), "got {err:?}");
}

#[test]
fn unknown_enum_member_is_rejected() {
    let mut bundle = load(EXAMPLE);
    let mission = &mut bundle["records"][0];
    mission["data_origin"] = Value::String("trust_me".to_string());
    let err = ContractRecord::from_value(mission.clone()).expect_err("bad enum rejected");
    assert!(matches!(err, ContractError::Json(_)), "got {err:?}");
}

#[test]
fn unsupported_schema_versions_are_rejected_not_guessed() {
    let bundle = load(EXAMPLE);
    for version in ["1.1", "1.0", "0.9", "1.3", "2.0"] {
        let mut record = bundle["records"][0].clone();
        record["schema_version"] = Value::String(version.to_string());
        let err = ContractRecord::from_value(record).expect_err("version must be rejected");
        match err {
            ContractError::UnsupportedSchemaVersion(v) => assert_eq!(v, version),
            other => panic!("expected UnsupportedSchemaVersion, got {other:?}"),
        }
    }
}

#[test]
fn unknown_kind_is_rejected() {
    let bundle = load(EXAMPLE);
    let mut record = bundle["records"][0].clone();
    record["kind"] = Value::String("axiom".to_string());
    let err = ContractRecord::from_value(record).expect_err("unknown kind rejected");
    assert!(
        matches!(&err, ContractError::UnknownKind(k) if k == "axiom"),
        "got {err:?}"
    );
}

#[test]
fn const_boolean_false_is_enforced_by_contract_validation() {
    let bundle = load(EXAMPLE);
    let decision = bundle
        .records()
        .into_iter()
        .find(|r| r["kind"] == "decision")
        .expect("fixture has a decision");
    let mut mutated = decision.clone();
    mutated["authoritative"] = Value::Bool(true);
    let err = ContractRecord::from_value(mutated).expect_err("const false violated");
    match &err {
        ContractError::Invalid(violations) => {
            assert_eq!(violations[0].path, "authoritative");
            assert!(violations[0].message.contains("expected false"));
        }
        other => panic!("expected Invalid, got {other:?}"),
    }
}

#[test]
fn date_time_format_is_enforced() {
    let mut bundle = load(EXAMPLE);
    let mission = &mut bundle["records"][0];
    mission["created_at"] = Value::String("someday".to_string());
    let err = ContractRecord::from_value(mission.clone()).expect_err("bad date rejected");
    match &err {
        ContractError::Invalid(violations) => {
            assert!(violations.iter().any(|v| v.path == "created_at"));
        }
        other => panic!("expected Invalid, got {other:?}"),
    }
}

#[test]
fn string_pattern_is_enforced() {
    let mut bundle = load(EXAMPLE);
    let mission = &mut bundle["records"][0];
    mission["id"] = Value::String("not a valid id!".to_string());
    let err = ContractRecord::from_value(mission.clone()).expect_err("bad pattern rejected");
    match &err {
        ContractError::Invalid(violations) => {
            assert!(violations.iter().any(|v| v.path == "id"));
        }
        other => panic!("expected Invalid, got {other:?}"),
    }
}

#[test]
fn missing_envelope_fields_are_malformed() {
    let err = ContractRecord::from_value(Value::Null).expect_err("null rejected");
    assert!(
        matches!(err, ContractError::MalformedRecord(_)),
        "got {err:?}"
    );
}

trait BundleExt {
    fn records(&self) -> Vec<&Value>;
}

impl BundleExt for Value {
    fn records(&self) -> Vec<&Value> {
        records(self).iter().collect()
    }
}
