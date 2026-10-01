//! Pressure-point operators (T-014, MASTER_SPEC section 7): turn authorized
//! trace records plus corpus evidence into structured `Opportunity` records
//! (R-019) — never supplied ideas. Each operator has a named contract:
//! what it accepts, what it rejects, and why. Rejected candidates are
//! retained with reasons (MASTER_SPEC:152); nothing is silently dropped.
//!
//! Scope: detection only. Mechanism search is T-015; hypothesis
//! compilation is T-016; prior-art investigation and any novelty language
//! are T-018 — a zero-hit search there cannot yield global-novelty
//! language, and nothing here claims novelty (R-009).
//!
//! Purity: no I/O, no ledger coupling (mission-compiler precedent).
//! Callers persist what they accept.

pub mod operators;
pub mod record;

pub use crate::knowledge::Span;
pub use operators::analyze;
pub use record::{
    AnalysisOutcome, Opportunity, PressureKind, RejectedCandidate, TraceRecord, TraceVal, Validity,
    ValueEstimate, assess_validity, narrative_polish,
};
