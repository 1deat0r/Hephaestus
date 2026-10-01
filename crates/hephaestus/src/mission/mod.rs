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
    Assumption, AssumptionKind, AuthReason, AuthorizationRequest, AutonomyProfile, CompileError,
    Compiled, ImpactReport, Intake, Mission, MissionChange, ResourceEnvelope, ResourceProfile,
    ReviseError, default_guardrails, forbidden_actions,
};
