//! T-002 migration registry tests: registered versions migrate and
//! revalidate; unregistered or unknown versions fail closed with reasons.

use hephaestus::contracts::{ContractError, SCHEMA_VERSION};
use hephaestus::migrate::{MigrationError, migrate_bundle, migrate_to_current};
use serde_json::Value;

const V1_0: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/software-mission.v1.0.json"
);
const V1_1: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/software-mission.v1.1.json"
);

fn load(path: &str) -> Value {
    let text = std::fs::read_to_string(path).expect("fixture readable");
    serde_json::from_str(&text).expect("fixture is valid JSON")
}

/// See `contracts.rs`: compare JSON numbers semantically, not lexically.
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

#[test]
fn v1_1_bundle_migrates_to_current_and_only_retags() {
    let bundle = load(V1_1);
    let migrated = migrate_bundle(&bundle).expect("1.1 -> 1.2 is registered");
    let originals = bundle["records"].as_array().unwrap();
    assert_eq!(migrated.len(), originals.len());
    for (record, original) in migrated.iter().zip(originals) {
        let value = record.to_value().unwrap();
        assert_eq!(value["schema_version"], SCHEMA_VERSION);
        // 1.1 and 1.2 share deep-equal record definitions, so a successful
        // migration may change exactly one field: schema_version.
        let mut expected = original.clone();
        expected["schema_version"] = Value::String(SCHEMA_VERSION.to_string());
        assert_eq!(
            normalize_numbers(&value),
            normalize_numbers(&expected),
            "migration of {} changed more than the tag",
            record.id()
        );
        assert_eq!(
            record.validate(),
            Vec::<hephaestus::contracts::ContractViolation>::new()
        );
    }
}

#[test]
fn v1_0_is_rejected_not_guessed() {
    let bundle = load(V1_0);
    let err = migrate_bundle(&bundle).expect_err("1.0 must fail closed");
    match err {
        MigrationError::NotRegistered { from, kind, reason } => {
            assert_eq!(from, "1.0");
            let expected_kind = bundle["records"][0]["kind"].as_str().unwrap();
            assert_eq!(kind, expected_kind, "first failing record kind is reported");
            assert!(
                reason.contains("fabricat"),
                "reason must explain the fabrication risk"
            );
            assert!(
                reason.contains("ADR-013"),
                "reason must cite the receipts rule"
            );
        }
        other => panic!("expected NotRegistered, got {other}"),
    }
}

#[test]
fn unknown_versions_are_unsupported() {
    for version in ["0.9", "1.3", "1.9", "2.0", "9.9"] {
        let mut record = load(V1_1)["records"][0].clone();
        record["schema_version"] = Value::String(version.to_string());
        match migrate_to_current(record) {
            Err(MigrationError::UnsupportedVersion(v)) => assert_eq!(v, version),
            other => panic!("expected UnsupportedVersion({version}), got {other:?}"),
        }
    }
}

#[test]
fn missing_or_non_string_version_is_malformed() {
    let mut record = load(V1_1)["records"][0].clone();
    record.as_object_mut().unwrap().remove("schema_version");
    assert!(matches!(
        migrate_to_current(record.clone()),
        Err(MigrationError::Malformed(_))
    ));
    record["schema_version"] = Value::from(11);
    assert!(matches!(
        migrate_to_current(record),
        Err(MigrationError::Malformed(_))
    ));
}

#[test]
fn migrated_record_with_unknown_kind_fails_contract_validation() {
    let mut record = load(V1_1)["records"][0].clone();
    record["kind"] = Value::String("axiom".to_string());
    match migrate_to_current(record) {
        Err(MigrationError::Contract(ContractError::UnknownKind(k))) => assert_eq!(k, "axiom"),
        other => panic!("expected Contract(UnknownKind), got {other:?}"),
    }
}

#[test]
fn bundle_without_records_is_malformed() {
    assert!(matches!(
        migrate_bundle(&Value::Object(serde_json::Map::new())),
        Err(MigrationError::Malformed(_))
    ));
}
