//! Deny-first tests for manifest approvals (T-004): a bare content hash,
//! a wrong key, or a tampered payload must never verify.

use hephaestus::security::approval::{
    ApprovalError, ApprovalKey, approve, approve_manifest, verify, verify_manifest,
};
use hephaestus::security::digest::record_digest;
use serde_json::json;

fn key(seed: u8) -> ApprovalKey {
    ApprovalKey::from_bytes(&[seed; 32]).expect("32-byte key")
}

#[test]
fn short_or_empty_key_material_is_rejected() {
    assert!(matches!(
        ApprovalKey::from_bytes(&[]),
        Err(ApprovalError::WeakKey(0))
    ));
    assert!(matches!(
        ApprovalKey::from_bytes(&[7u8; 31]),
        Err(ApprovalError::WeakKey(31))
    ));
}

#[test]
fn approve_then_verify_round_trips() {
    let k = key(1);
    let payload = b"manifest bytes";
    let tag = approve(&k, payload).expect("approve");
    assert!(verify(&k, payload, &tag).is_ok());
}

#[test]
fn approval_is_deterministic() {
    let k = key(2);
    assert_eq!(
        approve(&k, b"same input").unwrap(),
        approve(&k, b"same input").unwrap()
    );
    assert_ne!(
        approve(&k, b"same input").unwrap(),
        approve(&k, b"other input").unwrap()
    );
}

#[test]
fn bare_content_hash_never_verifies_as_approval() {
    let k = key(3);
    let payload = b"important manifest";
    // The unkeyed SHA-256 of the payload — exactly what T-004 forbids as an
    // authorization signature.
    let bare_hash = record_digest(&json!({"payload": "important manifest"})).unwrap();
    assert!(
        verify(&k, payload, &bare_hash).is_err(),
        "a plain content hash must not authenticate"
    );
    // And the actual sha256 of the raw bytes also fails.
    let raw_hash: String = {
        use sha2::{Digest, Sha256};
        Sha256::digest(payload)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    };
    assert!(verify(&k, payload, &raw_hash).is_err());
}

#[test]
fn wrong_key_denies() {
    let tag = approve(&key(4), b"payload").unwrap();
    assert_eq!(
        verify(&key(5), b"payload", &tag),
        Err(ApprovalError::VerificationFailed)
    );
}

#[test]
fn tampered_payload_denies() {
    let k = key(6);
    let tag = approve(&k, b"original").unwrap();
    assert_eq!(
        verify(&k, b"tampered", &tag),
        Err(ApprovalError::VerificationFailed)
    );
}

#[test]
fn malformed_tags_fail_closed() {
    let k = key(7);
    for tag in [
        "",
        "zz",
        &"a".repeat(63),
        &"A".repeat(64),
        "not-hex-at-all-padding-pad",
    ] {
        assert!(
            matches!(
                verify(&k, b"payload", tag),
                Err(ApprovalError::MalformedTag(_))
            ),
            "tag should fail closed: {tag:?}"
        );
    }
}

#[test]
fn manifest_approvals_use_canonical_json() {
    let k = key(8);
    // Same content, different key order — must approve identically.
    let a = json!({"alpha": 1, "beta": [true, null]});
    let b = json!({"beta": [true, null], "alpha": 1});
    let tag_a = approve_manifest(&k, &a).unwrap();
    let tag_b = approve_manifest(&k, &b).unwrap();
    assert_eq!(tag_a, tag_b, "canonical encoding must ignore key order");
    assert!(verify_manifest(&k, &b, &tag_a).is_ok());
    let mut tampered = a.clone();
    tampered["alpha"] = json!(2);
    assert_eq!(
        verify_manifest(&k, &tampered, &tag_a),
        Err(ApprovalError::VerificationFailed)
    );
}
