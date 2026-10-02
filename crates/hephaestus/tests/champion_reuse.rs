//! T-062 / R-119/AT-119 demonstration (T-033 exit clause): the end-to-end
//! controlled self-improvement loop — non-seeded opportunity -> bounded
//! proposal -> protected assessment -> persisted champion -> a LATER
//! mission uses it -> restart preserves state -> injected regression ->
//! rollback -> incumbent reused, with disqualified challengers undeployed.
//!
//! Data only (evaluator_access guard): fixture trace + in-test payloads;
//! digests bind every value the loop consumes.

use hephaestus::knowledge::sha256_hex;
use hephaestus::missionrun::{discovery_bound, run as chain_run};
use hephaestus::selfimprove::record::{
    Assessment, BenefitStatus, EvaluationManifest, GuardrailStatus, ImprovementCandidate,
    ImprovementLedger, LedgerEntry, ProposalInput, StandingGrant,
};
use hephaestus::selfimprove::{
    BoundKind, GuardrailIndicators, Indicator, MonitorOutcome, MonitorPolicy, PromotionRejection,
    active_champion, monitor_deployment, promote, propose_from_observations,
};

const TARGET: &str = "discovery.generation_bound";
const TRACE: &str = include_str!("fixtures/e2e-trace.log");

fn s(v: &str) -> String {
    v.to_string()
}

fn manifest() -> EvaluationManifest {
    EvaluationManifest {
        effect_bound: s("mechanisms_found bound not regressed"),
        noninferiority_bound: s("quality loss <= 0"),
        sampling_units: s("fixture trace observation"),
        unsuccessful_run_denominators: s("all fixture runs"),
        tooling_access: s("identical approved envelope"),
        method_qualification: s("mean-difference-z-interval 1.0.0"),
        stopping_rule: s("fixed sample"),
        multiplicity_rule: s("single endpoint, no correction"),
        total_resource_budget: 10,
    }
}

fn payload(value: u64) -> String {
    serde_json::json!({ "target": TARGET, "value": value }).to_string()
}

/// One mission = the proven chain: M2 discovery slice at the active
/// bound + M3 plan/interpret/export. Receipt records what was used.
fn run_mission(source: &str, bound: u64) -> serde_json::Value {
    let disc = discovery_bound(TRACE, bound).expect("M2 slice completes");
    let chain = chain_run(TRACE, 0.25, None).expect("M3 chain completes");
    serde_json::json!({
        "source": source,
        "bound_used": bound,
        "grounded_opportunities": disc.grounded_opportunities,
        "mechanisms_found": disc.mechanisms_found,
        "science": format!("{:?}", chain.typed.science),
        "dossier_receipt": chain.receipt_sha256,
    })
}

fn grant_for(verified: &[String]) -> StandingGrant {
    StandingGrant {
        covered_targets: vec![TARGET.to_string(), s("prompt"), s("scheduling")],
        protected_targets: vec![s("evaluator"), s("spending-limits"), s("safety-policy")],
        max_budget: 100,
        verified_artifacts: verified.to_vec(),
    }
}

fn supported_assessment(payload_digest: &str) -> Assessment {
    Assessment {
        benefit_status: BenefitStatus::Supported,
        guardrail_status: GuardrailStatus::Passing,
        evaluator_digest: s("protected-evaluator-digest"),
        bound_payload_digest: payload_digest.to_string(),
        challenger_cost: 5,
        confounds_present: false,
    }
}

#[test]
fn r119_full_loop_later_missions_use_the_persisted_champion() {
    let digest8 = sha256_hex(payload(8).as_bytes());
    let digest16 = sha256_hex(payload(16).as_bytes());

    // 1. Broad mission A reveals a NON-SEEDED opportunity (fixture
    //    trace; no hypothesis supplied) at the default bound.
    let mission_a = run_mission("default", 8);
    assert_eq!(mission_a["bound_used"], 8);
    assert_eq!(mission_a["source"], "default");
    assert!(
        mission_a["mechanisms_found"].as_u64().unwrap_or(0) >= 1,
        "opportunity yields mechanisms: {mission_a}"
    );

    // 2. The service proposes a challenger FROM the observations
    //    (deterministic, bounded, evidence-attached).
    let observations: Vec<String> = (0..4).map(|i| format!("t{i}")).collect();
    let candidate = propose_from_observations(ProposalInput {
        target: TARGET.to_string(),
        incumbent_id: s("bound-8"),
        incumbent_digest: digest8.clone(),
        challenger_id: s("bound-16"),
        challenger_digest: digest16.clone(),
        supporting_observation_ids: observations,
        fresh_partition_id: s("fresh-partition-contextA"),
        budget_from: s("standing-envelope-1"),
        intended_primary_benefit: s("deeper operator sweep per opportunity"),
        guardrails: vec![s("mechanisms_found must not regress")],
        rollout_artifact_digest: s("rollout-b16"),
        rollback_artifact_digest: s("rollback-b8"),
        manifest: manifest(),
    })
    .expect("observation-backed proposal");

    // 3. Protected fresh evaluation qualifies it (fixture assessment,
    //    external evaluator identity, bound to the exact payload).
    let assessment = supported_assessment(&digest16);
    let verified = vec![
        digest16.clone(),
        digest8.clone(),
        s("rollout-b16"),
        s("rollback-b8"),
    ];
    let deployment = promote(
        &candidate,
        &assessment,
        &grant_for(&verified),
        &verified,
        true,
    )
    .expect("champion deployed");
    assert_eq!(deployment.challenger_digest, digest16);
    assert_eq!(deployment.incumbent_digest, digest8, "incumbent retained");

    // 4. Persist (append-only ledger, convention: deployed).
    let mut ledger = ImprovementLedger::default();
    ledger.append(LedgerEntry {
        challenger_id: s("bound-16"),
        challenger_digest: digest16.clone(),
        incumbent_id: s("bound-8"),
        incumbent_digest: digest8.clone(),
        outcome: s("deployed"),
        reason: s("supported benefit, guardrails passing, fresh partition"),
        observations: s("mission-A opportunity t0..t3"),
        target: TARGET.to_string(),
        fresh_partition_id: s("fresh-partition-contextA"),
    });

    // 5. A LATER mission resolves and USES the champion: value comes
    //    from the payload the candidate's digest binds.
    let payloads = serde_json::json!({
        "bound-16": payload(16),
        "bound-8": payload(8),
    });
    let resolve = |led: &ImprovementLedger| -> (String, u64) {
        let ch = active_champion(led, TARGET).expect("an active champion");
        let text = payloads[&ch.id].as_str().expect("payload present");
        assert_eq!(
            sha256_hex(text.as_bytes()),
            ch.digest,
            "payload is exactly what the champion digest binds"
        );
        let v = serde_json::from_str::<serde_json::Value>(text).expect("payload json")["value"]
            .as_u64()
            .expect("value");
        (ch.id, v)
    };
    let (id_b, bound_b) = resolve(&ledger);
    assert_eq!((id_b.as_str(), bound_b), ("bound-16", 16));
    let mission_b = run_mission("resolved", bound_b);
    assert_eq!(mission_b["bound_used"], 16);
    assert_eq!(mission_b["source"], "resolved");
    assert!(
        mission_b["mechanisms_found"].as_u64().unwrap_or(0)
            >= mission_a["mechanisms_found"].as_u64().unwrap_or(0),
        "deeper sweep never regresses: a={} b={mission_a}",
        mission_b["mechanisms_found"]
    );

    // 6. Restart preserves the state (R-116): write, re-read, same
    //    resolution, byte-identical subsequent mission receipt.
    let dir = std::env::temp_dir().join(format!(
        "champion-reuse-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("improvement-ledger.json");
    std::fs::write(&path, ledger.to_json()).expect("persist");
    let reloaded =
        ImprovementLedger::from_json(&std::fs::read_to_string(&path).expect("read back"))
            .expect("persisted ledger recovers");
    assert_eq!(
        reloaded.entries(),
        ledger.entries(),
        "restart keeps entries"
    );
    let (id_r, bound_r) = resolve(&reloaded);
    assert_eq!((id_r.as_str(), bound_r), ("bound-16", 16));
    assert_eq!(
        serde_json::to_string(&run_mission("resolved", bound_r)).unwrap(),
        serde_json::to_string(&mission_b).unwrap(),
        "post-restart mission receipt is byte-identical"
    );

    // 7. Injected regression -> rollback (R-118/R-119): restore the
    //    VERIFIED incumbent; reasons and observations retained.
    // 7. The CONTINUOUS (bounded) monitor drives the stop (R-118): a
    //     declared-bound observation stream runs through the wired
    //     guardrail check; the first breach STOPS the rollout with a
    //     real rollback receipt naming the indicator.
    let clear = GuardrailIndicators {
        indicators: vec![Indicator {
            name: s("latency_ms"),
            kind: BoundKind::AtMost,
            observed: 90.0,
            limit: 100.0,
        }],
    };
    let breach = GuardrailIndicators {
        indicators: vec![Indicator {
            name: s("latency_ms"),
            kind: BoundKind::AtMost,
            observed: 250.0,
            limit: 100.0,
        }],
    };
    let monitor_policy = MonitorPolicy {
        max_checks: 3,
        stop_rules: vec![s("stop the rollout on any guardrail breach")],
        verified_incumbent_digest: digest8.clone(),
    };
    let outcome = monitor_deployment(
        &deployment,
        &monitor_policy,
        &[clear.clone(), breach, clear],
    )
    .expect("the monitor runs to its bound");
    let receipt = match outcome {
        MonitorOutcome::Stopped { checks, rollback } => {
            assert_eq!(checks, 2, "stopped at the breaching check");
            assert!(
                rollback.reason.contains("latency_ms"),
                "receipt retains the violation: {}",
                rollback.reason
            );
            rollback
        }
        other => panic!("the injected regression must stop the rollout: {other:?}"),
    };
    assert_eq!(receipt.restored_incumbent_digest, digest8);
    assert!(receipt.reason.contains("canary stop"));
    // Record the rollback on the ledger (outcome convention) and persist.
    let mut restarted = reloaded;
    restarted.append(LedgerEntry {
        challenger_id: s("bound-16"),
        challenger_digest: digest16.clone(),
        incumbent_id: s("bound-8"),
        incumbent_digest: digest8.clone(),
        outcome: s("rolled_back"),
        reason: receipt.reason.clone(),
        observations: receipt.observations.clone(),
        target: TARGET.to_string(),
        fresh_partition_id: s("fresh-partition-contextA"),
    });
    std::fs::write(&path, restarted.to_json()).expect("persist post-rollback");
    // 8. Later missions use the RESTORED incumbent again — and survive
    //    a second restart.
    let again = ImprovementLedger::from_json(&std::fs::read_to_string(&path).expect("second read"))
        .expect("persisted ledger recovers again");
    let (id_c, bound_c) = resolve(&again);
    assert_eq!((id_c.as_str(), bound_c), ("bound-8", 8));
    let mission_c = run_mission("resolved", bound_c);
    assert_eq!(
        mission_c["bound_used"], 8,
        "rollback puts the incumbent back to work"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn r119_disqualified_challengers_stay_undeployed() {
    let digest8 = sha256_hex(payload(8).as_bytes());
    let digest16 = sha256_hex(payload(16).as_bytes());
    let observations = vec![s("t0")];
    let base = || ImprovementCandidate {
        incumbent_id: s("bound-8"),
        incumbent_digest: digest8.clone(),
        challenger_id: s("bound-16"),
        challenger_digest: digest16.clone(),
        target: TARGET.to_string(),
        supporting_observation_ids: observations.clone(),
        fresh_partition_id: s("fresh-partition-x"),
        budget_from: s("standing-envelope-1"),
        intended_primary_benefit: s("deeper operator sweep"),
        guardrails: vec![s("no regression")],
        rollout_artifact_digest: s("rollout-b16"),
        rollback_artifact_digest: s("rollback-b8"),
        manifest: {
            let mut m = manifest();
            m.total_resource_budget = 10;
            m
        },
    };
    let verified = vec![
        digest16.clone(),
        digest8.clone(),
        s("rollout-b16"),
        s("rollback-b8"),
    ];
    let grant = grant_for(&verified);

    // (a) false benefit.
    let mut a = supported_assessment(&digest16);
    a.benefit_status = BenefitStatus::Unsupported;
    let err =
        promote(&base(), &a, &grant, &verified, true).expect_err("false benefit must not deploy");
    assert!(matches!(err, PromotionRejection::FalseBenefit), "{err:?}");

    // (b) confounded evaluation.
    let mut a = supported_assessment(&digest16);
    a.confounds_present = true;
    let err =
        promote(&base(), &a, &grant, &verified, true).expect_err("confounded must not deploy");
    assert!(
        matches!(err, PromotionRejection::ConfoundedEvaluation),
        "{err:?}"
    );

    // (c) over budget.
    let mut a = supported_assessment(&digest16);
    a.challenger_cost = 999;
    let err =
        promote(&base(), &a, &grant, &verified, true).expect_err("over budget must not deploy");
    assert!(matches!(err, PromotionRejection::OverBudget), "{err:?}");

    // (d) permission-expanding (protected target).
    let mut c = base();
    c.target = s("safety-policy");
    let a = supported_assessment(&digest16);
    let err =
        promote(&c, &a, &grant, &verified, true).expect_err("protected target must not deploy");
    assert!(
        matches!(err, PromotionRejection::PermissionExpanding),
        "{err:?}"
    );
}

#[test]
fn proposal_edge_refuses_unseeded_and_self_budgeted_candidates() {
    use hephaestus::selfimprove::ProposalError;
    let digest8 = sha256_hex(payload(8).as_bytes());
    let digest16 = sha256_hex(payload(16).as_bytes());
    let seeded = ProposalInput {
        target: TARGET.to_string(),
        incumbent_id: s("bound-8"),
        incumbent_digest: digest8,
        challenger_id: s("bound-16"),
        challenger_digest: digest16.clone(),
        supporting_observation_ids: vec![s("t0")],
        fresh_partition_id: s("fresh-partition-x"),
        budget_from: s("standing-envelope-1"),
        intended_primary_benefit: s("deeper sweep"),
        guardrails: vec![s("no regression")],
        rollout_artifact_digest: s("rollout-b16"),
        rollback_artifact_digest: s("rollback-b8"),
        manifest: manifest(),
    };
    // Unseeded: no observations => the proposal is a guess, refused.
    let mut unseeded = seeded.clone();
    unseeded.supporting_observation_ids = vec![];
    let err = propose_from_observations(unseeded).expect_err("an unseeded proposal must refuse");
    assert!(matches!(err, ProposalError::UnseededProposal), "{err:?}");

    // Self-budgeting: budget from the challenger itself, refused.
    let mut self_budgeted = seeded;
    self_budgeted.budget_from = self_budgeted.challenger_id.clone();
    let err = propose_from_observations(self_budgeted).expect_err("self-budgeting must refuse");
    assert!(matches!(err, ProposalError::SelfBudgeting), "{err:?}");
}
