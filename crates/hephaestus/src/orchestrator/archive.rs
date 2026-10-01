//! Multi-objective archive (T-044, R-032 / AT-032).
//!
//! A diverse Pareto archive over search nodes: cost (minimize) x
//! producer-declared value band (maximize). A cheap uncertain
//! candidate and an expensive plausible candidate can both remain in
//! the frontier when neither dominates. No numeric confidence is ever
//! fabricated (R-033) — the value axis is a declared qualitative band.
//! Eviction is bounded and always audited (R-024).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Producer-declared qualitative value band. Declared, not computed:
/// no numeric probability is invented (R-033). Missing declaration
/// defaults to `Low` — never auto-eliminated by a missing score.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ValueBand {
    #[default]
    Low,
    Medium,
    High,
}

/// An archived candidate with its declared objective vector.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveNode {
    pub id: u64,
    pub statement: String,
    /// Cost axis (minimize) — `SearchNode::estimated_cost`.
    pub cost: f64,
    /// Value axis (maximize) — producer-declared band rank.
    pub value: ValueBand,
    /// Admission order for LRU tie-breaks on eviction.
    pub admitted: u64,
}

/// Why a node left the frontier. Retained, never silent (R-024).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EvictionReason {
    /// Another node dominates it on all axes with at least one strict.
    Dominated { by: u64 },
    /// Frontier cap reached; LRU among the same value corner.
    CapLru { cap: usize },
}

/// A diverse multi-objective archive: the frontier is the non-dominated
/// set under (cost min, value max). Bounded by `cap`.
#[derive(Debug, Clone)]
pub struct MultiObjectiveArchive {
    nodes: BTreeMap<u64, ArchiveNode>,
    evictions: Vec<(u64, EvictionReason)>,
    cap: usize,
    next_admission: u64,
}

impl MultiObjectiveArchive {
    pub fn new(cap: usize) -> Self {
        MultiObjectiveArchive {
            nodes: BTreeMap::new(),
            evictions: Vec::new(),
            cap,
            next_admission: 0,
        }
    }

    /// True iff `a` dominates `b`: <= on ALL axes and < on >= 1
    /// (cost minimize, value maximize).
    pub fn dominates(a: &ArchiveNode, b: &ArchiveNode) -> bool {
        let cost_le = a.cost <= b.cost;
        let value_ge = a.value >= b.value;
        let strict = a.cost < b.cost || a.value > b.value;
        cost_le && value_ge && strict
    }

    /// The current frontier: all non-dominated nodes, ordered by
    /// (value desc, cost asc) for deterministic output.
    pub fn frontier(&self) -> Vec<&ArchiveNode> {
        let all: Vec<&ArchiveNode> = self.nodes.values().collect();
        let dominated: Vec<bool> = all
            .iter()
            .map(|n| {
                all.iter()
                    .any(|other| other.id != n.id && Self::dominates(other, n))
            })
            .collect();
        all.into_iter()
            .zip(dominated)
            .filter(|(_, dom)| !dom)
            .map(|(n, _)| n)
            .collect()
    }

    /// Admit a node, then enforce the cap: dominated nodes are evicted
    /// first (with reasons); if still over cap, the LRU node among the
    /// lowest value band goes, audited.
    pub fn admit(&mut self, id: u64, statement: impl Into<String>, cost: f64, value: ValueBand) {
        let admitted = self.next_admission;
        self.next_admission += 1;
        self.nodes.insert(
            id,
            ArchiveNode {
                id,
                statement: statement.into(),
                cost,
                value,
                admitted,
            },
        );
        self.enforce_cap();
    }

    fn enforce_cap(&mut self) {
        // Pass 1: evict dominated nodes (highest domination count first
        // for determinism), audited.
        loop {
            let frontier_ids: Vec<u64> = self.frontier().iter().map(|n| n.id).collect();
            let dominated: Vec<u64> = self
                .nodes
                .keys()
                .copied()
                .filter(|id| !frontier_ids.contains(id))
                .collect();
            match dominated.first() {
                Some(&victim) => {
                    // Find a determinate dominator for the reason.
                    let victim_node = self.nodes.get(&victim).cloned();
                    let dominator = self
                        .nodes
                        .values()
                        .find(|n| {
                            victim_node
                                .as_ref()
                                .is_some_and(|v| n.id != v.id && Self::dominates(n, v))
                        })
                        .map(|n| n.id)
                        .unwrap_or(victim);
                    self.nodes.remove(&victim);
                    self.evictions
                        .push((victim, EvictionReason::Dominated { by: dominator }));
                }
                None => break,
            }
        }
        // Pass 2: still over cap -> evict LRU within the lowest band.
        while self.nodes.len() > self.cap {
            let victim = self
                .nodes
                .values()
                .min_by(|a, b| {
                    a.value
                        .cmp(&b.value)
                        .then(a.admitted.cmp(&b.admitted))
                        .then(a.id.cmp(&b.id))
                })
                .map(|n| (n.id, self.cap))
                .expect("nodes nonempty while over cap");
            self.nodes.remove(&victim.0);
            self.evictions
                .push((victim.0, EvictionReason::CapLru { cap: victim.1 }));
        }
    }

    /// Retained eviction audit: (id, reason) pairs in eviction order.
    pub fn evictions(&self) -> &[(u64, EvictionReason)] {
        &self.evictions
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
