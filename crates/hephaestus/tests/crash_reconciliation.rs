//! T-063 / R-118/AT-118 crash reconciliation + wired canary (M3 clause 3.5b):
//! restart and crash recovery reconstruct the champion and reconcile
//! interrupted deployments; a breached guardrail stops the rollout via
//! the wired check — never a silent empty ledger, never a duplicated
//! deployment.

use hephaestus::selfimprove::record::{
    Deployment, GuardrailIndicators, ImprovementLedger, LedgerEntry,
};
use hephaestus::selfimprove::{
    RecoverError, RecoveryAction, begin_deployment, canonical_entry_id,
    check_deployment_guardrails, commit_deployment, recover,
};

fn s(v: &str) -> String {
    v.to_string()
}

fn entry() -> LedgerEntry {
    LedgerEntry {
        challenger_id: s("bound-16"),
        challenger_digest: s("d16"),
        incumbent_id: s("bound-8"),
        incumbent_digest: s("d8"),
        outcome: s("deployed"),
        reason: s("supported benefit, guardrails passing"),
        observations: s("fixture crash-window observations"),
        target: s("discovery.generation_bound"),
        fresh_partition_id: s("fresh-partition-x"),
    }
}

fn deployment() -> Deployment {
    Deployment {
        challenger_id: s("bound-16"),
        challenger_digest: s("d16"),
        incumbent_id: s("bound-8"),
        incumbent_digest: s("d8"),
        grant_scope: vec![s("discovery.generation_bound")],
        assessment_payload_digest: s("d16"),
        observed_scope: s("canary-bounded"),
    }
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "crash-recon-{}-{}-{tag}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

#[test]
fn corrupt_ledger_refuses_recovery_instead_of_silently_empting() {
    let dir = temp_dir("corrupt");
    let path = dir.join("improvement-ledger.json");
    std::fs::write(&path, r#"{"entries":[{"challenger_id":"bound-16","#).expect("torn write");
    let err = recover(&dir).expect_err("a corrupt ledger must refuse, never return empty");
    assert!(matches!(err, RecoverError::CorruptLedger(_)), "{err:?}");
    // The typed parse path is fail-closed too (R-116: restart loses nothing).
    let err = ImprovementLedger::from_json(r#"{"entries":"nope"#)
        .expect_err("from_json refuses corrupt input");
    assert!(matches!(err, RecoverError::CorruptLedger(_)), "{err:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recovery_reconstructs_state_and_notes_a_stale_temp_file() {
    let dir = temp_dir("reconstruct");
    let mut ledger = ImprovementLedger::default();
    let e = entry();
    ledger.append(e.clone());
    ledger
        .persist(&dir.join("improvement-ledger.json"))
        .expect("persist");
    // Simulate a crash mid-persist: a leftover temp file must be noted
    // and ignored, never treated as state.
    std::fs::write(dir.join("improvement-ledger.json.tmp"), b"{partial").expect("stale tmp");
    let (recovered, actions) = recover(&dir).expect("main intact");
    assert_eq!(recovered.entries().len(), 1, "champion state reconstructed");
    assert_eq!(recovered.entries()[0], e);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, RecoveryAction::StaleTempIgnored)),
        "stale temp noted: {actions:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_interrupted_deployment_reconciles_exactly_once() {
    let dir = temp_dir("wal");
    let e = entry();

    // Window 1: decision durable, append never happened (crash before
    // commit). Recovery reports a replayable deployment; completing it
    // appends exactly once.
    begin_deployment(&dir, &e).expect("pending written");
    let (mut recovered, actions) = recover(&dir).expect("recover with pending");
    assert_eq!(recovered.entries().len(), 0, "nothing committed yet");
    let replay = actions
        .iter()
        .find_map(|a| match a {
            RecoveryAction::DeploymentInterrupted { entry } => Some(entry.clone()),
            _ => None,
        })
        .expect("interrupted deployment reported for replay");
    assert_eq!(canonical_entry_id(&replay), canonical_entry_id(&e));
    commit_deployment(&dir, &mut recovered, replay).expect("replayed");
    assert_eq!(recovered.entries().len(), 1, "committed exactly once");

    // Window 2: append landed, pending cleanup did not (crash between
    // commit and pending removal). Replay must DEDUP, never duplicate.
    begin_deployment(&dir, &e).expect("pending again");
    let (mut again, actions) = recover(&dir).expect("recover stale pending");
    assert_eq!(again.entries().len(), 1, "commit survived the crash");
    let replay = actions
        .iter()
        .find_map(|a| match a {
            RecoveryAction::DeploymentInterrupted { entry } => Some(entry.clone()),
            _ => None,
        })
        .expect("stale pending still reported");
    commit_deployment(&dir, &mut again, replay).expect("second commit dedups");
    assert_eq!(again.entries().len(), 1, "no duplicate entry");

    // Malformed pending: reconciled LOUDLY (discarded with the action
    // retained), main state untouched.
    std::fs::write(dir.join("pending.json"), b"{torn").expect("torn pending");
    let (final_ledger, actions) = recover(&dir).expect("recover torn pending");
    assert_eq!(final_ledger.entries().len(), 1, "main untouched");
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, RecoveryAction::PendingDiscarded { .. })),
        "torn pending discarded loudly: {actions:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn canary_guardrail_breach_names_the_indicator_and_stops_the_rollout() {
    // Predeclared indicators: observed vs limit. A breach names itself
    // (the monitor step the demo's regression path runs through).
    let ok = GuardrailIndicators {
        indicators: vec![
            hephaestus::selfimprove::Indicator {
                name: s("mechanisms_found"),
                kind: hephaestus::selfimprove::BoundKind::AtLeast,
                observed: 12.0,
                limit: 8.0,
            },
            hephaestus::selfimprove::Indicator {
                name: s("latency_ms"),
                kind: hephaestus::selfimprove::BoundKind::AtMost,
                observed: 90.0,
                limit: 100.0,
            },
        ],
    };
    check_deployment_guardrails(&deployment(), &ok).expect("within limits");

    let breached = GuardrailIndicators {
        indicators: vec![hephaestus::selfimprove::Indicator {
            name: s("latency_ms"),
            kind: hephaestus::selfimprove::BoundKind::AtMost,
            observed: 250.0,
            limit: 100.0,
        }],
    };
    let err = check_deployment_guardrails(&deployment(), &breached)
        .expect_err("a breached guardrail stops the rollout");
    assert!(
        err.to_string().contains("latency_ms"),
        "violation names the indicator: {err}"
    );
}
