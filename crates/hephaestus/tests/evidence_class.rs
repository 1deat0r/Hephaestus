//! Evidence kinds distinct (T-057, R-007/AT-007; MASTER_SPEC §3).
//!
//! Red-first seam tests against `knowledge::attest_evidence_class`.
//! Deny-first: repetition of a model statement never promotes it to
//! empirical evidence.

use hephaestus::contracts::generated::EvidenceEvidenceType;
use hephaestus::knowledge::{ClassifyError, StatementIngestion, attest_evidence_class};

fn model_statement(ingestions: usize) -> StatementIngestion {
    StatementIngestion {
        text: "the throughput will improve if we add a cache".to_string(),
        ingestions,
        grounded_span: None,
    }
}

#[test]
fn at_007_repetition_never_promotes_a_model_statement_to_empirical_evidence() {
    // R-007 negative case: ingest an unsupported model statement
    // repeatedly — no repetition promotes it to empirical evidence.
    let repeated = model_statement(7);
    let err = attest_evidence_class(EvidenceEvidenceType::Observation, &repeated)
        .expect_err("an ungrounded model statement must never attest as observation");
    match err {
        ClassifyError::UngroundedObservation { ingestions } => {
            assert_eq!(ingestions, 7, "the seen count rides in the rejection");
        }
    }

    // The same input declared as what it IS stands, repetitions and all.
    attest_evidence_class(EvidenceEvidenceType::ModelJudgment, &repeated)
        .expect("a model judgment stands however often it re-appears");
    attest_evidence_class(EvidenceEvidenceType::Assumption, &model_statement(3))
        .expect("assumptions stand ungrounded");
    attest_evidence_class(EvidenceEvidenceType::Derivation, &model_statement(1))
        .expect("derivations stand ungrounded");
}

#[test]
fn at_007_a_grounded_observation_attests() {
    let grounded = StatementIngestion {
        text: "rebuild wall time measured at 4.2 s".to_string(),
        ingestions: 1,
        grounded_span: Some(hephaestus::knowledge::Span {
            source: 0,
            start: 0,
            end: 24,
            text: "rebuild wall time measured at 4.2 s".to_string(),
            transforms: vec![],
        }),
    };
    attest_evidence_class(EvidenceEvidenceType::Observation, &grounded)
        .expect("a span-grounded observation attests");
}

#[test]
fn at_007_the_four_kinds_are_pairwise_distinct() {
    // Distinctness: same text under different kinds is a different
    // record identity — kinds round-trip through the schema enum
    // unchanged and compare unequal.
    let kinds = [
        EvidenceEvidenceType::Observation,
        EvidenceEvidenceType::Assumption,
        EvidenceEvidenceType::ModelJudgment,
        EvidenceEvidenceType::Derivation,
    ];
    for (i, a) in kinds.iter().enumerate() {
        let round: EvidenceEvidenceType =
            serde_json::from_str(&serde_json::to_string(a).unwrap()).unwrap();
        assert_eq!(&round, a, "serde round-trip preserves kind {a:?}");
        for b in kinds.iter().skip(i + 1) {
            assert_ne!(a, b, "kinds must be distinct: {a:?} vs {b:?}");
        }
    }
}
