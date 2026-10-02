//! R-108/AT-108 trust-origin propagation (T-043): no transformation
//! elevates untrusted content into policy, approval or evidence.
//!
//! Integration tests at the public seam: `derive_origin`,
//! `DerivedTrust::derive`.

use hephaestus::contracts::generated::TrustOrigin;
use hephaestus::trustprop::{DerivedTrust, derive_origin, trust_rank};

use TrustOrigin::{
    ObservedTool, Owner, ProtectedService, SyntheticFixture, UntrustedModel, UntrustedSource,
};

#[test]
fn min_of_inputs_no_laundering() {
    // One untrusted input contaminates: a summary of owner + untrusted
    // source is UNTRUSTED, never laundered to owner.
    assert_eq!(derive_origin(&[Owner, UntrustedSource]), UntrustedSource);
    // All trusted inputs derive trusted.
    assert_eq!(derive_origin(&[Owner, ProtectedService]), Owner);
    // Protected service is the strongest single origin.
    assert_eq!(derive_origin(&[ProtectedService]), ProtectedService);
    // Empty -> UntrustedSource (nothing known is not trusted).
    assert_eq!(derive_origin(&[]), UntrustedSource);
}

#[test]
fn ranking_synthetic_below_observed() {
    assert!(trust_rank(&SyntheticFixture) < trust_rank(&ObservedTool));
    assert!(trust_rank(&UntrustedSource) < trust_rank(&UntrustedModel));
    assert!(trust_rank(&UntrustedModel) < trust_rank(&SyntheticFixture));
    assert!(trust_rank(&ObservedTool) < trust_rank(&Owner));
    assert!(trust_rank(&Owner) < trust_rank(&ProtectedService));
    // Synthetic input weakens an owner derivation to synthetic.
    assert_eq!(derive_origin(&[Owner, SyntheticFixture]), SyntheticFixture);
}

#[test]
fn derived_stamp_serializable() {
    let stamp = DerivedTrust::derive(
        &["knowledge:source-9", "memory:artifact-2"],
        &[ObservedTool, UntrustedSource],
        "summary",
    );
    assert_eq!(stamp.inherited_origin, UntrustedSource);
    assert_eq!(stamp.derivation, "summary");
    assert_eq!(stamp.source_refs.len(), 2);
    // Round-trips through JSON (ledger/cache/cross-session survival).
    let json = serde_json::to_string(&stamp).unwrap();
    let back: DerivedTrust = serde_json::from_str(&json).unwrap();
    assert_eq!(back, stamp);
}

#[test]
fn twin_run_byte_identical() {
    let a = DerivedTrust::derive(&["k:1"], &[Owner], "cache");
    let b = DerivedTrust::derive(&["k:1"], &[Owner], "cache");
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
