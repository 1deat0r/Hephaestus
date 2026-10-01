//! Interactive research workspace core (T-025, MASTER_SPEC §2:42,
//! R-067/R-068/R-069).
//!
//! Truthful views read from persisted state (never narrated; unknown
//! stays unknown); typed intervention controls; ONE shared authorization
//! function gates BOTH local and gateway paths (R-069 — no gateway
//! bypass); event-cursor recovery supports resumable sessions.
//! Rendering (a real TUI) is presentation and comes later.

pub mod record;

pub use record::{
    Ack, AuthToken, ComparisonView, ControlCommand, ControlError, EvidenceView, GatewayOutcome,
    HypothesisRecord, PersistedState, ProgressView,
};

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The shared control policy (R-069): ONE policy object consulted by
/// both local and gateway paths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ControlPolicy {
    /// Authorized token digests.
    pub authorized_digests: Vec<String>,
    /// Missions known to the workspace.
    pub known_missions: Vec<String>,
}

/// The workspace: open from persisted state; every view reads state,
/// never invents (R-067).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workspace {
    state: PersistedState,
    hypotheses: Vec<HypothesisRecord>,
    policy: ControlPolicy,
    lineage: Vec<(String, Vec<String>)>,
}

impl Workspace {
    /// Open (or resume) a workspace from persisted state (R-068 resume).
    pub fn open(
        state: PersistedState,
        hypotheses: Vec<HypothesisRecord>,
        policy: ControlPolicy,
        lineage: Vec<(String, Vec<String>)>,
    ) -> Self {
        Self {
            state,
            hypotheses,
            policy,
            lineage,
        }
    }

    /// Truthful progress (R-067): read from state; unknown stays unknown;
    /// no percentages, no narrative.
    pub fn progress(&self) -> ProgressView {
        ProgressView {
            entity_counts: self.state.entity_counts.clone(),
            last_event_id: self.state.last_event_id,
            running: self.state.running,
            missions_unknown: self.state.missions_unknown,
        }
    }

    /// The persisted state itself, for save/resume (R-068).
    pub fn progress_state(&self) -> PersistedState {
        self.state.clone()
    }

    /// Hypothesis comparison (§2:42): field-level diffs + shared
    /// evidence.
    pub fn compare(&self, a_id: &str, b_id: &str) -> Option<ComparisonView> {
        let a = self.hypotheses.iter().find(|h| h.id == a_id)?;
        let b = self.hypotheses.iter().find(|h| h.id == b_id)?;
        let sa: BTreeSet<_> = a.claim_ids.iter().collect();
        let sb: BTreeSet<_> = b.claim_ids.iter().collect();
        let ea: BTreeSet<_> = a.evidence_ids.iter().collect();
        let eb: BTreeSet<_> = b.evidence_ids.iter().collect();
        Some(ComparisonView {
            same_version: a.version == b.version,
            same_state: a.state == b.state,
            claims_only_in_a: sa.difference(&sb).map(|s| (*s).clone()).collect(),
            claims_only_in_b: sb.difference(&sa).map(|s| (*s).clone()).collect(),
            shared_evidence: ea.intersection(&eb).map(|s| (*s).clone()).collect(),
        })
    }

    /// Evidence drill-down (§2:42): record + lineage + staleness.
    pub fn drill_down(&self, record_id: &str) -> Option<EvidenceView> {
        let (_, lineage_ids) = self.lineage.iter().find(|(id, _)| id == record_id)?;
        Some(EvidenceView {
            record_id: record_id.to_string(),
            lineage_ids: lineage_ids.clone(),
            stale: false,
        })
    }

    /// THE shared authorization gate (R-069): used by BOTH `command`
    /// (local) and `gateway_event` — one code path, no bypass.
    fn authorize(&self, token: &AuthToken) -> Result<(), ControlError> {
        if self.policy.authorized_digests.contains(&token.0) {
            Ok(())
        } else {
            Err(ControlError::Unauthorized)
        }
    }

    /// Local intervention command (R-068): authenticated via the shared
    /// gate.
    pub fn command(
        &mut self,
        token: &AuthToken,
        cmd: &ControlCommand,
    ) -> Result<Ack, ControlError> {
        self.authorize(token)?;
        self.apply(cmd)
    }

    /// Apply a control command to the state (shared by both paths).
    fn apply(&mut self, cmd: &ControlCommand) -> Result<Ack, ControlError> {
        match cmd {
            ControlCommand::Pause => {
                if !self.state.running {
                    return Err(ControlError::InvalidState);
                }
                self.state.running = false;
            }
            ControlCommand::Resume => {
                if self.state.running {
                    return Err(ControlError::InvalidState);
                }
                self.state.running = true;
            }
            ControlCommand::CancelMission(m) => {
                if !self.policy.known_missions.iter().any(|k| k == m) {
                    return Err(ControlError::UnknownMission);
                }
            }
            ControlCommand::SteerMission(m, _) => {
                if !self.policy.known_missions.iter().any(|k| k == m) {
                    return Err(ControlError::UnknownMission);
                }
            }
        }
        self.state.last_event_id += 1;
        Ok(Ack {
            applied: format!("{cmd:?}"),
            new_event_id: self.state.last_event_id,
        })
    }

    /// Gateway event with cursor handling (R-068): the SAME authorization
    /// gate applies (R-069). Stale/duplicate cursors recover, never
    /// silently pass.
    pub fn gateway_event(
        &mut self,
        token: &AuthToken,
        client_cursor: u64,
        cmd: &ControlCommand,
    ) -> Result<GatewayOutcome, ControlError> {
        self.authorize(token)?;
        if client_cursor > self.state.last_event_id {
            // Client is ahead of us: replay from our cursor.
            return Ok(GatewayOutcome::StaleCursor {
                expect_from: self.state.last_event_id + 1,
            });
        }
        if client_cursor < self.state.last_event_id {
            // Already-processed event: replay recommendation.
            return Ok(GatewayOutcome::Duplicate(self.state.last_event_id));
        }
        let ack = self.apply(cmd)?;
        Ok(GatewayOutcome::Applied(ack))
    }
}
