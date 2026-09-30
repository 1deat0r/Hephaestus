//! Data-driven corpus tests (T-003): every scenario class is present, every
//! world regenerates its pinned observation receipt, and every probe lands
//! on its expected hidden verdict.

use std::collections::HashSet;

use synthetic_evaluator::{Scenario, Verdict, evaluate, load_corpus, observation_receipt};

#[test]
fn corpus_covers_all_six_scenario_classes() {
    let corpus = load_corpus();
    assert_eq!(corpus.len(), 6, "one world per scenario class");
    let scenarios: HashSet<Scenario> = corpus.iter().map(|e| e.world.scenario).collect();
    let expected: HashSet<Scenario> = [
        Scenario::TrueMechanism,
        Scenario::FalseMechanism,
        Scenario::Confounder,
        Scenario::ImpossibleConstraint,
        Scenario::AmbiguousEvidence,
        Scenario::ChangedBoundary,
    ]
    .into_iter()
    .collect();
    assert_eq!(scenarios, expected);
    let ids: HashSet<&str> = corpus.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(ids.len(), corpus.len(), "world ids unique");
}

#[test]
fn every_world_reproduces_its_observation_receipt() {
    for entry in load_corpus() {
        let observations = entry.world.sample_observations();
        let receipt = observation_receipt(&observations);
        assert_eq!(
            receipt, entry.observation_receipt_sha256,
            "observation stream drifted for {}",
            entry.id
        );
        // Re-sampling must be bit-identical too.
        assert_eq!(observations, entry.world.sample_observations());
    }
}

#[test]
fn every_probe_lands_on_its_expected_hidden_verdict() {
    let mut checked = 0;
    for entry in load_corpus() {
        let observations = entry.world.sample_observations();
        for probe in &entry.probes {
            let assessment = evaluate(&probe.candidate, &entry.world, &observations);
            assert_eq!(
                assessment.verdict, probe.expected,
                "{} / '{}' ({}) -> {}",
                entry.id, probe.label, probe.candidate.form as u8, assessment.reason
            );
            checked += 1;
        }
    }
    assert!(
        checked >= 14,
        "expected the full probe table, checked {checked}"
    );
}

#[test]
fn corpus_json_never_carries_probe_verdicts_into_observations() {
    // Observations are the only worker-facing output of a world; assert the
    // serialized stream contains no corpus metadata markers.
    for entry in load_corpus() {
        let text = serde_json::to_string(&entry.world.sample_observations()).unwrap();
        for marker in [
            "scenario",
            "truth",
            "regime",
            "requirement",
            "expected",
            "confirmed",
        ] {
            assert!(
                !text.contains(marker),
                "marker {marker:?} leaked into observations of {}",
                entry.id
            );
        }
    }
}

#[test]
fn verdicts_round_trip_through_json() {
    for verdict in [
        Verdict::Confirmed,
        Verdict::Rejected,
        Verdict::Inconclusive,
        Verdict::Infeasible,
        Verdict::OutOfScope,
    ] {
        let text = serde_json::to_string(&verdict).unwrap();
        let back: Verdict = serde_json::from_str(&text).unwrap();
        assert_eq!(back, verdict);
    }
}
