//! Dossier capture, reproduction, export (T-023, R-044, R-103, §15; T-051 R-082
//! evidence label; AT-002 (no dossier before an experiment — missing raw
//! data refuses export, output stays exploratory)).

use hephaestus::dossier::record::{
    CostReceipt, Deviation, Dossier, Environment, EvidenceLabel, FailureEntry, Lineage,
    NegativeResultKind, ReproductionOutcome, RunRecord, UnmeasuredReason,
};
use hephaestus::dossier::{ExportError, export, reproduce, validate_for_export};

fn s(v: &str) -> String {
    v.to_string()
}

fn run_record() -> RunRecord {
    RunRecord {
        measurements: vec![(s("rebuild_seconds"), 4.2)],
        environment: Environment {
            os: s("linux"),
            tool_versions: vec![(s("rustc"), s("1.90.0"))],
            artifact_digests: vec![s("artifact-1")],
        },
        deviations: vec![Deviation {
            from_plan_field: s("sampling_unit"),
            what_changed: s("3 runs rerun after env flake"),
            reason: s("recorded flake, plan unchanged"),
        }],
        cost_receipts: vec![
            CostReceipt {
                kind: s("generation"),
                quantity: s("1.5"),
                unit: s("CPU-hours"),
                provider_charge: None,
            },
            CostReceipt {
                kind: s("human_intervention"),
                quantity: s("0.5"),
                unit: s("human-hours"),
                provider_charge: None, // quantity, never priced (R-103)
            },
        ],
        failure_history: vec![FailureEntry {
            stage: s("first-build"),
            description: s("missing dependency, fixed in isolation"),
            artifact_digest: s("failed-artifact-1"),
        }],
    }
}

fn same_env() -> Environment {
    Environment {
        os: s("linux"),
        tool_versions: vec![(s("rustc"), s("1.90.0"))],
        artifact_digests: vec![s("artifact-1")],
    }
}

fn dossier() -> Dossier {
    Dossier {
        problem_and_beneficiary: s("rebuild latency; build-tool users"),
        prior_art_reference: s("priorart-report-run-1"),
        mechanism_explanation: s("mtime-keyed invalidation narrows rebuild set"),
        hypothesis_versions: vec![s("h0.9.0"), s("h1.0.0")],
        predeclared_tests: s("frozen plan h1.0.0, paired comparison"),
        environment: same_env(),
        raw_data: vec![(s("rebuild_seconds"), 4.2)],
        analysis: s("paired difference, Bonferroni family alpha 0.025"),
        results_json: s("{\"science\":\"Supported\"}"),
        counterevidence: s("none observed within scope"),
        failure_history: vec![FailureEntry {
            stage: s("first-build"),
            description: s("missing dependency"),
            artifact_digest: s("failed-artifact-1"),
        }],
        deviations: vec![Deviation {
            from_plan_field: s("sampling_unit"),
            what_changed: s("3 reruns"),
            reason: s("flake"),
        }],
        uncertainty: s("single-machine scope; correlated repeats unmodeled"),
        scope_limits: s("local fixture repositories only"),
        cost_ledger: vec![CostReceipt {
            kind: s("human_intervention"),
            quantity: s("0.5"),
            unit: s("human-hours"),
            provider_charge: None,
        }],
        repro_commands: vec![s("cargo test -p hephaestus"), s("make ci")],
        unresolved_risks: s("distribution shift on fresh repositories"),
        next_justified_action: s("hold-out confirmation on fresh episodes"),
        negative_kind: None,
        lineage: Lineage {
            mission_id: s("m-1"),
            opportunity_id: s("o-1"),
            mechanism_id: s("mech-1"),
            hypothesis_version: s("h1.0.0"),
            plan_id: s("plan-1"),
            result_id: s("res-1"),
        },
        evidence_label: EvidenceLabel::Measured {
            run_receipt: s("run-rcpt-1"),
        },
    }
}

// ---- Ticket 01: capture + reproduce ----

#[test]
fn capture_binds_raw_data_and_quantity_costs() {
    let record = run_record();
    assert_eq!(record.measurements, vec![(s("rebuild_seconds"), 4.2)]);
    // R-103: human intervention is a quantity with NO price.
    let human = record
        .cost_receipts
        .iter()
        .find(|c| c.kind == "human_intervention")
        .expect("receipt");
    assert!(human.provider_charge.is_none());
    assert_eq!(human.unit, "human-hours");
    // Deviations + failures retained.
    assert_eq!(record.deviations.len(), 1);
    assert_eq!(record.failure_history.len(), 1);
}

#[test]
fn reproduce_typed_outcomes() {
    let record = run_record();
    // Same env + same measurements -> Reproduced.
    assert_eq!(
        reproduce(&record, &same_env(), &[(s("rebuild_seconds"), 4.2)]),
        ReproductionOutcome::Reproduced
    );
    // Different tool version -> EnvironmentMismatch (typed, not silent).
    let mut other = same_env();
    other.tool_versions = vec![(s("rustc"), s("1.89.0"))];
    assert_eq!(
        reproduce(&record, &other, &[(s("rebuild_seconds"), 4.2)]),
        ReproductionOutcome::EnvironmentMismatch
    );
    // Different digests -> mismatch.
    let mut other = same_env();
    other.artifact_digests = vec![s("artifact-2")];
    assert_eq!(
        reproduce(&record, &other, &[(s("rebuild_seconds"), 4.2)]),
        ReproductionOutcome::EnvironmentMismatch
    );
    // Same env, disagreeing measurement -> Disagrees.
    assert_eq!(
        reproduce(&record, &same_env(), &[(s("rebuild_seconds"), 9.9)]),
        ReproductionOutcome::Disagrees
    );
}

// ---- Ticket 02: export gate + negative semantics ----

#[test]
fn export_gate_refusals_named() {
    let mut d = dossier();
    assert_eq!(validate_for_export(&d), Ok(()));
    // Missing raw data refused.
    d.raw_data.clear();
    assert_eq!(validate_for_export(&d), Err(ExportError::MissingRawData));
    // Both histories empty refused (R-044: both histories accounted).
    let mut d = dossier();
    d.failure_history.clear();
    d.deviations.clear();
    assert_eq!(
        validate_for_export(&d),
        Err(ExportError::MissingFailureHistoryField)
    );
    // Empty repro commands refused (§14:282 continuity).
    let mut d = dossier();
    d.repro_commands.clear();
    assert_eq!(
        validate_for_export(&d),
        Err(ExportError::MissingReproCommands)
    );
    // Priced human time refused (R-103).
    let mut d = dossier();
    d.cost_ledger[0].provider_charge = Some(s("$50"));
    assert_eq!(validate_for_export(&d), Err(ExportError::PricedHumanTime));
}

#[test]
fn negative_dossier_distinguishes_five_kinds() {
    // §15:300: five kinds, each first-class.
    let kinds = [
        NegativeResultKind::InvalidTest,
        NegativeResultKind::FailedImplementation,
        NegativeResultKind::UnsupportedMechanism,
        NegativeResultKind::UncompetitiveEngineeringRealization,
        NegativeResultKind::ResourceLimitedInvestigation,
    ];
    assert_eq!(kinds.len(), 5);
    // R-044 negative case: failed implementation + valid negative test,
    // BOTH histories present with SEPARATE conclusions.
    let mut d = dossier();
    d.negative_kind = Some(NegativeResultKind::FailedImplementation);
    d.results_json = s("{\"science\":\"NotAssessed\",\"execution\":\"Invalid\"}");
    d.counterevidence = s("negative test T-9 passed: mechanism absent");
    // Failure history (implementation failed) and the negative test
    // conclusion (counterevidence field) remain SEPARATE fields.
    assert!(!d.failure_history.is_empty());
    assert!(d.counterevidence.contains("negative test"));
    assert_eq!(validate_for_export(&d), Ok(()));
    let json = export(&d).expect("exports");
    assert!(json.contains("FailedImplementation"));
}

#[test]
fn lineage_recorded_and_export_twin_identical() {
    let d = dossier();
    assert_eq!(d.lineage.hypothesis_version, "h1.0.0");
    assert_eq!(d.lineage.mission_id, "m-1");
    let a = export(&dossier()).unwrap();
    let b = export(&dossier()).unwrap();
    assert_eq!(a, b, "twin-run byte-identical");
    // Exported JSON carries the §15:299 fields.
    assert!(a.contains("raw_data"));
    assert!(a.contains("cost_ledger"));
    assert!(a.contains("repro_commands"));
}

// ---- Ticket 01: R-082 evidence label ----

#[test]
fn at_082_packaged_example_exports_with_unmeasured_synthetic_label() {
    // R-082 negative case: load and export the packaged
    // context-assembly example — no synthetic fixture is rendered as
    // real experimental evidence (AT-082).
    let raw = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/software-mission.json"
    ))
    .expect("packaged example present");
    let example: serde_json::Value = serde_json::from_str(&raw).expect("example parses");
    let records = example["records"].as_array().expect("example records");
    assert!(!records.is_empty());
    for (i, record) in records.iter().enumerate() {
        assert_eq!(
            record["data_origin"], "synthetic_fixture",
            "record {i} of the packaged example is a synthetic fixture at the source"
        );
    }

    let mut d = dossier();
    d.evidence_label = EvidenceLabel::Unmeasured {
        reason: UnmeasuredReason::SyntheticFixture,
        source: s("examples/software-mission.json"),
    };
    let out = export(&d).expect("unmeasured dossiers export");
    let exported: serde_json::Value = serde_json::from_str(&out).expect("export json");
    let label = &exported["evidence_label"];
    assert!(
        label.get("Unmeasured").is_some(),
        "export must carry the unmeasured label: {label:?}"
    );
    assert_eq!(
        label["Unmeasured"]["reason"], "SyntheticFixture",
        "export must carry the synthetic-fixture reason: {label:?}"
    );
}

#[test]
fn at_082_a_measured_label_without_receipt_is_refused() {
    // A measured claim must carry its version-bound receipt.
    let mut d = dossier();
    d.evidence_label = EvidenceLabel::Measured { run_receipt: s("") };
    let err = export(&d).expect_err("measured-without-receipt must refuse");
    assert!(
        matches!(err, ExportError::MeasuredWithoutReceipt),
        "{err:?}"
    );

    // The honest case exports and keeps the receipt visible.
    let mut d = dossier();
    d.evidence_label = EvidenceLabel::Measured {
        run_receipt: s("run-rcpt-77"),
    };
    let out = export(&d).expect("measured with receipt exports");
    assert!(out.contains("run-rcpt-77"), "receipt visible in output");
}

// ---- Ticket 01: R-092 classified bundle exports ----

#[test]
fn at_092_an_unclassified_mixed_bundle_is_refused_by_record() {
    // R-092 negative case: export exploratory, test-ready, and
    // validated records together — without per-record evidence/scope
    // classification the bundle refuses, naming the offender.
    use hephaestus::dossier::export_bundle;
    use hephaestus::dossier::record::{BundleExportError, ClassifiedRecord};
    use hephaestus::lifecycle::HypothesisState;

    let bundle = [
        ClassifiedRecord {
            record_id: s("r-exploratory"),
            status: HypothesisState::Exploratory,
            evidence_ids: vec![],
            scope: s("context-assembly"),
            reproduction: hephaestus::dossier::record::ReproductionOutcome::Reproduced,
        },
        ClassifiedRecord {
            record_id: s("r-test-ready"),
            status: HypothesisState::TestReady,
            evidence_ids: vec![s("ev-1")],
            scope: s(""),
            reproduction: hephaestus::dossier::record::ReproductionOutcome::Disagrees,
        },
    ];
    let err = export_bundle(&bundle).expect_err("unclassified records must refuse");
    match err {
        BundleExportError::MissingEvidence { record_id } => {
            assert_eq!(record_id, "r-exploratory");
        }
        other => panic!("expected MissingEvidence, got {other:?}"),
    }

    // The second record is caught on its own axis (empty scope).
    let err = export_bundle(&bundle[1..]).expect_err("missing scope must refuse");
    assert!(
        matches!(err, BundleExportError::MissingScope { ref record_id } if record_id == "r-test-ready"),
        "{err:?}"
    );
}

#[test]
fn at_092_a_classified_mixed_bundle_exports_with_per_record_facts() {
    // Required outcome: each record carries its own status, evidence,
    // scope, and reproduction state — mixed togetherness without
    // collapse into one label.
    use hephaestus::dossier::export_bundle;
    use hephaestus::dossier::record::{ClassifiedRecord, ReproductionOutcome};
    use hephaestus::lifecycle::HypothesisState;

    let bundle = [
        ClassifiedRecord {
            record_id: s("r-exploratory"),
            status: HypothesisState::Exploratory,
            evidence_ids: vec![s("ev-a")],
            scope: s("context-assembly"),
            reproduction: ReproductionOutcome::Reproduced,
        },
        ClassifiedRecord {
            record_id: s("r-test-ready"),
            status: HypothesisState::TestReady,
            evidence_ids: vec![s("ev-b")],
            scope: s("context-assembly"),
            reproduction: ReproductionOutcome::Disagrees,
        },
        ClassifiedRecord {
            record_id: s("r-validated"),
            status: HypothesisState::Assessed,
            evidence_ids: vec![s("ev-c")],
            scope: s("context-assembly"),
            reproduction: ReproductionOutcome::Reproduced,
        },
    ];
    let out = export_bundle(&bundle).expect("classified mixed bundle exports");
    let json: serde_json::Value = serde_json::from_str(&out).expect("bundle json");
    let records = json.as_array().expect("array bundle");
    assert_eq!(records.len(), 3);
    let statuses: Vec<&str> = records
        .iter()
        .map(|r| r["status"].as_str().expect("status present"))
        .collect();
    assert_eq!(statuses, vec!["Exploratory", "TestReady", "Assessed"]);
    for r in records {
        assert!(!r["evidence_ids"].as_array().expect("eids").is_empty());
        assert!(!r["scope"].as_str().expect("scope").is_empty());
        assert!(r["reproduction"].is_string() || r["reproduction"].is_object());
    }
}
