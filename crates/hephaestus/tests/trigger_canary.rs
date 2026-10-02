//! T-064 / R-115/AT-115 + R-118 (M3 clause 3.5c): recorded triggers, bounded
//! improvement cycles with a recorded no-justified-change outcome, and
//! the monitor that actually STOPS a breached rollout by rolling back
//! to the caller-proven verified incumbent.

use hephaestus::selfimprove::record::{
    BoundKind, CycleOutcome, CyclePolicy, EvaluationManifest, GuardrailIndicators,
    ImprovementLedger, Indicator, MonitorPolicy, TriggerKind,
};
use hephaestus::selfimprove::{
    CycleError, MonitorError, ProposalInput, TriggerError, monitor_deployment, record_trigger,
    run_improvement_cycle,
};

fn s(v: &str) -> String {
    v.to_string()
}

fn manifest() -> EvaluationManifest {
    EvaluationManifest {
        effect_bound: s("no regression"),
        noninferiority_bound: s("quality loss <= 0"),
        sampling_units: s("fixture observation"),
        unsuccessful_run_denominators: s("all fixture runs"),
        tooling_access: s("identical approved envelope"),
        method_qualification: s("mean-difference-z-interval 1.0.0"),
        stopping_rule: s("fixed sample"),
        multiplicity_rule: s("single endpoint"),
        total_resource_budget: 10,
    }
}

fn seeded_input() -> ProposalInput {
    ProposalInput {
        target: s("discovery.generation_bound"),
        incumbent_id: s("bound-8"),
        incumbent_digest: s("d8"),
        challenger_id: s("bound-16"),
        challenger_digest: s("d16"),
        supporting_observation_ids: vec![s("t0"), s("t1")],
        fresh_partition_id: s("fresh-partition-y"),
        budget_from: s("standing-envelope-1"),
        intended_primary_benefit: s("deeper operator sweep"),
        guardrails: vec![s("no regression")],
        rollout_artifact_digest: s("rollout-b16"),
        rollback_artifact_digest: s("rollback-b8"),
        manifest: manifest(),
    }
}

fn policy() -> CyclePolicy {
    CyclePolicy {
        max_candidates: 2,
        reserved_budget: 10,
        deadline_cycle: 5,
        stop_rules: vec![s("halt after 3 cycles without a justified proposal")],
    }
}

fn deployment() -> hephaestus::selfimprove::Deployment {
    hephaestus::selfimprove::Deployment {
        challenger_id: s("bound-16"),
        challenger_digest: s("d16"),
        incumbent_id: s("bound-8"),
        incumbent_digest: s("d8"),
        grant_scope: vec![s("discovery.generation_bound")],
        assessment_payload_digest: s("d16"),
        observed_scope: s("canary-bounded"),
    }
}

fn indicators(name: &str, observed: f64, limit: f64) -> GuardrailIndicators {
    GuardrailIndicators {
        indicators: vec![Indicator {
            name: s(name),
            kind: BoundKind::AtMost,
            observed,
            limit,
        }],
    }
}

#[test]
fn triggers_persist_on_the_ledger_and_empty_subjects_refuse() {
    let mut ledger = ImprovementLedger::default();
    let trig = record_trigger(
        &mut ledger,
        TriggerKind::MissionCompletion,
        "m-fixture-1",
        "mission A completed with a grounded opportunity",
        1,
    )
    .expect("recorded");
    assert_eq!(trig.kind, TriggerKind::MissionCompletion);
    assert_eq!(ledger.triggers().len(), 1, "the trigger is ON the ledger");

    // Durability: the trigger rides the same persist/recover path.
    let dir = std::env::temp_dir().join(format!(
        "trigger-persist-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    ledger
        .persist(&dir.join("improvement-ledger.json"))
        .expect("persist");
    let text = std::fs::read_to_string(dir.join("improvement-ledger.json")).expect("read");
    let recovered = ImprovementLedger::from_json(&text).expect("recovers");
    assert_eq!(recovered.triggers().len(), 1, "trigger survives restart");
    assert_eq!(recovered.triggers()[0], ledger.triggers()[0]);
    let _ = std::fs::remove_dir_all(&dir);

    // Empty subject refuses (typed).
    let err = record_trigger(&mut ledger, TriggerKind::OwnerRequest, "", "owner asked", 2)
        .expect_err("an empty subject must refuse");
    assert!(matches!(err, TriggerError::EmptySubject), "{err:?}");
}

#[test]
fn cycles_are_bounded_typed_and_record_no_justified_change() {
    let trig = record_trigger(
        &mut ImprovementLedger::default(),
        TriggerKind::MissionCompletion,
        "m-1",
        "completed",
        1,
    )
    .expect("recorded");

    // Missing stop rule.
    let no_rules = CyclePolicy {
        stop_rules: vec![],
        ..policy()
    };
    let err = run_improvement_cycle(&trig, &[seeded_input()], &no_rules)
        .expect_err("a cycle without stop rules must refuse");
    assert!(matches!(err, CycleError::MissingStopRule), "{err:?}");

    // Candidate cap.
    let over_cap = vec![seeded_input(), seeded_input(), seeded_input()];
    let err = run_improvement_cycle(&trig, &over_cap, &policy())
        .expect_err("exceeding the candidate cap must refuse");
    assert!(matches!(err, CycleError::CandidateCapExceeded), "{err:?}");

    // Budget cap (manifest total above the cycle's reserved budget).
    let mut rich = seeded_input();
    rich.manifest.total_resource_budget = 99;
    let err = run_improvement_cycle(&trig, &[rich], &policy())
        .expect_err("over-reserved budgets must refuse");
    assert!(matches!(err, CycleError::BudgetCapExceeded), "{err:?}");

    // Deadline (cycle_index past the declared deadline).
    let late = record_trigger(
        &mut ImprovementLedger::default(),
        TriggerKind::BatchInterval,
        "m-2",
        "batch tick",
        9,
    )
    .expect("recorded");
    let err = run_improvement_cycle(&late, &[seeded_input()], &policy())
        .expect_err("past-deadline cycles must refuse");
    assert!(matches!(err, CycleError::DeadlinePassed), "{err:?}");

    // No justified change: nothing to propose from this trigger.
    let out = run_improvement_cycle(&trig, &[], &policy()).expect("bounded cycle");
    assert!(
        matches!(out, CycleOutcome::NoJustifiedChange { .. }),
        "{out:?}"
    );
    // ...and an unseeded candidate is NOT a justified change either.
    let mut unseeded = seeded_input();
    unseeded.supporting_observation_ids = vec![];
    let out = run_improvement_cycle(&trig, &[unseeded], &policy()).expect("bounded cycle");
    assert!(
        matches!(out, CycleOutcome::NoJustifiedChange { .. }),
        "{out:?}"
    );

    // A seeded candidate inside every bound -> Proposed via the single
    // propose path (observations carried through).
    let out = run_improvement_cycle(&trig, &[seeded_input()], &policy()).expect("bounded cycle");
    match out {
        CycleOutcome::Proposed { candidate } => {
            assert_eq!(candidate.target, "discovery.generation_bound");
            assert_eq!(candidate.supporting_observation_ids, vec![s("t0"), s("t1")]);
        }
        other => panic!("expected Proposed, got {other:?}"),
    }
}

#[test]
fn the_monitor_bounds_checks_and_stops_a_breached_rollout() {
    use hephaestus::selfimprove::record::{MonitorOutcome as M, RollbackReceipt};

    let policy = MonitorPolicy {
        max_checks: 3,
        stop_rules: vec![s("stop the rollout on any guardrail breach")],
        verified_incumbent_digest: s("d8"),
    };

    // All clear within the bound -> Completed.
    let clear = indicators("latency_ms", 90.0, 100.0);
    let out = monitor_deployment(&deployment(), &policy, &[clear.clone(), clear.clone()])
        .expect("within limits");
    assert!(matches!(out, M::Completed { checks: 2 }), "{out:?}");

    // An over-long stream is BOUNDED by max_checks (the declared stop).
    let long = vec![clear.clone(); 10];
    let out = monitor_deployment(&deployment(), &policy, &long).expect("bounded");
    assert!(
        matches!(out, M::Completed { checks: 3 }),
        "monitor stops at max_checks: {out:?}"
    );

    // A breach STOPS the rollout via a real rollback to the proven
    // incumbent, naming the indicator in the retained reason.
    let breach = indicators("latency_ms", 250.0, 100.0);
    let out = monitor_deployment(
        &deployment(),
        &policy,
        &[clear.clone(), breach, clear.clone()],
    )
    .expect("stopping is a successful outcome, not an error");
    match out {
        M::Stopped { checks, rollback } => {
            assert_eq!(checks, 2, "stopped at the breaching check");
            assert_eq!(
                rollback.restored_incumbent_digest, "d8",
                "restores the verified incumbent"
            );
            assert!(
                rollback.reason.contains("latency_ms"),
                "reason retains the violation: {}",
                rollback.reason
            );
            assert!(!rollback.observations.is_empty(), "observations retained");
            let _: RollbackReceipt = rollback;
        }
        other => panic!("expected Stopped, got {other:?}"),
    }

    // The incumbent proof is caller-supplied — a mismatch refuses
    // BEFORE any check (fail-early, never a vacuous rollback).
    let unproven = MonitorPolicy {
        verified_incumbent_digest: s("wrong-digest"),
        ..policy.clone()
    };
    let err = monitor_deployment(&deployment(), &unproven, &[clear])
        .expect_err("an unproven incumbent must refuse before monitoring");
    assert!(
        matches!(err, MonitorError::UnverifiedIncumbent { .. }),
        "{err:?}"
    );

    // Policy validity: stop rules required, bound must be positive.
    let err = monitor_deployment(
        &deployment(),
        &MonitorPolicy {
            stop_rules: vec![],
            ..policy.clone()
        },
        &[],
    )
    .expect_err("monitor without stop rules refuses");
    assert!(matches!(err, MonitorError::MissingStopRule), "{err:?}");
    let err = monitor_deployment(
        &deployment(),
        &MonitorPolicy {
            max_checks: 0,
            ..policy.clone()
        },
        &[],
    )
    .expect_err("zero checks is no monitor at all");
    assert!(matches!(err, MonitorError::InvalidPolicy), "{err:?}");
}
