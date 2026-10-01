//! Release packet (T-028, R-077/R-078/R-080).
//!
//! Integration tests at the public seam: `scope_label`, `assemble_release`.

use hephaestus::release::assemble_release;
use hephaestus::release::record::{
    Finding, Measurement, OutcomeCounts, QualificationInputs, ReleaseBlock, ReleasePacket,
    ReproducibilityReport, ScopeLabel, Severity, Uncertainty, scope_label,
};

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
    // R-078: missing uncertainty blocks.
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
