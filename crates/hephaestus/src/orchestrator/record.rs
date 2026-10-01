//! Orchestrator records (T-017): search nodes, bounds, archive, policy.

use serde::{Deserialize, Serialize};

/// Why expansion stopped (R-031): every bound records its reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopReason {
    /// Nothing left to expand.
    FrontierExhausted,
    /// Total candidate count bound reached.
    CandidateBoundReached,
    /// Expansion depth bound reached.
    DepthBoundReached,
    /// Operator-call bound reached.
    OperatorCallBoundReached,
    /// Duplicate-generation bound reached (AT-031).
    DuplicateBoundReached,
}

/// Payload kinds the search graph carries (§11:217). Experiment nodes are
/// M3 scope; here: opportunities and their downstream candidates as
/// statements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodePayload {
    Opportunity,
    Mechanism,
    Hypothesis,
}

/// A search node (MASTER_SPEC §11:217): parents, operator, evidence
/// snapshot, estimated cost, rejection reason.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchNode {
    pub id: u64,
    pub parents: Vec<u64>,
    /// Which operator produced this node ("seed" for roots).
    pub operator: String,
    pub statement: String,
    /// Evidence snapshot: span references captured at creation.
    pub evidence_snapshot: String,
    pub estimated_cost: f64,
    pub depth: usize,
    pub rejection: Option<String>,
}

/// Bounds on expansion (R-031, §11:229).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchBounds {
    pub max_depth: usize,
    pub max_candidates: usize,
    pub max_operator_calls: usize,
    /// Consecutive duplicate strikes before the loop is declared
    /// unproductive (AT-031).
    pub max_duplicate_strikes: usize,
}

/// An archived (non-kept or rejected) candidate with its reason (R-024).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub node: SearchNode,
    pub wave: usize,
    pub reason: String,
}

/// Provisional allocation policy (§11:221): 55% promising, 25% diverse/
/// uncertain, 15% replication/counterevidence, 5% auditing rejections.
/// Versioned — changes only through a logged policy version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllocationPolicy {
    pub version: u32,
    pub exploit_pct: u8,
    pub diversify_pct: u8,
    pub replicate_pct: u8,
    pub audit_pct: u8,
}

impl Default for AllocationPolicy {
    fn default() -> Self {
        Self {
            version: 1,
            exploit_pct: 55,
            diversify_pct: 25,
            replicate_pct: 15,
            audit_pct: 5,
        }
    }
}

/// The outcome: nodes, archive, stop reason, policy visibility (§11:221).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchOutcome {
    pub nodes: Vec<SearchNode>,
    pub archive: Vec<ArchiveEntry>,
    pub stop: Option<StopReason>,
    pub allocation_policy: AllocationPolicy,
    pub expansions_considered: usize,
    pub duplicates_strike: usize,
}
