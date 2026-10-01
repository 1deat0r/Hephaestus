//! Evaluation suite (T-026, R-074/R-075).
//!
//! Integration tests at the public seam: `define_baseline`,
//! `check_matched`, `ablation`.

use hephaestus::evalsuite::record::{AblationKind, ArmKind, ArmResult, BaselineArm, Envelope};
use hephaestus::evalsuite::{EvalSuite, ablation, check_matched, define_baseline};

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
