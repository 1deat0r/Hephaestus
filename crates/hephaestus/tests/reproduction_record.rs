//! Independent reproduction (T-034).
//!
//! Integration tests at the public seam: `warrants_reproduction`,
//! `record_outcome`.

use hephaestus::reproduction::record::{OutcomeError, ReproductionOutcome, WarrantError};
use hephaestus::reproduction::{record_outcome, warrants_reproduction};
use std::collections::BTreeSet;

fn s(v: &str) -> String {
    v.to_string()
}

fn candidate() -> hephaestus::reproduction::record::Candidate {
    hephaestus::reproduction::record::Candidate {
        candidate_id: s("cand-1"),
        novelty_claim: s("new mechanism X"),
        utility_claim: s("improves metric M"),
        claim_digest: s("claim-digest-1"),
        analysis_endpoint: s("endpoint://analysis-1"),
        claimant_digest: s("claimant-digest"),
    }
}

#[test]
fn warrant_requires_both_claims_and_first_candidate() {
    // Full claims, no prior reproduction: warrants.
    assert_eq!(
        warrants_reproduction(&candidate(), &BTreeSet::new()),
        Ok(())
    );
    // Missing novelty: refused.
    let mut c = candidate();
    c.novelty_claim = s("  ");
    assert_eq!(
        warrants_reproduction(&c, &BTreeSet::new()),
        Err(WarrantError::NoNoveltyClaim)
    );
    // Missing utility: refused.
    let mut c = candidate();
    c.utility_claim = String::new();
    assert_eq!(
        warrants_reproduction(&c, &BTreeSet::new()),
        Err(WarrantError::NoUtilityClaim)
    );
    // Already reproduced (first candidate only): refused.
    let mut ids = BTreeSet::new();
    ids.insert(s("cand-1"));
    assert_eq!(
        warrants_reproduction(&candidate(), &ids),
        Err(WarrantError::AlreadyReproduced)
    );
}

#[test]
fn recording_enforces_independence_and_endpoint_registration() {
    // Independent reproducer, registered endpoint, Confirmed: recorded.
    let rec = record_outcome(
        &candidate(),
        ReproductionOutcome::Confirmed,
        "reproducer-digest",
        true,
    )
    .expect("recorded");
    assert_eq!(rec.outcome, ReproductionOutcome::Confirmed);
    // Same digest as claimant: NOT independent, refused.
    assert_eq!(
        record_outcome(
            &candidate(),
            ReproductionOutcome::Confirmed,
            "claimant-digest",
            true
        ),
        Err(OutcomeError::NotIndependent)
    );
    // Confirmed without registered endpoint: refused (R-037).
    assert_eq!(
        record_outcome(
            &candidate(),
            ReproductionOutcome::Confirmed,
            "reproducer-digest",
            false
        ),
        Err(OutcomeError::UnregisteredEndpoint)
    );
}

#[test]
fn negative_and_inconclusive_outcomes_are_first_class() {
    // Refuted is a valid recorded outcome.
    let rec = record_outcome(
        &candidate(),
        ReproductionOutcome::Refuted,
        "reproducer-digest",
        false,
    )
    .expect("refuted recorded");
    assert_eq!(rec.outcome, ReproductionOutcome::Refuted);
    // Inconclusive is a valid recorded outcome.
    let rec = record_outcome(
        &candidate(),
        ReproductionOutcome::Inconclusive,
        "reproducer-digest",
        false,
    )
    .expect("inconclusive recorded");
    assert_eq!(rec.outcome, ReproductionOutcome::Inconclusive);
    // PendingExternal: no fabricated confirmation without a reproducer.
    let rec = record_outcome(
        &candidate(),
        ReproductionOutcome::PendingExternal,
        "reproducer-digest",
        false,
    )
    .expect("pending recorded");
    assert_eq!(rec.outcome, ReproductionOutcome::PendingExternal);
}

#[test]
fn twin_run_byte_identical() {
    let a = record_outcome(
        &candidate(),
        ReproductionOutcome::Confirmed,
        "reproducer-digest",
        true,
    )
    .unwrap();
    let b = record_outcome(
        &candidate(),
        ReproductionOutcome::Confirmed,
        "reproducer-digest",
        true,
    )
    .unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
