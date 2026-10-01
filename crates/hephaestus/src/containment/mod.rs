//! Containment race guard (T-038, R-111).
//!
//! Monitor containment violations, stop safely, and preserve events
//! during revocation and cancellation races: one append-only event
//! ledger for both sequences; a revoked grant grounds no further
//! authorized events; completed work is never fabricated-undone; a
//! violation is recorded as evidence and never un-revokes.

pub mod record;

pub use record::{RaceEvent, RaceOutcome};

use serde::{Deserialize, Serialize};

/// Guard errors — each named.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardError {
    /// An event references an unknown dispatch.
    UnknownDispatch,
    /// An authorized event was attempted after the grant was revoked:
    /// refused (no resurrection of revoked authorization).
    AuthorizedEventAfterRevocation,
}

/// Decide the race outcome for one dispatch from the append-only event
/// sequence (order IS the decision input): if the grant was revoked
/// before the dispatch completed, the outcome is StoppedSafely;
/// otherwise CompletedBeforeCancel.
pub fn race_outcome(events: &[RaceEvent], dispatch_id: &str) -> Result<RaceOutcome, GuardError> {
    let mut started = false;
    for e in events {
        match e {
            RaceEvent::DispatchStarted { dispatch_id: d } if d == dispatch_id => {
                started = true;
            }
            RaceEvent::GrantRevoked { .. } if started => {
                return Ok(RaceOutcome::StoppedSafely);
            }
            RaceEvent::DispatchCompleted { dispatch_id: d } if d == dispatch_id => {
                return Ok(RaceOutcome::CompletedBeforeCancel);
            }
            _ => {}
        }
    }
    if started {
        // Started, neither revoked nor completed: still in flight, and
        // with no revocation on record the dispatch continues.
        Ok(RaceOutcome::CompletedBeforeCancel)
    } else {
        Err(GuardError::UnknownDispatch)
    }
}

/// Append a containment violation to the ledger. The violation is
/// EVIDENCE: appending it must not and does not alter any grant state
/// (the grant stays revoked; no un-revoke path exists).
pub fn record_violation(
    ledger: &mut Vec<RaceEvent>,
    dispatch_id: &str,
    detail: &str,
) -> Result<(), GuardError> {
    let known = ledger
        .iter()
        .any(|e| matches!(e, RaceEvent::DispatchStarted { dispatch_id: d } if d == dispatch_id));
    if !known {
        return Err(GuardError::UnknownDispatch);
    }
    ledger.push(RaceEvent::Violation {
        dispatch_id: dispatch_id.to_string(),
        detail: detail.to_string(),
    });
    Ok(())
}

/// Can this dispatch still emit AUTHORIZED events? After a GrantRevoked
/// event in the ledger: no — for any dispatch. This is the safe-stop
/// gate (R-111); authorization is never resurrected.
pub fn may_emit_authorized(ledger: &[RaceEvent], dispatch_id: &str) -> Result<bool, GuardError> {
    let known = ledger
        .iter()
        .any(|e| matches!(e, RaceEvent::DispatchStarted { dispatch_id: d } if d == dispatch_id));
    if !known {
        return Err(GuardError::UnknownDispatch);
    }
    Ok(!ledger
        .iter()
        .any(|e| matches!(e, RaceEvent::GrantRevoked { .. })))
}
