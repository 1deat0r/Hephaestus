//! `RecordingExecutor` — capability-contract decorator (T-011 grill Q10).
//!
//! No scheduler changes: this wrapper appends `operation.dispatched`
//! **before** delegating (append-before-effect, MASTER_SPEC:369), then
//! commits and records the receipt after. Unplanned tasks are refused
//! before any event is written.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::scheduler::{Task, TaskExecutor, TaskOutcome};

use super::{OperationRecorder, Receipt, RecorderError};

/// Wraps a task executor with durable operation recording.
pub struct RecordingExecutor<E> {
    inner: E,
    recorder: Arc<Mutex<OperationRecorder>>,
    planned: HashMap<String, String>,
    attempts: Arc<Mutex<HashMap<String, u32>>>,
}

impl<E: TaskExecutor> RecordingExecutor<E> {
    /// Wrap `inner`; the recorder is shared via `Arc` so callers can replay
    /// after the run (single writer discipline: lock spans each append).
    pub fn new(inner: E, recorder: OperationRecorder) -> Self {
        RecordingExecutor {
            inner,
            recorder: Arc::new(Mutex::new(recorder)),
            planned: HashMap::new(),
            attempts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Shared handle for replay/inspection after the run.
    pub fn recorder(&self) -> Arc<Mutex<OperationRecorder>> {
        Arc::clone(&self.recorder)
    }

    /// Delegating read: every ledger event of one lifecycle type.
    pub fn events_of_type(&self, event_type: &str) -> Vec<crate::contracts::generated::Event> {
        self.recorder
            .lock()
            .expect("recorder lock")
            .events_of_type(event_type)
    }

    /// Delegating read: the receipt event for one operation.
    pub fn receipt_events_for(&self, op: &str) -> Option<crate::contracts::generated::Event> {
        self.recorder
            .lock()
            .expect("recorder lock")
            .receipt_events_for(op)
    }

    /// Durably plan an operation for one task id (call before `run`).
    pub fn plan_task(&mut self, task_id: &str) -> Result<String, RecorderError> {
        let mut rec = self.recorder.lock().expect("recorder lock");
        let op = rec.plan(task_id)?;
        self.planned.insert(task_id.to_string(), op.clone());
        Ok(op)
    }

    fn receipt_for(
        task: &Task,
        outcome: &TaskOutcome,
        wall_ms: u128,
        attempt: u32,
        executor: String,
    ) -> Receipt {
        let (outcome_name, cost, reason) = match outcome {
            TaskOutcome::Succeeded { cost } => ("Succeeded", cost.clone(), None),
            TaskOutcome::Failed { reason } => ("Failed", task.cost.clone(), Some(reason.clone())),
            TaskOutcome::TimedOut => ("TimedOut", task.cost.clone(), None),
            TaskOutcome::Cancelled => ("Cancelled", task.cost.clone(), None),
            TaskOutcome::AmbiguousEffect { reason } => {
                ("AmbiguousEffect", task.cost.clone(), Some(reason.clone()))
            }
        };
        Receipt {
            outcome: outcome_name.to_string(),
            cost,
            wall_ms,
            reason,
            attempt,
            executor,
            artifacts: vec![], // populated by callers that stage outputs
        }
    }
}

impl<E: TaskExecutor> TaskExecutor for RecordingExecutor<E> {
    fn run(&self, task: &Task) -> TaskOutcome {
        let Some(op) = self.planned.get(&task.id).cloned() else {
            // No plan => no dispatch event may exist; refuse cleanly.
            return TaskOutcome::Failed {
                reason: "OPS_UNPLANNED: plan_task before run".to_string(),
            };
        };
        // Attempt counter (each run of one task is one attempt).
        let attempt = {
            let mut attempts = self.attempts.lock().expect("attempts lock");
            let counter = attempts.entry(task.id.clone()).or_insert(0);
            *counter += 1;
            *counter
        };
        // Append BEFORE the effect (MASTER_SPEC:369).
        {
            let mut rec = self.recorder.lock().expect("recorder lock");
            if let Err(e) = rec.dispatched(&op, attempt) {
                return TaskOutcome::Failed {
                    reason: format!("OPS_RECORD: {e}"),
                };
            }
        }
        let started = std::time::Instant::now();
        let outcome = self.inner.run(task);
        let wall_ms = started.elapsed().as_millis();
        // Receipt after the effect: facts first, then durable record.
        {
            let mut rec = self.recorder.lock().expect("recorder lock");
            let receipt = Self::receipt_for(
                task,
                &outcome,
                wall_ms,
                attempt,
                std::any::type_name::<E>().to_string(),
            );
            if let Err(e) = rec.receipt(&op, &receipt) {
                // Surface recording failure without hiding it in logs:
                // the effect's outcome already happened, but the audit
                // trail must be loud.
                return TaskOutcome::Failed {
                    reason: format!("OPS_RECEIPT: {e}"),
                };
            }
        }
        outcome
    }
}
