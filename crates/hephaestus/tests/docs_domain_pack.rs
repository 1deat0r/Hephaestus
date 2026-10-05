//! Docs domain pack qualification (dev-roadmap ticket 03; Covers AC 3).
//!
//! Builds a DomainPack fixture for the repository documentation
//! domain (code-execution only, no actuation), qualifies it through
//! the real `qualify` seam with an independent verifier and an
//! adjudicated oracle, and pins twin-run byte identity.

use hephaestus::domainpack::record::{DomainPack, ExecutionClass, QualificationError};
use hephaestus::domainpack::{check_execution_class, qualify};

fn s(v: &str) -> String {
    v.to_string()
}

/// The docs pack fixture: documentation retrieval and assessment
/// over repository text. Code execution only — no equipment, and
/// the scope limits name what the pack does not claim.
fn docs_pack() -> DomainPack {
    DomainPack {
        name: s("repository-docs"),
        version: s("1.0.0"),
        governing_assumptions: vec![s("docs are versioned text under review")],
        unit_system_name: s("count"),
        unit_dimensions: vec![s("documents"), s("tokens")],
        simulator_validity_conditions: vec![s("no simulator: direct text retrieval")],
        equipment_authorization: vec![],
        human_review_requirements: vec![s("any safety-relevant conclusion")],
        execution_class: ExecutionClass::CodeExecution,
        author_digest: s("docs-author-digest"),
        oracle_id: s("oracle-docs-adjudicated-1"),
        oracle_qualified: true,
        scope_limits: vec![s("retrieval only: never a novelty verdict")],
    }
}

#[test]
fn docs_pack_qualifies_with_independent_verifier() {
    let qualified = qualify(docs_pack(), "docs-verifier-digest").expect("docs pack qualifies");
    assert_eq!(qualified.verifier_digest, "docs-verifier-digest");
    assert_eq!(qualified.pack.oracle_id, "oracle-docs-adjudicated-1");
    check_execution_class(&docs_pack()).expect("code execution carries no actuation");
}

#[test]
fn docs_pack_refuses_self_unqualified_oracle_and_unbounded_claims() {
    assert_eq!(
        qualify(docs_pack(), "docs-author-digest"),
        Err(QualificationError::SelfQualification)
    );
    let mut p = docs_pack();
    p.oracle_qualified = false;
    assert_eq!(
        qualify(p, "docs-verifier-digest"),
        Err(QualificationError::UnqualifiedOracle)
    );
    let mut p = docs_pack();
    p.scope_limits.clear();
    assert_eq!(
        qualify(p, "docs-verifier-digest"),
        Err(QualificationError::UnboundedClaims)
    );
}

#[test]
fn twin_qualifications_are_byte_identical() {
    let a = qualify(docs_pack(), "docs-verifier-digest").unwrap();
    let b = qualify(docs_pack(), "docs-verifier-digest").unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap(),
        "deterministic qualification"
    );
}
