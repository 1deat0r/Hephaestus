//! Independent reproduction record (T-034, IMPLEMENTATION_PLAN:114).
//!
//! The first candidate whose novelty and utility claims warrant it gets
//! a typed reproduction contract: independence enforced by reproducer
//! identity, outcomes retain Inconclusive and Refuted as first-class
//! results, and no external confirmation is fabricated — without a real
//! independent reproducer the record stays PendingExternal.

pub mod record;

pub use record::{Candidate, OutcomeError, ReproductionOutcome, ReproductionRecord, WarrantError};

use std::collections::BTreeSet;

/// Does this candidate warrant reproduction? Requires BOTH declared
/// claims, and first-candidate-only (no existing reproduction).
pub fn warrants_reproduction(
    candidate: &Candidate,
    existing_candidate_ids: &BTreeSet<String>,
) -> Result<(), WarrantError> {
    if candidate.novelty_claim.trim().is_empty() {
        return Err(WarrantError::NoNoveltyClaim);
    }
    if candidate.utility_claim.trim().is_empty() {
        return Err(WarrantError::NoUtilityClaim);
    }
    if existing_candidate_ids.contains(&candidate.candidate_id) {
        return Err(WarrantError::AlreadyReproduced);
    }
    Ok(())
}

/// Record a reproduction outcome. Independence is enforced: the
/// reproducer digest must differ from the claimant digest, and a
/// Confirmed outcome requires the analysis endpoint to have been
/// registered (R-037). Refuted/Inconclusive are recorded as-is — a
/// negative result is a valid result.
pub fn record_outcome(
    candidate: &Candidate,
    outcome: ReproductionOutcome,
    reproducer_digest: &str,
    endpoint_registered: bool,
) -> Result<ReproductionRecord, OutcomeError> {
    if reproducer_digest == candidate.claimant_digest {
        return Err(OutcomeError::NotIndependent);
    }
    if outcome == ReproductionOutcome::Confirmed && !endpoint_registered {
        return Err(OutcomeError::UnregisteredEndpoint);
    }
    Ok(ReproductionRecord {
        candidate_id: candidate.candidate_id.clone(),
        claim_digest: candidate.claim_digest.clone(),
        outcome,
        reproducer_digest: reproducer_digest.to_string(),
    })
}
