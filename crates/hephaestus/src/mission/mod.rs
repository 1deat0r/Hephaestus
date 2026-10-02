//! Mission intake and Intent-to-Mission compiler (T-012, M2 opener).
//!
//! Implements MASTER_SPEC section 4: a broad authorized goal plus the
//! owner's value frame compiles to a versioned `Mission` — or, for
//! authorization-grade goals, to explicit authorization requests instead of
//! a guessed Mission. No seed hypothesis is required (R-001); inferred
//! requirements carry provenance and an override path (R-010); materially
//! unresolved authorization or intent is asked, never assumed (R-011);
//! goal/cost/dataset/quality changes mint new versions (R-012).
//!
//! Scope note: the compiler is pure (no I/O, no ledger coupling). Callers
//! persist the Mission they accept; the scheduler owns dispatch, pause,
//! cancellation, and plan invalidation.

pub mod compiler;
pub mod record;

pub use compiler::{compile, revise};
pub use record::{
    AgentAgreement, Assumption, AssumptionKind, AuthReason, AuthorizationRequest, AutonomyProfile,
    CompileError, Compiled, CompletionError, EvidenceRef, ImpactReport, Intake, Mission,
    MissionChange, MissionCompletion, ResourceEnvelope, ResourceProfile, ReviseError,
    default_guardrails, forbidden_actions,
};

/// Mission completion gate (R-091/AT-091, MASTER_SPEC §31): a mission
/// completes with USABLE version-bound evidence — unanimous agent
/// agreement is recorded but structurally never consulted, so approval
/// without evidence cannot produce a completion record (hence no
/// validated-invention-candidate claim can rest on it).
pub fn complete_mission(
    mission_id: &str,
    _agreement: &record::AgentAgreement,
    evidence: &[record::EvidenceRef],
) -> Result<record::MissionCompletion, record::CompletionError> {
    if evidence.is_empty() {
        return Err(record::CompletionError::AgreementWithoutEvidence);
    }
    for r in evidence {
        if r.evidence_id.is_empty() || r.version.is_empty() {
            return Err(record::CompletionError::UnusableEvidence {
                evidence_id: r.evidence_id.clone(),
            });
        }
    }
    Ok(record::MissionCompletion {
        mission_id: mission_id.to_string(),
        evidence: evidence.to_vec(),
    })
}
