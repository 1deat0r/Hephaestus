//! R-104 guardrail presence gate (T-041).
//!
//! Integration tests at the public seam: `require_guardrails`,
//! `check_guardrails` (semantics unchanged).

use hephaestus::evaluation::{GuardrailClass, check_guardrails, require_guardrails};

#[test]
fn complete_declaration_passes() {
    let all = vec![
        GuardrailClass::ScopedError,
        GuardrailClass::Correctness,
        GuardrailClass::PrecisionPower,
        GuardrailClass::Noninferiority,
    ];
    assert!(
        require_guardrails(&all).is_empty(),
        "all four declared: no gaps"
    );
}

#[test]
fn missing_classes_named_per_class() {
    // Only two declared: the other two named as gaps.
    let partial = vec![GuardrailClass::ScopedError, GuardrailClass::Correctness];
    let gaps = require_guardrails(&partial);
    assert_eq!(gaps.len(), 2);
    let missing: Vec<_> = gaps.iter().map(|g| g.missing).collect();
    assert!(missing.contains(&GuardrailClass::PrecisionPower));
    assert!(missing.contains(&GuardrailClass::Noninferiority));
    // Nothing declared: all four gaps.
    assert_eq!(require_guardrails(&[]).len(), 4);
}

#[test]
fn evaluation_semantics_unchanged() {
    // The presence gate does not alter check_guardrails: a failing
    // guardrail still records ok=false independently.
    let report = check_guardrails(&[(s("peak_memory_ratio"), 1.25, 1.10)]);
    assert!(!report.all_pass);
    let ok = check_guardrails(&[(s("peak_memory_ratio"), 1.05, 1.10)]);
    assert!(ok.all_pass);
}

fn s(v: &str) -> String {
    v.to_string()
}

#[test]
fn twin_run_byte_identical() {
    let partial = vec![GuardrailClass::ScopedError];
    let a = require_guardrails(&partial);
    let b = require_guardrails(&partial);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
