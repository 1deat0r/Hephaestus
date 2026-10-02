//! E2E mission driver — M3 chain (T-061 ticket 02; IMPLEMENTATION_PLAN:82).
//!
//! Assertions over the shared chain in `hephaestus::missionrun`:
//! trace fixture -> qualified interval -> experiment plan -> typed result
//! -> dossier export (supported path + honest negative). Data only: no
//! evaluator linkage (evaluator_access guard); hidden verdicts stay the
//! evaluator crate's own suite, cited by the exit assessment.

use hephaestus::dossier::record::NegativeResultKind;
use hephaestus::evaluation::{ExecutionValidity, ScientificConclusion};
use hephaestus::missionrun::run;

fn trace() -> &'static str {
    include_str!("fixtures/e2e-trace.log")
}

#[test]
fn e2e_m3_supported_path_exports_a_measured_dossier() {
    // theta below the interval's lower bound -> supported claim.
    let receipt = run(trace(), 0.25, None).expect("chain completes");
    assert_eq!(
        receipt.typed.execution,
        ExecutionValidity::Valid,
        "digest + qualified method: {:?}",
        receipt.typed
    );
    assert_eq!(
        receipt.typed.science,
        ScientificConclusion::Supported,
        "interval {:?} above theta {}: {:?}",
        receipt.interval,
        receipt.theta,
        receipt.typed
    );
    let out: serde_json::Value = serde_json::from_str(&receipt.exported).expect("export json");
    assert_eq!(
        out["evidence_label"]["Measured"]["run_receipt"],
        receipt.receipt_sha256.as_str()
    );
    assert!(!out["raw_data"].as_array().expect("raw data").is_empty());
    assert!(
        receipt.exported.contains("Supported"),
        "result carried through"
    );
}

#[test]
fn e2e_m3_contradicted_result_exports_as_an_honest_negative() {
    // theta far above the interval -> the claim is contradicted, and the
    // dossier carries a named negative kind (M3 exit: a negative result
    // is exported honestly, never smoothed away).
    let receipt = run(
        trace(),
        10.0,
        Some(NegativeResultKind::UnsupportedMechanism),
    )
    .expect("chain completes");
    assert_eq!(
        receipt.typed.execution,
        ExecutionValidity::Valid,
        "{:?}",
        receipt.typed
    );
    assert_eq!(
        receipt.typed.science,
        ScientificConclusion::Contradicted,
        "interval {:?} below theta {}: {:?}",
        receipt.interval,
        receipt.theta,
        receipt.typed
    );
    let out: serde_json::Value = serde_json::from_str(&receipt.exported).expect("export json");
    assert_eq!(
        out["negative_kind"], "UnsupportedMechanism",
        "negative kind attached: {out}"
    );
    assert!(
        receipt.exported.contains("Contradicted"),
        "verdict carried through"
    );
    assert_eq!(
        out["evidence_label"]["Measured"]["run_receipt"],
        receipt.receipt_sha256.as_str()
    );
}
