//! Recovery planning and application (T-011 grill Q6).

use crate::budget::{BudgetError, BudgetLedger};

use super::{OperationView, Receipt};

/// Retry posture for one operation, supplied by the caller (the DAG owns
/// retry policy — the module never invents it).
#[derive(Debug, Clone, Copy)]
pub struct RetryPosture {
    /// Whether automatic retry is safe for this operation at all
    /// (MASTER_SPEC:371 — false means never requeue, only reconcile).
    pub retryable: bool,
    /// Total attempts allowed.
    pub max_attempts: u32,
}

/// What recovery decides — pure output of [`recover`]; applying it is an
/// explicit second step (grill Q6).
#[derive(Debug, Clone, Default)]
pub struct RecoveryPlan {
    /// Safe to run again (Planned, or Ambiguous within retry budget and
    /// with no durable cancel intent).
    pub requeue: Vec<String>,
    /// Needs budget reconciliation (ambiguous outside retry budget, or
    /// carrying durable cancel intent — an in-flight effect is reconciled,
    /// never requeued: MASTER_SPEC:375).
    pub unresolved: Vec<String>,
    /// Receipt-backed terminal states (Succeeded/Failed/TimedOut).
    pub terminal: Vec<String>,
    /// Durable cancel intent with no dispatch.
    pub cancelled: Vec<String>,
    /// Impossible histories — surfaced, never auto-requeued.
    pub corrupt: Vec<String>,
}

impl RecoveryPlan {
    /// Total operations classified (every op appears exactly once).
    pub fn total(&self) -> usize {
        self.requeue.len()
            + self.unresolved.len()
            + self.terminal.len()
            + self.cancelled.len()
            + self.corrupt.len()
    }
}

/// AT-057's rule table over a replay view. Pure (grill Q6): budget effects
/// happen only in [`apply`].
pub fn recover(view: &OperationView, posture_for: impl Fn(&str) -> RetryPosture) -> RecoveryPlan {
    let mut plan = RecoveryPlan::default();
    for (op, state) in &view.states {
        let posture = posture_for(op);
        let attempts = view.dispatch_count(op);
        match state {
            super::OperationState::Planned => plan.requeue.push(op.clone()),
            super::OperationState::Cancelled => plan.cancelled.push(op.clone()),
            super::OperationState::Corrupt => plan.corrupt.push(op.clone()),
            super::OperationState::Succeeded
            | super::OperationState::Failed
            | super::OperationState::TimedOut => plan.terminal.push(op.clone()),
            // Ambiguous (incl. executor-declared unknown effects): requeue
            // ONLY when retry is permitted and budget remains — never a
            // blind retry (MASTER_SPEC:371, AT-057). Durable cancel intent
            // outranks retry permission: cancellation prevents new
            // reservations (MASTER_SPEC:375), so a dispatched op carrying
            // cancel intent is reconciled (unresolved), never requeued and
            // never hidden as Cancelled (AT-056/R-056).
            super::OperationState::Ambiguous => {
                let retry_permitted = posture.retryable && attempts < posture.max_attempts;
                if retry_permitted && !view.cancel_requested(op) {
                    plan.requeue.push(op.clone());
                } else {
                    plan.unresolved.push(op.clone());
                }
            }
        }
    }
    plan
}

/// Why an apply step failed.
#[derive(Debug)]
pub enum ApplyError {
    /// A budget operation failed for a reason other than "already recorded".
    Budget(BudgetError),
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplyError::Budget(e) => write!(f, "recovery budget apply: {e}"),
        }
    }
}

impl std::error::Error for ApplyError {}

/// Execute a plan's budget effects: release any hold still carried under
/// the op id (post-crash ledgers normally have none — NotFound is fine),
/// then record each unresolved op **exactly once** (a repeat run sees
/// `DuplicateCharge` and records nothing — AT-056). Returns the ops newly
/// recorded as unresolved.
pub fn apply(
    plan: &RecoveryPlan,
    view: &OperationView,
    budget: &mut BudgetLedger,
) -> Result<Vec<String>, ApplyError> {
    let mut recorded = Vec::new();
    for op in &plan.unresolved {
        // Release a pre-crash hold if this is a live budget (idempotent).
        match budget.release(op) {
            Ok(()) => {}
            Err(BudgetError::NotFound { .. }) => {}
            Err(e) => return Err(ApplyError::Budget(e)),
        }
        let (amount, reason) = match view.receipt_for(op) {
            Some(Receipt { cost, reason, .. }) if cost.minor_units > 0 => (
                Some(cost.clone()),
                reason
                    .clone()
                    .unwrap_or_else(|| "ambiguous effect (receipt)".to_string()),
            ),
            Some(Receipt { reason, .. }) => (
                None,
                reason
                    .clone()
                    .unwrap_or_else(|| "ambiguous effect (receipt, zero cost)".to_string()),
            ),
            None => (None, "no receipt after dispatch".to_string()),
        };
        match budget.mark_unresolved(op, amount, &reason) {
            Ok(()) => recorded.push(op.clone()),
            Err(BudgetError::DuplicateCharge { .. }) => {
                // Already reconciled by a previous apply — idempotent.
            }
            Err(e) => return Err(ApplyError::Budget(e)),
        }
    }
    Ok(recorded)
}
