//! Pure replay of operation state from the verified event chain (T-011).

use serde::Serialize;

use crate::contracts::generated::Event;

use super::Receipt;

/// Derived operation state — never stored, always folded from events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationState {
    /// Planned, never dispatched.
    Planned,
    /// Dispatched, no receipt yet.
    Ambiguous,
    /// Receipt with a successful outcome.
    Succeeded,
    /// Receipt with a failed outcome.
    Failed,
    /// Receipt with a timed-out outcome.
    TimedOut,
    /// Cancel requested (and no contradicting receipt).
    Cancelled,
    /// Impossible history (receipt without dispatch, events without plan).
    Corrupt,
}

/// Replay result: per-operation states in ledger order + parsed receipts.
#[derive(Debug, Clone, Serialize)]
pub struct OperationView {
    /// (operation id, derived state) in first-seen ledger order.
    pub states: Vec<(String, OperationState)>,
    /// Operation ids with a durable cancel request.
    pub cancelled: Vec<String>,
    /// Parsed receipts in ledger order (payload read via the store);
    /// unreadable payloads increment `receipt_read_failures` and force the
    /// affected row to the safe Ambiguous side, never to a requeueable state.
    pub receipts: Vec<(String, Receipt)>,
    /// Receipt events whose payload could not be read back.
    pub receipt_read_failures: usize,
    /// Lifecycle event types this fold did not recognize (counted, ignored).
    pub unknown_events: usize,
    /// (operation id, dispatched-event count) in ledger order — attempts.
    pub dispatch_counts: Vec<(String, u32)>,
}

impl OperationView {
    /// Derived state for one operation.
    pub fn state_of(&self, op: &str) -> Option<&OperationState> {
        self.states.iter().find(|(id, _)| id == op).map(|(_, s)| s)
    }

    /// Whether a durable cancel request exists for this operation.
    pub fn cancel_requested(&self, op: &str) -> bool {
        self.cancelled.iter().any(|id| id == op)
    }

    /// Dispatch (attempt) count for one operation.
    pub fn dispatch_count(&self, op: &str) -> u32 {
        self.dispatch_counts
            .iter()
            .find(|(id, _)| id == op)
            .map(|(_, n)| *n)
            .unwrap_or(0)
    }

    /// The parsed receipt for this operation, if one exists.
    pub fn receipt_for(&self, op: &str) -> Option<&Receipt> {
        self.receipts
            .iter()
            .find(|(id, _)| id == op)
            .map(|(_, r)| r)
    }
}

/// The known lifecycle vocabulary.
const KNOWN: &[&str] = &[
    "operation.planned",
    "operation.dispatched",
    "operation.receipt",
    "operation.cancel_requested",
    "operation.output_committed",
    "operation.reconciled",
];

#[derive(Default)]
struct Fold {
    planned: Vec<String>,
    dispatched: Vec<String>,
    receipt_ops: Vec<String>, // receipt EVENT seen (parse-independent)
    receipt_outcomes: Vec<(String, String)>, // (op, parsed outcome name)
    cancels: Vec<String>,
    order: Vec<String>,
    unknown: usize,
    receipt_read_failures: usize,
    parsed_receipts: Vec<(String, Receipt)>,
}

impl Fold {
    fn touch(&mut self, op: &str) {
        if !self.order.iter().any(|o| o == op) {
            self.order.push(op.to_string());
        }
    }

    fn state(&self, op: &str) -> OperationState {
        let planned = self.planned.iter().any(|o| o == op);
        let dispatched = self.dispatched.iter().any(|o| o == op);
        let receipt_seen = self.receipt_ops.iter().any(|o| o == op);
        let receipt = self.receipt_outcomes.iter().find(|(o, _)| o == op);
        // Impossible orders first (fail-closed, never repaired) — receipt
        // EVENT presence counts even when its payload is unreadable (an
        // unreadable receipt must never downgrade to a requeueable state).
        if receipt_seen && !dispatched {
            return OperationState::Corrupt;
        }
        if !planned && (dispatched || receipt_seen) {
            return OperationState::Corrupt;
        }
        if receipt_seen && receipt.is_none() {
            return OperationState::Ambiguous;
        }
        if let Some((_, outcome)) = receipt {
            return match outcome.as_str() {
                "Succeeded" => OperationState::Succeeded,
                "Failed" => OperationState::Failed,
                "TimedOut" => OperationState::TimedOut,
                "Cancelled" => OperationState::Cancelled,
                // An executor-declared unknown effect rests in the same
                // recovery state as a missing receipt: Ambiguous (never a
                // success, never blindly retried — grill Q6).
                "AmbiguousEffect" => OperationState::Ambiguous,
                _ => OperationState::Corrupt,
            };
        }
        // Dispatched-but-receiptless stays Ambiguous EVEN IF a cancel was
        // later requested: an uncertain effect must not hide behind
        // Cancelled (recovery would cancel instead of reconciling).
        if dispatched {
            return OperationState::Ambiguous;
        }
        if self.cancels.iter().any(|o| o == op) {
            return OperationState::Cancelled;
        }
        OperationState::Planned
    }
}

/// Fold lifecycle events (ledger order) into a view; `load_receipt` maps a
/// receipt event to its parsed receipt via the store (None = no/empty
/// payload). Internal so unit tests can hand-build events without a chain.
pub(crate) fn fold(
    events: &[Event],
    load_receipt: impl Fn(&Event) -> Option<Receipt>,
) -> OperationView {
    let mut fold = Fold::default();
    for event in events {
        if !KNOWN.contains(&event.event_type.as_str()) {
            fold.unknown += 1;
            continue;
        }
        let op = event.operation_id.as_str();
        if op.is_empty() {
            continue;
        }
        fold.touch(op);
        match event.event_type.as_str() {
            "operation.planned" => fold.planned.push(op.to_string()),
            "operation.dispatched" => fold.dispatched.push(op.to_string()),
            "operation.cancel_requested" => fold.cancels.push(op.to_string()),
            "operation.receipt" => {
                fold.receipt_ops.push(op.to_string());
                if let Some(receipt) = load_receipt(event) {
                    fold.receipt_outcomes
                        .push((op.to_string(), receipt.outcome.clone()));
                    fold.parsed_receipts.push((op.to_string(), receipt));
                } else {
                    fold.receipt_read_failures += 1;
                }
            }
            _ => {} // output_committed / reconciled: recorded, not state-shaping yet
        }
    }
    let receipts = std::mem::take(&mut fold.parsed_receipts);
    let states = fold
        .order
        .iter()
        .map(|op| (op.clone(), fold.state(op)))
        .collect();
    let mut dispatch_counts: Vec<(String, u32)> = Vec::new();
    for op in &fold.order {
        let n = fold.dispatched.iter().filter(|d| *d == op).count() as u32;
        dispatch_counts.push((op.clone(), n));
    }
    OperationView {
        states,
        cancelled: fold.cancels,
        unknown_events: fold.unknown,
        dispatch_counts,
        receipts,
        receipt_read_failures: fold.receipt_read_failures,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::generated::{
        EventKind, MissionDataOrigin, Provenance, RecordRef, SchemaVersion, TrustOrigin,
    };

    fn event(op: &str, kind: &str) -> Event {
        Event {
            id: format!("EVT-{op}-1"),
            schema_version: SchemaVersion::V1_2,
            record_version: 1,
            created_at: "2026-10-01T00:00:00Z".to_string(),
            data_origin: MissionDataOrigin::Live,
            provenance: Provenance {
                actor_id: "t".to_string(),
                method: "t".to_string(),
                input_refs: vec![],
                artifact_hashes: vec![],
                trust_origin: TrustOrigin::Owner,
            },
            kind: EventKind::Event,
            mission_ref: RecordRef {
                id: "OPS".to_string(),
                version: 1,
            },
            sequence: 1,
            event_type: kind.to_string(),
            subject_ref: RecordRef {
                id: op.to_string(),
                version: 1,
            },
            operation_id: op.to_string(),
            policy_version: "t011-ops-1".to_string(),
            payload_sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                .to_string(),
            previous_event_sha256: None,
        }
    }

    #[test]
    fn fold_counts_unknown_lifecycle_types_without_panicking() {
        let events = vec![
            event("OP-0-a", "operation.planned"),
            event("OP-0-a", "something.from.the.future"),
        ];
        let view = fold(&events, |_| None);
        assert_eq!(view.unknown_events, 1);
        assert_eq!(format!("{:?}", view.state_of("OP-0-a").unwrap()), "Planned");
    }

    #[test]
    fn unreadable_receipt_payload_never_downgrades_to_requeueable() {
        // Pass-2 gap: receipt EVENT present, payload unreadable must never
        // become Planned (requeueable).
        let dispatched_unreadable = vec![
            event("OP-0-u", "operation.planned"),
            event("OP-0-u", "operation.dispatched"),
            event("OP-0-u", "operation.receipt"),
        ];
        let view = fold(&dispatched_unreadable, |_| None);
        assert_eq!(
            format!("{:?}", view.state_of("OP-0-u").unwrap()),
            "Ambiguous"
        );
        assert_eq!(view.receipt_read_failures, 1);

        let corrupt_unreadable = vec![
            event("OP-0-v", "operation.planned"),
            event("OP-0-v", "operation.receipt"),
        ];
        let view = fold(&corrupt_unreadable, |_| None);
        assert_eq!(
            format!("{:?}", view.state_of("OP-0-v").unwrap()),
            "Corrupt",
            "receipt presence beats payload trouble"
        );
    }

    #[test]
    fn corrupt_order_beats_every_other_interpretation() {
        // Receipt without dispatch: impossible regardless of plan/cancel.
        let events = vec![
            event("OP-0-x", "operation.planned"),
            event("OP-0-x", "operation.cancel_requested"),
            event("OP-0-x", "operation.receipt"),
        ];
        let view = fold(&events, |_| {
            Some(Receipt {
                outcome: "Succeeded".to_string(),
                cost: crate::contracts::generated::Money {
                    currency: "USD".to_string(),
                    minor_units: 0,
                },
                wall_ms: 1,
                reason: None,
                attempt: 1,
                executor: "t".to_string(),
                artifacts: vec![],
            })
        });
        assert_eq!(format!("{:?}", view.state_of("OP-0-x").unwrap()), "Corrupt");
    }
}
