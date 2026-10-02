//! Domain packs (T-032, R-006/AT-006 (incomplete pack without validity
//! conditions or evaluators cannot activate — qualify() -> QualifiedPack
//! is the only input), R-004/AT-004 (a physical-action attempt under the
//! software profile is denied — execution_class_separation)). M6 exit.
//!
//! Integration tests at the public seam: `qualify`, `validate_measurement`,
//! `check_execution_class`.

use hephaestus::domainpack::record::{DomainPack, ExecutionClass, QualificationError};
use hephaestus::domainpack::{check_execution_class, qualify, validate_measurement};
fn s(v: &str) -> String {
    v.to_string()
}

fn pack() -> DomainPack {
    DomainPack {
        name: s("numerical-simulation"),
        version: s("1.0.0"),
        governing_assumptions: vec![s("linear elasticity below yield")],
        unit_system_name: s("SI"),
        unit_dimensions: vec![s("seconds"), s("meters"), s("newtons")],
        simulator_validity_conditions: vec![s("small deformations"), s("quasi-static loading")],
        equipment_authorization: vec![],
        human_review_requirements: vec![s("any safety-relevant conclusion")],
        execution_class: ExecutionClass::CodeExecution,
        author_digest: s("author-digest"),
        oracle_id: s("oracle-num-1"),
        oracle_qualified: true,
        scope_limits: vec![s("not valid beyond quasi-static regime")],
    }
}

#[test]
fn qualification_refuses_self_and_unqualified_oracle() {
    // Self-qualification refused (verifier == author).
    assert_eq!(
        qualify(pack(), "author-digest"),
        Err(QualificationError::SelfQualification)
    );
    // Unqualified oracle refused.
    let mut p = pack();
    p.oracle_qualified = false;
    assert_eq!(
        qualify(p, "verifier-digest"),
        Err(QualificationError::UnqualifiedOracle)
    );
    // Unbounded claims refused.
    let mut p = pack();
    p.scope_limits.clear();
    assert_eq!(
        qualify(p, "verifier-digest"),
        Err(QualificationError::UnboundedClaims)
    );
    // Independent verifier + qualified oracle + scoped claims: qualified.
    let qualified = qualify(pack(), "verifier-digest").expect("qualified");
    assert_eq!(qualified.verifier_digest, "verifier-digest");
}

#[test]
fn measurement_units_and_simulator_validity() {
    let qualified = qualify(pack(), "verifier-digest").unwrap();
    // Known dimension, within validity: ok.
    assert_eq!(validate_measurement(&qualified, "seconds", true), Ok(()));
    // Unknown dimension: named error.
    assert_eq!(
        validate_measurement(&qualified, "furlongs", true),
        Err(hephaestus::domainpack::record::UnitError::UnknownDimension(
            s("furlongs")
        ))
    );
    // Outside simulator validity: refused (simulation ≠ deployment).
    assert_eq!(
        validate_measurement(&qualified, "meters", false),
        Err(hephaestus::domainpack::record::UnitError::OutsideSimulatorValidity)
    );
}

#[test]
fn execution_class_separation() {
    // Code-execution pack WITHOUT actuation equipment: coherent.
    assert!(check_execution_class(&pack()).is_ok());
    // Code-execution pack WITH actuation equipment: conflated, refused.
    let mut p = pack();
    p.equipment_authorization = vec![s("robotic-actuator-1 by lab-tech")];
    assert!(check_execution_class(&p).is_err());
    // Actuation pack MUST have equipment authorization.
    let mut p = pack();
    p.execution_class = ExecutionClass::PhysicalActuation;
    assert!(check_execution_class(&p).is_err());
    p.equipment_authorization = vec![s("actuator-2 by licensed operator")];
    assert!(check_execution_class(&p).is_ok());
}

#[test]
fn twin_run_byte_identical() {
    let a = qualify(pack(), "verifier-digest").unwrap();
    let b = qualify(pack(), "verifier-digest").unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
