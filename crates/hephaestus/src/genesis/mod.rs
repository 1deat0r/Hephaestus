//! Genesis mechanism-operator registry (T-015, MASTER_SPEC section 8).
//!
//! Scope: versioned operators transform discovery Opportunities into
//! structured `MechanismRecord`s with provenance, or structured
//! `RejectedApplicability` reasons — never silent drops. Hard filters are
//! restricted to justified constraints (R-024); plausibility flags are
//! advisory. Hypothesis compilation is T-016; prior-art and novelty
//! language are T-018 — nothing here claims novelty (R-009).
//!
//! Purity: no I/O, no ledger coupling. Callers persist what they accept.

pub mod record;
pub mod registry;

pub use record::{MechanismError, MechanismRecord, PlausibilityFlag, Provenance};
pub use registry::{OperatorOutcome, RejectedApplicability};
