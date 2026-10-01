//! Durable operations, effect receipts, replay, and recovery (T-011).
//!
//! The execution-side counterpart of T-006's history: operations are
//! recorded **before** their effects (MASTER_SPEC:369), receipts are
//! content-addressed, recovery replays the verified chain, and fault
//! injection is proven by reopening from disk.
//!
//! Limitations (recorded honestly): single in-process writer
//! (MASTER_SPEC:379), durability = `sync_data` on the event ledger (same
//! class as T-006), and `Ambiguous` resolution against *real* externals
//! needs the future reconciliation layer — until then unresolved budget
//! entries are the resting state (never a blind retry, MASTER_SPEC:371).

mod executor;
mod recorder;
mod recover;
mod replay;

pub use executor::RecordingExecutor;
pub use recorder::{OperationRecorder, RecorderError};
pub use recover::{ApplyError, RecoveryPlan, RetryPosture, apply, recover};
pub use replay::{OperationState, OperationView};

use crate::contracts::generated::Money;

/// One durable effect receipt (grill Q7). Serialized JSON is committed to
/// the artifact store; its sha256 rides the ledger event.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Receipt {
    /// Outcome class: `Succeeded` | `Failed` | `TimedOut` | `Cancelled` |
    /// `AmbiguousEffect`.
    pub outcome: String,
    /// Reported cost (integer minor units).
    pub cost: Money,
    /// Supervised wall time in milliseconds.
    pub wall_ms: u128,
    /// Failure/ambiguity reason, if any.
    pub reason: Option<String>,
    /// Attempt number this receipt belongs to.
    pub attempt: u32,
    /// Opaque executor tag (e.g. "sandbox", "spy").
    pub executor: String,
    /// Content-addressed artifact digests produced by the effect.
    pub artifacts: Vec<String>,
}

/// Policy version stamped on recorder events (single-writer M1).
pub(crate) const OPS_POLICY: &str = "t011-ops-1";
/// sha256 of the empty payload: events with no receipt use this as their
/// mandatory `payload_sha256` (the contract field is required).
pub(crate) const EMPTY_SHA256: &str =
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
/// Mission id under which recorder events chain.
pub(crate) const OPS_MISSION: &str = "OPS";

/// Minimal RFC 3339 (UTC, second precision) without the `time` formatting
/// feature — keeps dependencies untouched (grill Q11).
pub(crate) fn rfc3339_utc(dt: OffsetDateTime) -> String {
    let u = dt.to_offset(time::UtcOffset::UTC);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        u.year(),
        u.month() as u8,
        u.day(),
        u.hour(),
        u.minute(),
        u.second()
    )
}

use time::OffsetDateTime;

/// Current UTC timestamp for event envelopes.
pub(crate) fn now_rfc3339() -> String {
    rfc3339_utc(OffsetDateTime::now_utc())
}

/// Money helper for zero-cost receipts.
pub fn zero_cost(currency: &str) -> Money {
    Money {
        currency: currency.to_string(),
        minor_units: 0,
    }
}
