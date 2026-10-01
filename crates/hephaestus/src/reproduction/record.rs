//! Reproduction records (T-034, M6: independent reproduction).

use serde::{Deserialize, Serialize};

/// A candidate warranting reproduction: declared novelty + utility
/// claims (roadmap: "the first candidate whose novelty and utility
/// claims warrant it").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candidate {
    pub candidate_id: String,
    pub novelty_claim: String,
    pub utility_claim: String,
    /// Digest of the original claim (binds the reproduction to it).
    pub claim_digest: String,
    /// Registered analysis endpoint (R-037: immutable, registered
    /// before confirmation).
    pub analysis_endpoint: String,
    /// Digest of the original claimant (identity separation check).
    pub claimant_digest: String,
}

/// Warrant errors — each named.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarrantError {
    /// Missing novelty claim.
    NoNoveltyClaim,
    /// Missing utility claim.
    NoUtilityClaim,
    /// A reproduction already exists for this candidate (first only).
    AlreadyReproduced,
}

/// The reproduction outcome — Inconclusive and Refuted are FIRST-CLASS
/// valid results, never coerced to confirmation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReproductionOutcome {
    Confirmed,
    Refuted,
    Inconclusive,
    /// No independent reproducer available yet: honest pending state.
    PendingExternal,
}

/// Outcome recording errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeError {
    /// Reproducer digest equals claimant digest: not independent.
    NotIndependent,
    /// The endpoint was not registered before confirmation (R-037).
    UnregisteredEndpoint,
}

/// A recorded reproduction bound to its candidate and outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReproductionRecord {
    pub candidate_id: String,
    pub claim_digest: String,
    pub outcome: ReproductionOutcome,
    pub reproducer_digest: String,
}
