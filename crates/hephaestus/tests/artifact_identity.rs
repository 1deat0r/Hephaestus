//! Deny-first tests for artifact identity (T-004).

use hephaestus::security::artifact::ArtifactDigest;

#[test]
fn valid_digest_parses() {
    let d = ArtifactDigest::parse("a".repeat(64).as_str()).expect("valid");
    assert_eq!(d.as_str(), "a".repeat(64));
}

#[test]
fn malformed_digests_are_rejected() {
    let uppercase = "A".repeat(64);
    let non_hex = "g".repeat(64);
    let short = "a".repeat(63);
    let long = "a".repeat(65);
    for bad in [
        "",
        "abc",
        uppercase.as_str(),
        non_hex.as_str(),
        short.as_str(),
        long.as_str(),
    ] {
        assert!(
            ArtifactDigest::parse(bad).is_err(),
            "should reject: {bad:?}"
        );
    }
}

#[test]
fn digest_of_bytes_matches_known_sha256() {
    // sha256("abc") — published NIST test vector.
    let d = ArtifactDigest::of_bytes(b"abc");
    assert_eq!(
        d.as_str(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn different_bytes_different_identity() {
    assert_ne!(
        ArtifactDigest::of_bytes(b"candidate-v1"),
        ArtifactDigest::of_bytes(b"candidate-v2")
    );
}
