//! Containment records (T-038, R-111).

use serde::{Deserialize, Serialize};

/// An event in a revocation/cancellation race. Events are append-only
/// and ordered — both sequences (grant lifecycle + dispatch lifecycle)
/// land in one ledger, nothing erased (R-111).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaceEvent {
    /// A dispatch started under a then-valid grant.
    DispatchStarted { dispatch_id: String },
    /// The grant was revoked while dispatches may be in flight.
    GrantRevoked { grant_id: String },
    /// Cancellation was requested for a dispatch.
    CancelRequested { dispatch_id: String },
    /// A dispatch completed before cancellation reached it.
    DispatchCompleted { dispatch_id: String },
    /// In-flight work stopped safely after revocation: no further
    /// authorized side effects.
    StoppedSafely { dispatch_id: String },
    /// A containment violation observed during the race — evidence
    /// only; never un-revokes the grant.
    Violation { dispatch_id: String, detail: String },
}

/// The race outcome for one dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaceOutcome {
    /// Revocation arrived while the dispatch was in flight: work
    /// stopped safely, no further authorized events.
    StoppedSafely,
    /// The dispatch completed before cancellation reached it: finished
    /// work is NOT fabricated-undone.
    CompletedBeforeCancel,
}
