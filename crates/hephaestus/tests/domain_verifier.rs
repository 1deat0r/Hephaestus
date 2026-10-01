//! R-061/R-062/R-063 domain contract + adapter verifier gates (T-046).
//!
//! Integration tests at the public seam: `TypedRejection` codes,
//! `verify_cross_record`, `verify_no_provider_leak`,
//! `verify_supported_capability`, `verify_version_and_refs`.

use hephaestus::domainver::{
    RejectionCode, verify_cross_record, verify_no_provider_leak, verify_supported_capability,
    verify_version_and_refs,
};
use serde_json::json;

#[test]
fn at061_contradictory_records_rejected() {
    // Schema-valid JSON, semantically contradictory: two records with
    // the same (id, version) — a duplicate the validator of record
    // rejects.
    let bundle = json!({
        "records": [
            {"id": "rec-1", "record_version": 1, "kind": "observation",
             "mission_ref": {"id": "m-1", "version": 1}},
            {"id": "rec-1", "record_version": 1, "kind": "observation",
             "mission_ref": {"id": "m-1", "version": 1}},
            {"id": "m-1", "record_version": 1, "kind": "mission"}
        ]
    });
    let err = verify_cross_record(&bundle).unwrap_err();
    assert!(
        err.iter()
            .any(|r| r.code == RejectionCode::SemanticContradiction
                && r.detail.contains("DUPLICATE_VERSION"))
    );
    // A clean bundle passes.
    let clean = json!({
        "records": [
            {"id": "rec-1", "record_version": 1, "kind": "observation",
             "mission_ref": {"id": "m-1", "version": 1}},
            {"id": "m-1", "record_version": 1, "kind": "mission"}
        ]
    });
    assert!(verify_cross_record(&clean).is_ok());
    // A schema-valid bundle with an unresolvable reference -> the
    // MISSING_REFERENCE stable code.
    let dangling = json!({
        "records": [
            {"id": "rec-9", "record_version": 1, "kind": "observation",
             "mission_ref": {"id": "nope", "version": 7}}
        ]
    });
    let err = verify_cross_record(&dangling).unwrap_err();
    assert!(
        err.iter()
            .any(|r| r.code == RejectionCode::MissingReference)
    );
}

#[test]
fn at062_no_provider_leak_and_explicit_unsupported() {
    // Core symbols free of provider markers pass.
    assert!(
        verify_no_provider_leak(&[
            "ExecutionBackend".to_string(),
            "DispatchReceipt".to_string()
        ])
        .is_ok()
    );
    // A provider SDK type in a core symbol leaks.
    let err = verify_no_provider_leak(&["OpenAiClientAdapter".to_string()]).unwrap_err();
    assert!(
        err.iter()
            .any(|r| r.code == RejectionCode::ProviderLeak && r.detail.contains("openai"))
    );
    // Adapter swap: a declared capability works, an undeclared one
    // fails explicitly (never silently).
    let declared = vec!["local.exec".to_string(), "local.replay".to_string()];
    assert!(verify_supported_capability(&declared, "local.exec").is_ok());
    let err = verify_supported_capability(&declared, "remote.gpu").unwrap_err();
    assert_eq!(err.code, RejectionCode::UnsupportedCapability);
    assert!(err.detail.contains("remote.gpu"));
}

#[test]
fn at063_unknown_version_and_missing_reference_stable_codes() {
    // Unknown schema version -> stable code.
    let err = verify_version_and_refs("9.9", &["a:1".to_string()], &[]).unwrap_err();
    assert!(
        err.iter()
            .any(|r| r.code == RejectionCode::UnknownSchemaVersion && r.detail.contains("9.9"))
    );
    // Missing reference -> stable code.
    let err =
        verify_version_and_refs("1.3", &["a:1".to_string()], &["b:2".to_string()]).unwrap_err();
    assert!(
        err.iter()
            .any(|r| r.code == RejectionCode::MissingReference && r.detail.contains("b:2"))
    );
    // Both known -> Ok.
    assert!(verify_version_and_refs("1.3", &["a:1".to_string()], &["a:1".to_string()]).is_ok());
}

#[test]
fn twin_run_byte_identical_rejections() {
    let bundle = json!({
        "records": [
            {"id": "r", "record_version": 1, "kind": "observation",
             "mission_ref": {"id": "gone", "version": 1}}
        ]
    });
    let ser = |b: &serde_json::Value| {
        let rs = verify_cross_record(b).unwrap_err();
        serde_json::to_string(&rs).unwrap()
    };
    assert_eq!(ser(&bundle), ser(&bundle));
}
