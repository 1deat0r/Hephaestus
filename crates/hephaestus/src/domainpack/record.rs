//! Domain-pack records (T-032, M6 exit, R-102 continuity).

use serde::{Deserialize, Serialize};

/// Execution class: physical actuation is SEPARATE from ordinary code
/// execution (IMPLEMENTATION_PLAN:112) — never conflated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionClass {
    CodeExecution,
    PhysicalActuation,
}

/// A declared domain pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainPack {
    pub name: String,
    pub version: String,
    /// Governing assumptions of the domain model.
    pub governing_assumptions: Vec<String>,
    /// Unit system: name + dimension list (e.g. ["seconds", "meters"]).
    pub unit_system_name: String,
    pub unit_dimensions: Vec<String>,
    /// Conditions under which the simulator may stand in for reality.
    pub simulator_validity_conditions: Vec<String>,
    /// Equipment that may be actuated/measured, by whom.
    pub equipment_authorization: Vec<String>,
    /// Which decisions require human review.
    pub human_review_requirements: Vec<String>,
    pub execution_class: ExecutionClass,
    /// Digest of the pack author (for the self-qualification refusal).
    pub author_digest: String,
    /// Oracle id qualified against adjudicated fixtures (R-102).
    pub oracle_id: String,
    pub oracle_qualified: bool,
    /// Scope-limited claims: what this pack does NOT claim.
    pub scope_limits: Vec<String>,
}

/// Qualification errors — each named.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualificationError {
    /// Verifier digest equals author digest: no self-qualification.
    SelfQualification,
    /// Oracle not qualified against adjudicated fixtures.
    UnqualifiedOracle,
    /// Missing scope limits: claims must be scope-limited.
    UnboundedClaims,
}

/// A qualified pack: qualification bound to an independent verifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualifiedPack {
    pub pack: DomainPack,
    pub verifier_digest: String,
}

/// Unit/validity errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitError {
    /// Unit not in the pack's declared dimension list.
    UnknownDimension(String),
    /// Measurement outside the simulator's declared validity conditions.
    OutsideSimulatorValidity,
}
