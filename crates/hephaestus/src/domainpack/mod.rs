//! Independently qualified domain packs (T-032, M6 exit).
//!
//! New capabilities are qualified SEPARATELY — they never inherit proof
//! from the software domain. A pack declares governing assumptions,
//! units, simulator validity, equipment authorization, and human-review
//! requirements; physical actuation is typed separately from ordinary
//! code execution.

pub mod record;

pub use record::{DomainPack, ExecutionClass, QualificationError, QualifiedPack, UnitError};

/// Qualify a pack (M6 exit: independent qualification):
/// - verifier identity must differ from the author (no
///   self-qualification);
/// - the pack's oracle must be qualified against adjudicated fixtures
///   (R-102);
/// - claims must be scope-limited.
pub fn qualify(
    pack: DomainPack,
    verifier_digest: &str,
) -> Result<QualifiedPack, QualificationError> {
    if verifier_digest == pack.author_digest {
        return Err(QualificationError::SelfQualification);
    }
    if !pack.oracle_qualified {
        return Err(QualificationError::UnqualifiedOracle);
    }
    if pack.scope_limits.is_empty() {
        return Err(QualificationError::UnboundedClaims);
    }
    Ok(QualifiedPack {
        pack,
        verifier_digest: verifier_digest.to_string(),
    })
}

/// Validate a measurement against the pack's unit system and simulator
/// validity: unknown dimensions refused; measurements outside declared
/// validity conditions refused (simulation ≠ deployment).
pub fn validate_measurement(
    pack: &QualifiedPack,
    unit: &str,
    within_simulator_validity: bool,
) -> Result<(), UnitError> {
    if !pack.pack.unit_dimensions.iter().any(|d| d == unit) {
        return Err(UnitError::UnknownDimension(unit.to_string()));
    }
    if !within_simulator_validity {
        return Err(UnitError::OutsideSimulatorValidity);
    }
    Ok(())
}

/// Check that a pack's execution class is coherent with its equipment
/// authorization: a code-execution pack may not list actuation
/// equipment, and an actuation pack must list equipment authorization.
pub fn check_execution_class(pack: &DomainPack) -> Result<(), String> {
    let mentions_actuation = pack
        .equipment_authorization
        .iter()
        .any(|e| e.contains("actuator") || e.contains("actuation"));
    match pack.execution_class {
        ExecutionClass::CodeExecution if mentions_actuation => {
            Err("code-execution pack lists actuation equipment: classes conflated".to_string())
        }
        ExecutionClass::PhysicalActuation if pack.equipment_authorization.is_empty() => {
            Err("actuation pack without equipment authorization".to_string())
        }
        _ => Ok(()),
    }
}
