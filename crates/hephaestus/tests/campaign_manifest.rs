//! frozen-campaign 02 — manifest re-verification (AC 1-3).
//!
//! A manifest that no longer matches live bytes refuses as stale,
//! never as valid. Version binding means a code change re-freezes.

use hephaestus::campaign::{FrozenManifest, ManifestVerdict, freeze_campaign, verify_manifest};

fn fixture() -> (String, String, String) {
    let trace = include_str!("fixtures/e2e-trace.log").to_string();
    let pilot = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/PILOT_CAMPAIGN_REPORT.md"
    ))
    .expect("committed pilot report");
    let baseline = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/BASELINE_COMPARISON_REPORT.md"
    ))
    .expect("committed baseline report");
    (trace, pilot, baseline)
}

#[test]
fn verify_manifest_accepts_matching_bytes() {
    // Positive: freshly frozen inputs verify as valid.
    let (trace, pilot, baseline) = fixture();
    let manifest = freeze_campaign(&trace, &pilot, &baseline).expect("freezes");
    assert_eq!(
        verify_manifest(
            &manifest,
            trace.as_bytes(),
            pilot.as_bytes(),
            baseline.as_bytes()
        ),
        ManifestVerdict::Valid
    );
    // The committed manifest file parses and validates too.
    let committed: FrozenManifest = serde_json::from_str(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/CAMPAIGN_MANIFEST.json"
        ))
        .expect("committed manifest"),
    )
    .expect("manifest parses");
    assert_eq!(
        verify_manifest(
            &committed,
            trace.as_bytes(),
            pilot.as_bytes(),
            baseline.as_bytes()
        ),
        ManifestVerdict::Valid
    );
}

#[test]
fn verify_manifest_refuses_tampered_and_mismatched() {
    // Refusal: one flipped byte names its field; a foreign
    // package version names both versions.
    let (trace, pilot, baseline) = fixture();
    let manifest = freeze_campaign(&trace, &pilot, &baseline).expect("freezes");
    let mut tampered = trace.clone();
    tampered.push('x');
    assert_eq!(
        verify_manifest(
            &manifest,
            tampered.as_bytes(),
            pilot.as_bytes(),
            baseline.as_bytes()
        ),
        ManifestVerdict::DigestMismatch {
            field: "trace_sha256".to_string(),
        }
    );
    let mut moved = manifest.clone();
    moved.package_version = "9.9.9".to_string();
    assert_eq!(
        verify_manifest(
            &moved,
            trace.as_bytes(),
            pilot.as_bytes(),
            baseline.as_bytes()
        ),
        ManifestVerdict::VersionMismatch {
            expected: env!("CARGO_PKG_VERSION").to_string(),
            found: "9.9.9".to_string(),
        }
    );
}

#[test]
fn verify_manifest_names_each_tampered_field() {
    // Every digest path names its field; multi-mismatch reports
    // the first field in fixed order.
    let (trace, pilot, baseline) = fixture();
    let manifest = freeze_campaign(&trace, &pilot, &baseline).expect("freezes");
    let bad_pilot = format!("{pilot}x");
    assert_eq!(
        verify_manifest(
            &manifest,
            trace.as_bytes(),
            bad_pilot.as_bytes(),
            baseline.as_bytes()
        ),
        ManifestVerdict::DigestMismatch {
            field: "pilot_report_digest".to_string(),
        }
    );
    let bad_baseline = format!("{baseline}x");
    assert_eq!(
        verify_manifest(
            &manifest,
            trace.as_bytes(),
            pilot.as_bytes(),
            bad_baseline.as_bytes()
        ),
        ManifestVerdict::DigestMismatch {
            field: "baseline_report_digest".to_string(),
        }
    );
    let mut both = trace.clone();
    both.push('x');
    assert_eq!(
        verify_manifest(
            &manifest,
            both.as_bytes(),
            bad_pilot.as_bytes(),
            baseline.as_bytes()
        ),
        ManifestVerdict::DigestMismatch {
            field: "trace_sha256".to_string(),
        }
    );
    // Top-level edit against an intact frozen section refuses.
    let mut top = manifest.clone();
    top.trace_sha256 = "0".repeat(64);
    assert_eq!(
        verify_manifest(
            &top,
            trace.as_bytes(),
            pilot.as_bytes(),
            baseline.as_bytes()
        ),
        ManifestVerdict::DigestMismatch {
            field: "trace_sha256:top-level".to_string(),
        }
    );
    // Rendered manifests parse back and validate.
    let rendered = serde_json::to_string(&manifest).expect("serializes");
    let parsed: FrozenManifest = serde_json::from_str(&rendered).expect("parses");
    assert_eq!(
        verify_manifest(
            &parsed,
            trace.as_bytes(),
            pilot.as_bytes(),
            baseline.as_bytes()
        ),
        ManifestVerdict::Valid
    );
}

#[test]
fn verify_manifest_replays_deterministically() {
    // Regression: twin checks agree on every verdict path.
    let (trace, pilot, baseline) = fixture();
    let manifest = freeze_campaign(&trace, &pilot, &baseline).expect("freezes");
    let args = || {
        verify_manifest(
            &manifest,
            trace.as_bytes(),
            pilot.as_bytes(),
            baseline.as_bytes(),
        )
    };
    assert_eq!(args(), args());
    assert_eq!(args(), ManifestVerdict::Valid);
}
