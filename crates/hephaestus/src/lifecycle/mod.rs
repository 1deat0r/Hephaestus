//! Evidence lifecycle (T-024, MASTER_SPEC section 16): explicit states,
//! append-only invalidation, budget-gated portfolio queue, failure
//! reactivation. Results connect back into the graph; corrections and
//! retractions invalidate dependent interpretations without erasing
//! history; restarting research still requires available budget.

pub mod record;

pub use record::{
    CandidateFields, Correction, CorrectionKind, HypothesisState, InvalidationReport,
    OpportunityState, QueueError, Reactivation, ReevaluationEntry, StaleMark, TransitionError,
    TransitionRecord,
};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Legal opportunity transitions (§16:311). Parked reachable from
/// grounded/prioritized/explored; closed from any active state.
fn opportunity_transition(from: OpportunityState, to: OpportunityState) -> bool {
    use OpportunityState::*;
    matches!(
        (from, to),
        (Discovered, Grounded)
            | (Grounded, Prioritized)
            | (Prioritized, Explored)
            | (Explored, Closed)
            | (Grounded, Parked)
            | (Prioritized, Parked)
            | (Explored, Parked)
            | (Parked, Prioritized) // reactivation via budget gate
            | (_, Closed)
    )
}

/// Legal hypothesis transitions (§16:311). Blocked from compiled onwards;
/// archived from any state.
fn hypothesis_transition(from: HypothesisState, to: HypothesisState) -> bool {
    use HypothesisState::*;
    matches!(
        (from, to),
        (Exploratory, Compiled)
            | (Compiled, Reviewed)
            | (Reviewed, TestReady)
            | (TestReady, Testing)
            | (Testing, Assessed)
            | (Compiled, Blocked)
            | (Reviewed, Blocked)
            | (TestReady, Blocked)
            | (Blocked, Compiled) // unblock returns to compiled
            | (Exploratory, Archived)
            | (Assessed, Archived)
            | (Blocked, Archived)
    )
}

/// Advance an opportunity or hypothesis (§16:311): named illegal
/// transitions; every legal transition binds a snapshot ID (§16:318)
/// and records lineage (R-097).
pub fn advance_opportunity(
    entity_id: &str,
    from: OpportunityState,
    to: OpportunityState,
    snapshot_id: &str,
    lineage_mission: &str,
    lineage_hypothesis_version: &str,
) -> Result<TransitionRecord, TransitionError> {
    if !opportunity_transition(from, to) {
        return Err(TransitionError {
            from: format!("{from:?}"),
            to: format!("{to:?}"),
            reason: "illegal opportunity transition".to_string(),
        });
    }
    Ok(TransitionRecord {
        entity_id: entity_id.to_string(),
        from_state: format!("{from:?}"),
        to_state: format!("{to:?}"),
        snapshot_id: snapshot_id.to_string(),
        lineage_mission: lineage_mission.to_string(),
        lineage_hypothesis_version: lineage_hypothesis_version.to_string(),
    })
}

/// Advance a hypothesis (§16:311).
pub fn advance_hypothesis(
    entity_id: &str,
    from: HypothesisState,
    to: HypothesisState,
    snapshot_id: &str,
    lineage_mission: &str,
    lineage_hypothesis_version: &str,
) -> Result<TransitionRecord, TransitionError> {
    if !hypothesis_transition(from, to) {
        return Err(TransitionError {
            from: format!("{from:?}"),
            to: format!("{to:?}"),
            reason: "illegal hypothesis transition".to_string(),
        });
    }
    Ok(TransitionRecord {
        entity_id: entity_id.to_string(),
        from_state: format!("{from:?}"),
        to_state: format!("{to:?}"),
        snapshot_id: snapshot_id.to_string(),
        lineage_mission: lineage_mission.to_string(),
        lineage_hypothesis_version: lineage_hypothesis_version.to_string(),
    })
}

/// The evidence graph: append-only records (§16:314). Corrections are
/// new events; originals are never erased.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EvidenceGraph {
    /// record_id -> dependent record IDs (edges).
    dependencies: BTreeMap<String, Vec<String>>,
    /// record_id -> current-stale flag. Historical results stay intact
    /// (§16:317); only the CURRENT label goes stale.
    stale_labels: BTreeMap<String, bool>,
    /// Append-only correction log (R-017).
    corrections: Vec<Correction>,
}

impl EvidenceGraph {
    pub fn add_dependency(&mut self, record_id: &str, depends_on: &str) {
        self.dependencies
            .entry(depends_on.to_string())
            .or_default()
            .push(record_id.to_string());
    }

    /// Register a correction/retraction (append-only) and traverse
    /// dependents: mark dossiers/records stale, queue re-evaluation
    /// (which awaits budget+permission — never auto-dispatched,
    /// §16:314).
    pub fn invalidate(&mut self, correction: Correction) -> InvalidationReport {
        self.corrections.push(correction.clone());
        let mut report = InvalidationReport::default();
        let mut frontier = vec![correction.target_id.clone()];
        let mut visited = std::collections::BTreeSet::new();
        while let Some(id) = frontier.pop() {
            if !visited.insert(id.clone()) {
                continue;
            }
            // The corrected record's current label goes stale; its
            // historical result remains intact (§16:317).
            self.stale_labels.insert(id.clone(), true);
            report.stale.push(StaleMark {
                dependent_id: id.clone(),
                via: correction.correction_id.clone(),
                historical_result_intact: true,
            });
            report.reevaluation.push(ReevaluationEntry {
                target_id: id.clone(),
                requires_budget_and_permission: true,
            });
            if let Some(dependents) = self.dependencies.get(&id) {
                frontier.extend(dependents.iter().cloned());
            }
        }
        report
    }

    pub fn is_stale(&self, record_id: &str) -> bool {
        self.stale_labels.get(record_id).copied().unwrap_or(false)
    }

    pub fn corrections(&self) -> &[Correction] {
        &self.corrections
    }
}

/// Queue a candidate into the portfolio (roadmap: restarting research
/// still requires available budget): refusal without budget; every
/// entry binds a snapshot (§16:318).
pub fn queue(
    candidate_id: &str,
    budget_remaining: u64,
    snapshot_id: Option<&str>,
) -> Result<TransitionRecord, QueueError> {
    if budget_remaining == 0 {
        return Err(QueueError::BudgetUnavailable);
    }
    let snapshot_id = snapshot_id.ok_or(QueueError::MissingSnapshot)?;
    Ok(TransitionRecord {
        entity_id: candidate_id.to_string(),
        from_state: "reactivated".to_string(),
        to_state: "queued".to_string(),
        snapshot_id: snapshot_id.to_string(),
        lineage_mission: String::new(),
        lineage_hypothesis_version: String::new(),
    })
}

/// Reactivate a previously failed mechanism (§16:316): a NEW version
/// referencing the original failure + the changed-condition evidence;
/// the failure is not erased and no claim is made that the new condition
/// fixes it.
pub fn reactivate(
    new_version_id: &str,
    original_failure_id: &str,
    changed_condition_evidence: &str,
) -> Reactivation {
    Reactivation {
        new_version_id: new_version_id.to_string(),
        original_failure_id: original_failure_id.to_string(),
        changed_condition_evidence: changed_condition_evidence.to_string(),
    }
}
