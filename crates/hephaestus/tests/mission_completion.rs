//! Mission completion (T-053, R-091/AT-091; MASTER_SPEC §31).
//!
//! Red-first seam tests against `hephaestus::mission::complete_mission`.
//! Deny-first: agreement never substitutes evidence.

use hephaestus::mission::{AgentAgreement, CompletionError, EvidenceRef, complete_mission};

fn unanimous() -> AgentAgreement {
    AgentAgreement {
        reviewers: vec![
            "rev-a".to_string(),
            "rev-b".to_string(),
            "rev-c".to_string(),
        ],
    }
}

fn evidence(id: &str, version: &str) -> EvidenceRef {
    EvidenceRef {
        evidence_id: id.to_string(),
        version: version.to_string(),
    }
}

#[test]
fn at_091_unanimous_approval_without_usable_evidence_cannot_complete() {
    // R-091 negative case: return unanimous approval but no usable
    // evidence — the mission cannot claim a validated invention
    // candidate, because no completion record exists.
    let err = complete_mission("m-1", &unanimous(), &[])
        .expect_err("agreement without evidence must never complete a mission");
    assert!(
        matches!(err, CompletionError::AgreementWithoutEvidence),
        "{err:?}"
    );
}

#[test]
fn at_091_malformed_evidence_refs_are_refused_by_name() {
    for bad in [evidence("", "v1"), evidence("ev-1", "")] {
        let err = complete_mission("m-1", &unanimous(), std::slice::from_ref(&bad))
            .expect_err("an unusable evidence ref must refuse");
        match err {
            CompletionError::UnusableEvidence { ref evidence_id } => {
                assert_eq!(evidence_id, &bad.evidence_id);
            }
            other => panic!("expected UnusableEvidence, got {other:?}"),
        }
    }
}

#[test]
fn at_091_well_formed_evidence_completes_and_carries_every_ref() {
    // Required outcome: completion rests on version-bound evidence;
    // the reviewer list does not affect the outcome at all.
    let refs = [evidence("ev-1", "v1"), evidence("ev-2", "v3")];
    let completion = complete_mission("m-1", &unanimous(), &refs)
        .expect("usable evidence completes the mission");
    assert_eq!(completion.mission_id, "m-1");
    assert_eq!(completion.evidence, refs);

    // Same evidence with a different (or empty) agreement still completes.
    let no_agreement = AgentAgreement { reviewers: vec![] };
    let completion = complete_mission("m-1", &no_agreement, &refs)
        .expect("agreement is irrelevant when evidence is usable");
    assert_eq!(completion.evidence, refs);
}
