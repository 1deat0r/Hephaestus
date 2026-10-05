//! Evaluation suite (T-026, R-074/R-075; T-052 R-073 admission gate).
//!
//! Integration tests at the public seam: `define_baseline`,
//! `check_matched`, `ablation`.

use hephaestus::evalsuite::record::{AblationKind, ArmKind, ArmResult, BaselineArm, Envelope};
use hephaestus::evalsuite::{
    BenchmarkError, EvalSuite, OriginatedHypothesis, ablation, begin_benchmark, check_matched,
    define_baseline,
};

fn s(v: &str) -> String {
    v.to_string()
}

/// The shared approved envelope (campaign: identical across arms).
fn envelope() -> Envelope {
    Envelope {
        compute_budget: 100,
        tool_access: vec![s("search"), s("files"), s("decision-model")],
        model_access: s("provider-neutral-json"),
        tuning_record: s("2h tuning, recorded"),
    }
}

fn arm(id: &str, kind: ArmKind) -> BaselineArm {
    BaselineArm {
        id: s(id),
        kind,
        envelope: envelope(),
    }
}

// ---- Ticket 01: arms + matching ----

#[test]
fn four_baseline_arms_defined_and_matched() {
    let mut suite = EvalSuite::default();
    for (id, kind) in [
        ("arm-retrieval", ArmKind::ActiveRetrievalModel),
        ("arm-genreview", ArmKind::FixedGenerateReview),
        ("arm-search", ArmKind::SimpleSearch),
        ("arm-domain", ArmKind::DomainMethod),
    ] {
        define_baseline(&mut suite, arm(id, kind)).expect("registered");
    }
    // Identical envelopes: matched.
    assert!(check_matched(&suite).is_ok());
    // Duplicate id refused.
    assert!(define_baseline(&mut suite, arm("arm-search", ArmKind::SimpleSearch)).is_err());
    // Zero budget refused (measures nothing).
    let mut zero = arm("arm-zero", ArmKind::SimpleSearch);
    zero.envelope.compute_budget = 0;
    assert!(define_baseline(&mut suite, zero).is_err());
}

#[test]
fn unmatched_envelope_named_per_arm_and_resource() {
    let mut suite = EvalSuite::default();
    define_baseline(&mut suite, arm("a", ArmKind::SimpleSearch)).unwrap();
    // Budget differs.
    let mut b = arm("b", ArmKind::ActiveRetrievalModel);
    b.envelope.compute_budget = 50;
    define_baseline(&mut suite, b).unwrap();
    // Model access differs.
    let mut c = arm("c", ArmKind::FixedGenerateReview);
    c.envelope.model_access = s("premium-model");
    define_baseline(&mut suite, c).unwrap();
    // Tool access differs.
    let mut d = arm("d", ArmKind::DomainMethod);
    d.envelope.tool_access = vec![s("search")];
    define_baseline(&mut suite, d).unwrap();
    // Missing tuning record.
    let mut e = arm("e", ArmKind::SimpleSearch);
    e.envelope.tuning_record = String::new();
    define_baseline(&mut suite, e).unwrap();

    // R-074/AT-074 negative case: a campaign arm with unequal model or
    // tool access -- the comparison is flagged invalid until budgets and
    // access are reconciled (per-arm, per-resource mismatches named).
    let report = check_matched(&suite).expect_err("mismatches exist");
    let has = |arm: &str, res: &str| {
        report
            .mismatches
            .iter()
            .any(|m| m.arm_id == arm && m.resource == res)
    };
    assert!(has("b", "compute_budget"));
    assert!(has("c", "model_access"));
    assert!(has("d", "tool_access"));
    assert!(has("e", "tuning_record"));
}

// ---- Ticket 02: ablations + separated results ----

#[test]
fn single_ingredient_ablations() {
    let a = arm("arm-retrieval", ArmKind::ActiveRetrievalModel);
    // The four named ablation kinds.
    for kind in [
        AblationKind::RemoveGraph,
        AblationKind::RemoveReview,
        AblationKind::RemoveDecomposition,
        AblationKind::RemoveOptionalDecisionModel,
    ] {
        let spec = ablation(&a, kind).expect("ablation built");
        assert_eq!(spec.of_arm_id, "arm-retrieval");
        assert_eq!(spec.kind, kind);
    }
    // Cannot remove an ingredient the arm lacks.
    let mut bare = arm("bare", ArmKind::SimpleSearch);
    bare.envelope.tool_access = vec![s("search")];
    assert!(ablation(&bare, AblationKind::RemoveOptionalDecisionModel).is_err());
}

#[test]
fn result_fields_stay_separate() {
    // R-075/AT-075 negative case: a high model-judge score on a
    // retrospective task -- the four fields stay separate, so the report
    // cannot convert a judge score into prospective invention validation.
    // R-075: judge scores, rediscovery, prospective novelty, independent
    // replication are four separate optional fields — never collapsed.
    let r = ArmResult {
        judge_score: Some(s("0.72")),
        rediscovery_status: Some(s("none-within-scope")),
        novelty_status: Some(s("unresolved")),
        replication_status: None, // not yet replicated: honest absence
    };
    assert!(r.replication_status.is_none());
    assert!(r.novelty_status.as_deref() == Some("unresolved"));
}

#[test]
fn twin_run_byte_identical() {
    let mut a = EvalSuite::default();
    let mut b = EvalSuite::default();
    for suite in [&mut a, &mut b] {
        define_baseline(suite, arm("x", ArmKind::SimpleSearch)).unwrap();
    }
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}

// ---- Ticket 01: R-073 domain-only discovery admission ----

#[test]
fn at_073_the_benchmark_refuses_to_run_without_supplied_hypotheses() {
    // R-073 negative case: run the full benchmark without supplying
    // hypotheses — the campaign evaluator refuses admission.
    let err = begin_benchmark(&[]).expect_err("empty benchmark must not start");
    assert!(
        matches!(err, BenchmarkError::NoSuppliedHypotheses),
        "{err:?}"
    );
}

#[test]
fn at_073_the_generator_is_evaluated_on_independently_originated_inputs() {
    // Required outcome: evaluation runs on hypotheses with their own
    // opportunity + mechanism lineage.
    let ok = begin_benchmark(&[
        OriginatedHypothesis {
            hypothesis_id: s("h-1"),
            opportunity_id: s("opp-7"),
            mechanism_id: s("mech-3"),
        },
        OriginatedHypothesis {
            hypothesis_id: s("h-2"),
            opportunity_id: s("opp-9"),
            mechanism_id: s("mech-5"),
        },
    ])
    .expect("fully lineaged hypotheses admit");
    assert_eq!(ok.hypothesis_ids, vec![s("h-1"), s("h-2")]);

    // A hypothesis without an opportunity is not independently
    // originated...
    let err = begin_benchmark(&[OriginatedHypothesis {
        hypothesis_id: s("h-seeded"),
        opportunity_id: s(""),
        mechanism_id: s("mech-3"),
    }])
    .expect_err("missing opportunity lineage refuses");
    assert!(
        matches!(err, BenchmarkError::NotIndependentlyOriginated { ref hypothesis_id } if hypothesis_id == "h-seeded"),
        "{err:?}"
    );

    // ...and neither is one without a mechanism.
    let err = begin_benchmark(&[OriginatedHypothesis {
        hypothesis_id: s("h-nomech"),
        opportunity_id: s("opp-7"),
        mechanism_id: s(""),
    }])
    .expect_err("missing mechanism lineage refuses");
    assert!(
        matches!(err, BenchmarkError::NotIndependentlyOriginated { ref hypothesis_id } if hypothesis_id == "h-nomech"),
        "{err:?}"
    );
}
