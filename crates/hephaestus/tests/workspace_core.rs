//! Workspace core (T-025, R-067/AT-067 (progress reads persisted
//! state — unknown stays unknown), R-068/AT-068 (event-cursor recovery
//! resumes a session), R-069/AT-069 (one authorization gate for the
//! local and the gateway path), MASTER_SPEC §2:42).
//!
//! Integration tests at the public seam: `progress`, `compare`,
//! `drill_down`, `command`, `gateway_event`.

use hephaestus::workspace::record::{
    AuthToken, ControlCommand, ControlError, GatewayOutcome, HypothesisRecord, PersistedState,
};
use hephaestus::workspace::{ControlPolicy, Workspace};

fn s(v: &str) -> String {
    v.to_string()
}

fn state() -> PersistedState {
    PersistedState {
        entity_counts: vec![(s("opportunities"), 12), (s("hypotheses"), 7)],
        last_event_id: 41,
        running: true,
        missions_unknown: 2,
    }
}

fn hypotheses() -> Vec<HypothesisRecord> {
    vec![
        HypothesisRecord {
            id: s("h-1"),
            version: s("1.0.0"),
            state: s("test-ready"),
            claim_ids: vec![s("C1"), s("C2")],
            has_discriminator: true,
            evidence_ids: vec![s("e-1"), s("e-2")],
        },
        HypothesisRecord {
            id: s("h-2"),
            version: s("1.1.0"),
            state: s("exploratory"),
            claim_ids: vec![s("C2"), s("C3")],
            has_discriminator: false,
            evidence_ids: vec![s("e-2"), s("e-3")],
        },
    ]
}

fn policy() -> ControlPolicy {
    ControlPolicy {
        authorized_digests: vec![s("token-alice")],
        known_missions: vec![s("m-1")],
    }
}

fn workspace() -> Workspace {
    Workspace::open(
        state(),
        hypotheses(),
        policy(),
        vec![(s("e-1"), vec![s("o-1"), s("m-1")])],
    )
}

// ---- Ticket 01: truthful views ----

#[test]
fn progress_reads_persisted_state_unknown_stays_unknown() {
    let ws = workspace();
    let p = ws.progress();
    // R-067/AT-067: a stalled mission with no active workers shows the
    // persisted counts only — the view invents no activity, no
    // percentage, and no completed work.
    // R-067: counts FROM persisted state.
    assert_eq!(
        p.entity_counts,
        vec![(s("opportunities"), 12), (s("hypotheses"), 7)]
    );
    assert_eq!(p.last_event_id, 41);
    assert!(p.running);
    // Unknown stays unknown: no invented percentage, no narrative.
    assert_eq!(p.missions_unknown, 2);
}

#[test]
fn compare_and_drill_down() {
    let ws = workspace();
    let view = ws.compare("h-1", "h-2").expect("both known");
    assert!(!view.same_version);
    assert!(!view.same_state);
    assert_eq!(view.claims_only_in_a, vec![s("C1")]);
    assert_eq!(view.claims_only_in_b, vec![s("C3")]);
    assert_eq!(view.shared_evidence, vec![s("e-2")]);
    // Drill-down: lineage + record.
    let ev = ws.drill_down("e-1").expect("known record");
    assert_eq!(ev.record_id, "e-1");
    assert_eq!(ev.lineage_ids, vec![s("o-1"), s("m-1")]);
    assert!(
        ws.drill_down("e-99").is_none(),
        "unknown drill-down is None"
    );
}

// ---- Ticket 02: controls + gateway ----

// R-069/AT-069: an operation refused on the local path is refused the
// same way through the gateway — the remote client cannot bypass the
// local authorization gate.
#[test]
fn shared_authorization_gates_both_paths_identically() {
    let mut ws = workspace();
    let good = AuthToken(s("token-alice"));
    let bad = AuthToken(s("token-eve"));
    // Local path: unauthorized refused.
    assert_eq!(
        ws.command(&bad, &ControlCommand::Pause),
        Err(ControlError::Unauthorized)
    );
    // Gateway path: the SAME token refused the SAME way (R-069).
    assert_eq!(
        ws.gateway_event(&bad, 41, &ControlCommand::Pause),
        Err(ControlError::Unauthorized)
    );
    // Authorized pause works on the local path.
    let ack = ws.command(&good, &ControlCommand::Pause).expect("applied");
    assert!(!ws.progress().running);
    assert_eq!(ack.new_event_id, 42);
}

// R-068/AT-068: pause, resume, cancellation, and steering are typed
// controls; each refusal names its own state error.
#[test]
fn intervention_state_rules() {
    let mut ws = workspace();
    let tok = AuthToken(s("token-alice"));
    // Pause while running: ok. Pause again: InvalidState.
    ws.command(&tok, &ControlCommand::Pause).unwrap();
    assert_eq!(
        ws.command(&tok, &ControlCommand::Pause),
        Err(ControlError::InvalidState)
    );
    // Resume.
    ws.command(&tok, &ControlCommand::Resume).unwrap();
    assert!(ws.progress().running);
    // Unknown mission refused.
    assert_eq!(
        ws.command(&tok, &ControlCommand::CancelMission(s("m-99"))),
        Err(ControlError::UnknownMission)
    );
    // Steering a known mission ok.
    ws.command(
        &tok,
        &ControlCommand::SteerMission(s("m-1"), s("prioritize cost")),
    )
    .unwrap();
}

// R-068/AT-068: a client that disconnects, changes priorities, and
// reconnects resumes from its cursor — duplicates and stale cursors get
// replay recommendations, and the reopened session reads the persisted
// event id.
#[test]
fn event_cursor_recovery() {
    let mut ws = workspace();
    let tok = AuthToken(s("token-alice"));
    // Fresh event at cursor 41: applied, cursor advances.
    match ws.gateway_event(&tok, 41, &ControlCommand::Resume) {
        Err(ControlError::InvalidState) => { /* already running: expected */ }
        other => panic!("unexpected: {other:?}"),
    }
    ws.command(&tok, &ControlCommand::Pause).unwrap(); // now paused, event 42
    // Duplicate (client replays 41): replay recommendation, cursor 42.
    match ws.gateway_event(&tok, 41, &ControlCommand::Resume) {
        Ok(GatewayOutcome::Duplicate(cur)) => assert_eq!(cur, 42),
        other => panic!("unexpected: {other:?}"),
    }
    // Stale/ahead cursor (client at 45): expect replay from 43.
    match ws.gateway_event(&tok, 45, &ControlCommand::Resume) {
        Ok(GatewayOutcome::StaleCursor { expect_from }) => assert_eq!(expect_from, 43),
        other => panic!("unexpected: {other:?}"),
    }
    // Resume = re-open from persisted state.
    let resumed = Workspace::open(ws.progress_state(), hypotheses(), policy(), vec![]);
    assert_eq!(resumed.progress().last_event_id, 42);
    assert!(!resumed.progress().running);
}

#[test]
fn twin_run_byte_identical() {
    let a = workspace();
    let b = workspace();
    assert_eq!(
        serde_json::to_string(&a.progress()).unwrap(),
        serde_json::to_string(&b.progress()).unwrap()
    );
}
