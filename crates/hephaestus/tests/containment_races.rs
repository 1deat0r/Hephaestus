//! Containment race guard (T-038, R-111).
//!
//! Integration tests at the public seam: `race_outcome`,
//! `record_violation`, `may_emit_authorized`.

use hephaestus::containment::record::RaceEvent;
use hephaestus::containment::{RaceOutcome, race_outcome, record_violation};

fn s(v: &str) -> String {
    v.to_string()
}

fn started(id: &str) -> RaceEvent {
    RaceEvent::DispatchStarted { dispatch_id: s(id) }
}

#[test]
fn revocation_stops_in_flight_dispatch_safely() {
    // Revocation arrives while the dispatch is in flight.
    let events = vec![started("d1"), RaceEvent::GrantRevoked { grant_id: s("g1") }];
    assert_eq!(race_outcome(&events, "d1"), Ok(RaceOutcome::StoppedSafely));
    // After revocation, no further AUTHORIZED events may be emitted
    // (safe stop; authorization never resurrected).
    assert_eq!(
        hephaestus::containment::may_emit_authorized(&events, "d1"),
        Ok(false)
    );
}

#[test]
fn completed_before_cancel_is_not_fabricated_undone() {
    // The dispatch finished before cancellation reached it: the
    // completed work stands (no retroactive undo).
    let events = vec![
        started("d2"),
        RaceEvent::DispatchCompleted {
            dispatch_id: s("d2"),
        },
        RaceEvent::CancelRequested {
            dispatch_id: s("d2"),
        },
    ];
    assert_eq!(
        race_outcome(&events, "d2"),
        Ok(RaceOutcome::CompletedBeforeCancel)
    );
}

#[test]
fn events_from_both_sequences_preserved_in_order() {
    // Grant lifecycle and dispatch lifecycle interleave in ONE
    // append-only ledger; nothing erased.
    let mut ledger = vec![
        started("d3"),
        RaceEvent::DispatchStarted {
            dispatch_id: s("d4"),
        },
    ];
    ledger.push(RaceEvent::GrantRevoked { grant_id: s("g1") });
    // Violation recorded during the race: appended, evidence only.
    record_violation(&mut ledger, "d3", "descriptor escaped sandbox").expect("recorded");
    // All events still present, in order (append-only preservation).
    assert_eq!(ledger.len(), 4);
    assert!(matches!(ledger[3], RaceEvent::Violation { .. }));
    // The violation never un-revokes: authorized emission stays denied
    // for BOTH dispatches (revocation is global to the grant).
    assert_eq!(
        hephaestus::containment::may_emit_authorized(&ledger, "d3"),
        Ok(false)
    );
    assert_eq!(
        hephaestus::containment::may_emit_authorized(&ledger, "d4"),
        Ok(false)
    );
}

#[test]
fn unknown_dispatch_refused() {
    let events = vec![started("d1")];
    assert_eq!(
        race_outcome(&events, "unknown"),
        Err(hephaestus::containment::GuardError::UnknownDispatch)
    );
    let mut ledger = events.clone();
    assert_eq!(
        record_violation(&mut ledger, "unknown", "x"),
        Err(hephaestus::containment::GuardError::UnknownDispatch)
    );
}

#[test]
fn twin_run_byte_identical() {
    let mut a = vec![started("d1"), RaceEvent::GrantRevoked { grant_id: s("g1") }];
    record_violation(&mut a, "d1", "x").unwrap();
    let mut b = vec![started("d1"), RaceEvent::GrantRevoked { grant_id: s("g1") }];
    record_violation(&mut b, "d1", "x").unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
