//! T-048 ticket 01 — external-act gate (R-045/AT-045; MASTER_SPEC §15
//! "Invention Engine and dossier exporter"). Deny-first: a positive
//! prototype result never authorizes adoption, publication, or
//! manufacturing; each act needs its own subject-bound authorization.

use hephaestus::realization::{
    ActError, ExternalAct, Justification, RealizationGate, record_authorization,
};

fn positive_justification() -> Justification {
    Justification::PositivePrototypeResult {
        result_digest: "proto-result-42".to_string(),
    }
}

fn adoption_authorization(
    subject: &str,
    digest: &str,
) -> hephaestus::realization::ActAuthorization {
    hephaestus::realization::ActAuthorization {
        act: ExternalAct::Adoption,
        subject: subject.to_string(),
        subject_digest: digest.to_string(),
        granted_by: "standing-authority".to_string(),
        authority_receipt: "receipt-adoption-1".to_string(),
    }
}

#[test]
fn at_045_a_positive_prototype_result_never_authorizes_publication() {
    // R-045 negative case: finish a positive prototype and request
    // automatic public deployment — the prototype result does not
    // itself authorize the external action.
    let gate = RealizationGate::new();
    let err = gate
        .request(
            ExternalAct::Publication,
            "candidate-1",
            "digest-v1",
            &positive_justification(),
        )
        .expect_err("a positive result alone must never authorize publication");
    match err {
        ActError::ResultNeverAuthorizes {
            act,
            justification_digest,
        } => {
            assert_eq!(act, ExternalAct::Publication);
            assert_eq!(justification_digest, "proto-result-42");
        }
        other => panic!("expected ResultNeverAuthorizes, got {other:?}"),
    }

    // The same is true for adoption and manufacturing: no act is ever
    // authorized by the result.
    for act in [ExternalAct::Adoption, ExternalAct::Manufacturing] {
        let err = gate
            .request(act, "candidate-1", "digest-v1", &positive_justification())
            .expect_err("a positive result alone must never authorize");
        assert!(
            matches!(err, ActError::ResultNeverAuthorizes { .. }),
            "{act:?}: {err:?}"
        );
    }
}

#[test]
fn authorizations_are_separate_per_act() {
    // R-045: adoption, publication, and manufacturing each need their
    // OWN authorization — one act's grant never covers another.
    let mut gate = RealizationGate::new();
    record_authorization(
        &mut gate,
        adoption_authorization("candidate-1", "digest-v1"),
    );

    // The adoption authorization is honored for adoption itself.
    gate.request(
        ExternalAct::Adoption,
        "candidate-1",
        "digest-v1",
        &positive_justification(),
    )
    .expect("the recorded adoption authorization covers adoption");

    // ...but covers neither of the other two acts.
    for act in [ExternalAct::Publication, ExternalAct::Manufacturing] {
        let err = gate
            .request(act, "candidate-1", "digest-v1", &positive_justification())
            .expect_err("adoption authorization must not cover other acts");
        assert!(
            matches!(err, ActError::MissingSeparateAuthorization { .. }),
            "{act:?}: {err:?}"
        );
    }

    // A different subject's authorization covers nothing here either.
    let err = gate
        .request(
            ExternalAct::Adoption,
            "candidate-2",
            "digest-v9",
            &positive_justification(),
        )
        .expect_err("authorizations are subject-bound");
    assert!(
        matches!(err, ActError::ResultNeverAuthorizes { .. }),
        "{err:?}"
    );
}

#[test]
fn authorizations_are_version_bound_by_subject_digest() {
    // "Complete version-bound runtime evidence": the authorization is
    // pinned to the subject DIGEST, not merely the subject name.
    let mut gate = RealizationGate::new();
    record_authorization(
        &mut gate,
        adoption_authorization("candidate-1", "digest-v1"),
    );

    let err = gate
        .request(
            ExternalAct::Adoption,
            "candidate-1",
            "digest-v2",
            &positive_justification(),
        )
        .expect_err("a stale subject digest must not ride an old authorization");
    assert!(
        matches!(err, ActError::AuthorizationMismatch { act } if act == ExternalAct::Adoption),
        "{err:?}"
    );

    let receipt = gate
        .request(
            ExternalAct::Adoption,
            "candidate-1",
            "digest-v1",
            &positive_justification(),
        )
        .expect("matching subject digest authorizes");
    assert_eq!(receipt.act, ExternalAct::Adoption);
    assert_eq!(receipt.authorization_digest, "receipt-adoption-1");
}
