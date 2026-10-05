//! Release packet (T-028, R-077/R-078/R-080, R-093 (release stays
//! EXPERIMENTAL unless qualification holds — scope_label,
//! AT-105 (positive/negative yield, originality and economics stay
//! independent with explicit costs)); T-049 R-076 target labeling).
//!
//! Integration tests at the public seam: `scope_label`, `assemble_release`.

use hephaestus::release::record::{
    Finding, Measurement, OutcomeCounts, PerformanceClaim, QualificationInputs, ReleaseBlock,
    ReleasePacket, ReproducibilityReport, ScopeLabel, Severity, Uncertainty, scope_label,
};
use hephaestus::release::{achieved_benchmarks, assemble_release};

fn s(v: &str) -> String {
    v.to_string()
}

fn uncertainty() -> Uncertainty {
    Uncertainty {
        suite_id: s("suite-7"),
        n: 42,
        interval: (0.05, 0.25),
    }
}

fn packet() -> ReleasePacket {
    ReleasePacket {
        label: ScopeLabel::Experimental,
        declared_scope: s("context-assembly, local fixture repositories"),
        findings: vec![],
        uncertainty: uncertainty(),
        outcome_counts: OutcomeCounts {
            positive: 3,
            negative: 1,
            inconclusive: 2,
            invalid: 1,
            blocked: 1,
        },
        cost_latency: vec![Measurement {
            kind: s("generation"),
            quantity: s("4.0"),
            unit: s("CPU-hours"),
        }],
        reproducibility: ReproducibilityReport {
            environment_pins: vec![(s("rustc"), s("1.90.0"))],
            repro_commands: vec![s("make ci")],
            artifact_verifications: vec![(s("artifact-1"), true)],
        },
        unresolved_questions: vec![s("does the mechanism hold on deep topologies?")],
        performance_claims: vec![],
    }
}

// ---- Ticket 01: findings + scope labels ----

#[test]
fn scope_label_defaults_experimental_requires_all_inputs() {
    // Default: EXPERIMENTAL.
    assert_eq!(
        scope_label(&QualificationInputs::default()),
        ScopeLabel::Experimental
    );
    // PARTIAL qualification is still EXPERIMENTAL (no halfway label).
    let partial = QualificationInputs {
        frozen_campaign: true,
        oracle_method_qualified: true,
        guardrails_passing: true,
        independent_reproduction: false, // missing
        security_gates_passing: true,
    };
    assert_eq!(scope_label(&partial), ScopeLabel::Experimental);
    // All five: QUALIFIED_FOR_DECLARED_SCOPE.
    let full = QualificationInputs {
        frozen_campaign: true,
        oracle_method_qualified: true,
        guardrails_passing: true,
        independent_reproduction: true,
        security_gates_passing: true,
    };
    assert_eq!(scope_label(&full), ScopeLabel::QualifiedForDeclaredScope);
}

// ---- Ticket 02: assembly + gate ----

#[test]
fn critical_unresolved_in_scope_blocks() {
    let mut p = packet();
    p.findings = vec![Finding {
        id: s("sec-1"),
        severity: Severity::Critical,
        resolved: false,
        in_scope: true,
        description: s("unvalidated path traversal in tool contract"),
        // R-077/AT-077 negative case: a known critical issue left open --
        // release stays blocked for the affected scope (assembled below).
    }];
    assert_eq!(
        assemble_release(p),
        Err(ReleaseBlock::CriticalUnresolvedInScope(s("sec-1")))
    );
    // Resolved: passes.
    let mut p = packet();
    p.findings = vec![Finding {
        id: s("sec-1"),
        severity: Severity::Critical,
        resolved: true,
        in_scope: true,
        description: s("fixed and verified"),
    }];
    assert!(assemble_release(p).is_ok());
    // Critical but OUT of scope: recorded, non-blocking (scope honesty).
    let mut p = packet();
    p.findings = vec![Finding {
        id: s("sec-2"),
        severity: Severity::Critical,
        resolved: false,
        in_scope: false,
        description: s("out of declared scope"),
    }];
    let out = assemble_release(p).expect("assembled");
    assert_eq!(out.findings.len(), 1, "finding recorded");
}

#[test]
fn uncertainty_required_and_no_universal_field() {
    // R-078/AT-078 negative case: zero failures in a small sample --
    // the report still gives independent n, failure counts and a scoped
    // qualified interval; missing uncertainty blocks. Zero failures never
    // becomes a zero-risk claim (no universal-reliability field exists).
    let mut p = packet();
    p.uncertainty = Uncertainty {
        suite_id: s(""),
        n: 0,
        interval: (0.0, 0.0),
    };
    assert_eq!(assemble_release(p), Err(ReleaseBlock::MissingUncertainty));
    // The packet struct has no universal-reliability field: uncertainty
    // is finite-suite only (suite id + n + interval).
    let p = packet();
    assert_eq!(p.uncertainty.n, 42);
    assert!(!p.uncertainty.suite_id.is_empty());
}

#[test]
fn five_outcome_classes_and_quantities_only() {
    let out = assemble_release(packet()).expect("assembled");
    // R-080: all five classes present.
    let c = out.outcome_counts;
    let _ = (c.positive, c.negative, c.inconclusive, c.invalid, c.blocked);
    // Priced measurement refused (R-103 continuity).
    let mut p = packet();
    p.cost_latency.push(Measurement {
        kind: s("human_intervention"),
        quantity: s("1.0"),
        unit: s("$-hours"),
    });
    assert_eq!(assemble_release(p), Err(ReleaseBlock::PricedMeasurement));
    // Missing repro commands blocks.
    let mut p = packet();
    p.reproducibility.repro_commands.clear();
    assert_eq!(
        assemble_release(p),
        Err(ReleaseBlock::MissingReproducibility)
    );
}

#[test]
fn twin_run_byte_identical() {
    let a = assemble_release(packet()).unwrap();
    let b = assemble_release(packet()).unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}

// ---- Ticket 01: R-076 provisional targets vs measured benchmarks ----

#[test]
fn at_076_a_provisional_p95_target_never_reaches_the_achieved_funnel() {
    // R-076 negative case: render an unimplemented feature with a p95
    // target — the dossier or UI cannot display it as an achieved
    // benchmark (AT-076).
    let mut p = packet();
    p.performance_claims = vec![PerformanceClaim::ProvisionalTarget {
        feature: s("unimplemented-widget"),
        metric_label: s("p95 latency"),
        target_value: s("<100 ms"),
    }];
    // Recording a section-26 provisional target is legal...
    assemble_release(p.clone()).expect("provisional targets may be recorded");
    // ...but the single achieved-display funnel never emits it.
    assert!(
        achieved_benchmarks(&p).is_empty(),
        "provisional target leaked into the achieved-benchmark display"
    );
}

#[test]
fn at_076_a_measured_claim_without_receipt_or_machine_is_refused() {
    // A target dressed up as a measurement must not pass the verifier.
    let mut p = packet();
    p.performance_claims = vec![PerformanceClaim::MeasuredBenchmark {
        feature: s("state-operations"),
        metric_label: s("p95 latency"),
        measured_value: s("7"),
        unit: s("ms"),
        benchmark_receipt: s(""),
        reference_machine: s(""),
    }];
    let err = assemble_release(p).expect_err("unmeasured target cannot pass as achieved");
    assert!(
        matches!(err, ReleaseBlock::UnmeasuredTargetDisplayed(ref f) if f == "state-operations"),
        "{err:?}"
    );
}

#[test]
fn at_076_a_fully_pinned_measured_claim_passes_and_is_funneled() {
    let mut p = packet();
    p.performance_claims = vec![PerformanceClaim::MeasuredBenchmark {
        feature: s("state-operations"),
        metric_label: s("p95 latency"),
        measured_value: s("7"),
        unit: s("ms"),
        benchmark_receipt: s("bench-918"),
        reference_machine: s("ref-machine-7 (recorded)"),
    }];
    assemble_release(p.clone()).expect("fully pinned measurement passes");
    let achieved = achieved_benchmarks(&p);
    assert_eq!(
        achieved.len(),
        1,
        "measured claim is displayable as achieved"
    );
}
