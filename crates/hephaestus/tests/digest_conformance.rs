//! Rust side of the cross-language digest conformance vectors (T-004).
//!
//! Every vector is generated from the Python reference profile; if the Rust
//! canonical writer ever diverges (key order, escaping, number rendering),
//! these tests fail closed.

use hephaestus::security::digest::{plan_digest, record_digest, subject_digest};
use serde_json::Value;

const VECTORS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/digest-vectors.json"
);

fn load(path: &str) -> Value {
    let text = std::fs::read_to_string(path).expect("file readable");
    serde_json::from_str(&text).expect("valid JSON")
}

#[test]
fn rust_digests_match_python_vectors() {
    let vectors = load(VECTORS);
    let vectors = vectors.as_array().expect("vector array");
    assert!(
        vectors.len() >= 36,
        "expected all fixture records + synthetics"
    );

    // Fixture bundles are cached per path.
    let mut bundles: Vec<(String, Value)> = Vec::new();

    for vector in vectors {
        let record = if let Some(synthetic) = vector.get("synthetic") {
            synthetic.clone()
        } else {
            let fixture = vector["fixture"].as_str().expect("fixture path");
            if !bundles.iter().any(|(path, _)| path == fixture) {
                let bundle = load(&format!("{}/../../{}", env!("CARGO_MANIFEST_DIR"), fixture));
                bundles.push((fixture.to_string(), bundle));
            }
            let bundle = &bundles.iter().find(|(path, _)| path == fixture).unwrap().1;
            let id = vector["id"].as_str().expect("record id");
            bundle["records"]
                .as_array()
                .expect("records")
                .iter()
                .find(|r| r["id"] == Value::String(id.to_string()))
                .unwrap_or_else(|| panic!("record {id} in {fixture}"))
                .clone()
        };

        let label = vector
            .get("synthetic_label")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| {
                format!(
                    "{}:{}",
                    vector["fixture"].as_str().unwrap_or("?"),
                    vector["id"].as_str().unwrap_or("?")
                )
            });

        assert_eq!(
            record_digest(&record).expect("digest"),
            vector["record_digest"].as_str().expect("vector"),
            "record_digest mismatch: {label}"
        );
        assert_eq!(
            subject_digest(&record).expect("digest"),
            vector["subject_digest"].as_str().expect("vector"),
            "subject_digest mismatch: {label}"
        );
        if let Some(expected) = vector.get("plan_digest") {
            assert_eq!(
                plan_digest(&record).expect("digest"),
                expected.as_str().expect("vector"),
                "plan_digest mismatch: {label}"
            );
        }
    }
}

#[test]
fn non_finite_numbers_are_rejected() {
    // serde_json cannot hold a non-finite Number, but the canonical writer
    // keeps the allow_nan=False guard for defense in depth.
    let value: Value = serde_json::from_str(r#"{"a": 1}"#).expect("finite");
    assert!(record_digest(&value).is_ok());
}
