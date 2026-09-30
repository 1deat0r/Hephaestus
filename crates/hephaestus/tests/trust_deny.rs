//! Deny-first tests for the external trust context (T-004).

use hephaestus::security::digest::record_digest;
use hephaestus::security::trust::TrustContext;
use serde_json::json;

fn grant_record() -> serde_json::Value {
    json!({
        "id": "GRANT-1",
        "kind": "authorization_grant",
        "record_version": 1,
        "state": "approved"
    })
}

#[test]
fn empty_context_denies_everything() {
    let ctx = TrustContext::empty();
    let record = grant_record();
    assert!(!ctx.trusted_grant("GRANT-1", 1, &record));
    assert!(!ctx.trusted_receipt("GRANT-1", 1, &record));
    assert!(!ctx.trusted_family("GRANT-1", 1, &record));
    assert!(!ctx.has_verified_artifact(&"a".repeat(64)));
    assert!(
        ctx.evaluated_at().is_none(),
        "no external clock, no time claims"
    );
    assert_eq!(ctx.current_policy("MIS-1"), None);
}

#[test]
fn allowlist_membership_alone_is_not_trust() {
    let mut ctx = TrustContext::empty();
    ctx.trust_grant("GRANT-1", 1);
    let record = grant_record();
    // Membership without an externally authenticated hash: deny.
    assert!(!ctx.trusted_grant("GRANT-1", 1, &record));
}

#[test]
fn authenticated_hash_must_match_current_bytes() {
    let record = grant_record();
    let digest = record_digest(&record).unwrap();
    let mut ctx = TrustContext::empty();
    ctx.trust_grant("GRANT-1", 1);
    ctx.authenticate_record_hash("GRANT-1", 1, digest.clone());
    assert!(ctx.trusted_grant("GRANT-1", 1, &record));

    // Content changed after authentication (retraction/tamper): deny.
    let mut changed = record.clone();
    changed["state"] = json!("revoked_but_record_also_edited");
    assert!(!ctx.trusted_grant("GRANT-1", 1, &changed));
}

#[test]
fn wrong_version_is_not_trust() {
    let record = grant_record();
    let digest = record_digest(&record).unwrap();
    let mut ctx = TrustContext::empty();
    ctx.trust_grant("GRANT-1", 1);
    ctx.authenticate_record_hash("GRANT-1", 1, digest);
    assert!(!ctx.trusted_grant("GRANT-1", 2, &record));
}

#[test]
fn record_hash_comparison_fails_closed_when_unknown() {
    let ctx = TrustContext::empty();
    assert_eq!(ctx.record_hash_matches("MIS-1", 1, &grant_record()), None);
}

#[test]
fn trust_context_has_no_deserialization_surface() {
    // Compile-time property by construction: TrustContext implements no
    // Deserialize/From<Value>, so a candidate bundle has no path to supply
    // one. This test documents the guarantee and exercises the only
    // constructor.
    let ctx = TrustContext::empty();
    let _ = ctx.clone();
}
