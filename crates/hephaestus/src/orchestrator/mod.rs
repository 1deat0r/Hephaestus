//! Bounded search orchestrator (T-017, MASTER_SPEC section 11).
//!
//! A transparent best-first beam over the discovery→genesis chain: search
//! nodes carry parents, operator, evidence snapshot, estimated cost, and
//! rejection reasons (§11:217). Deduplication stops paraphrase loops
//! (AT-031); bounds stop expansion with recorded reasons (R-031); the
//! diversity archive retains every non-kept candidate (R-024); the
//! allocation policy is versioned and visible (§11:221). No learned
//! controller, no fabricated posteriors (§11:225).

pub mod archive;
pub mod record;

pub use record::{
    AllocationPolicy, ArchiveEntry, NodePayload, SearchBounds, SearchNode, SearchOutcome,
    StopReason,
};

use crate::discovery::Opportunity;
use std::collections::BTreeMap;

/// Derive a normalized dedup key from a candidate statement: lowercase,
/// whitespace-collapsed, tokens sorted — so paraphrases that reuse the
/// same content words collide (AT-031) while genuinely different
/// candidates do not.
pub fn dedup_key(statement: &str) -> String {
    let lowered = statement.to_lowercase();
    let mut tokens: Vec<&str> = lowered
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .collect();
    tokens.sort_unstable();
    tokens.join(" ")
}

/// Run a bounded best-first beam over candidate expansions.
///
/// `seeds` are the starting opportunities (evidence snapshots carried on
/// the seed node). `expansions` maps each parent candidate's dedup key to
/// the children an operator WOULD produce next (the caller supplies the
/// expansion function's results; the orchestrator owns the loop, bounds,
/// beam, dedup, archive, and audit).
pub fn run(
    seed: &Opportunity,
    expansions: &BTreeMap<String, Vec<String>>,
    bounds: &SearchBounds,
    beam_width: usize,
    policy: &AllocationPolicy,
) -> SearchOutcome {
    let mut outcome = SearchOutcome {
        nodes: Vec::new(),
        archive: Vec::new(),
        stop: None,
        allocation_policy: policy.clone(),
        expansions_considered: 0,
        duplicates_strike: 0,
    };

    // Seed node.
    let evidence = seed
        .evidence_references
        .iter()
        .map(|s| format!("{}:{}-{}", s.source, s.start, s.end))
        .collect::<Vec<_>>()
        .join(",");
    let seed_node = SearchNode {
        id: 0,
        parents: Vec::new(),
        operator: "seed".into(),
        statement: seed.problem_statement.clone(),
        evidence_snapshot: evidence,
        estimated_cost: 0.0,
        depth: 0,
        rejection: None,
    };
    outcome.nodes.push(seed_node);

    // Best-first frontier: (priority, node index). Priority = (has
    // evidence, token count) — named, deterministic, no fabricated
    // probabilities (§11:225).
    let mut frontier: Vec<usize> = vec![0];
    let mut seen_keys: std::collections::BTreeSet<String> =
        std::collections::BTreeSet::from([dedup_key(&seed.problem_statement)]);
    let mut next_id = 1u64;

    loop {
        // Bounds checks (R-031) — each stop reason recorded.
        if frontier.is_empty() {
            outcome.stop = Some(StopReason::FrontierExhausted);
            break;
        }
        if outcome.nodes.len() >= bounds.max_candidates {
            outcome.stop = Some(StopReason::CandidateBoundReached);
            break;
        }
        let &current = frontier.first().expect("frontier nonempty");
        if outcome.nodes[current].depth >= bounds.max_depth {
            outcome.stop = Some(StopReason::DepthBoundReached);
            break;
        }

        // Beam: keep only the top `beam_width` frontier nodes per wave.
        frontier.sort_by_key(|&n| {
            let node = &outcome.nodes[n];
            (node.evidence_snapshot.is_empty(), node.statement.len())
        });
        for &dropped in frontier.iter().skip(beam_width) {
            outcome.archive.push(ArchiveEntry {
                node: outcome.nodes[dropped].clone(),
                wave: outcome.nodes[dropped].depth,
                reason: "below_beam_cut".into(),
            });
        }
        frontier.truncate(beam_width);

        // Expand the current node (clone the fields we need so `outcome`
        // stays mutably borrowable inside the loop).
        let (node_id, node_statement, node_evidence, node_depth) = {
            let node = &outcome.nodes[current];
            (
                node.id,
                node.statement.clone(),
                node.evidence_snapshot.clone(),
                node.depth,
            )
        };
        let children = expansions
            .get(&dedup_key(&node_statement))
            .cloned()
            .unwrap_or_default();
        if children.is_empty() {
            // No expansion available: pop and continue.
            frontier.remove(0);
            if frontier.is_empty() {
                outcome.stop = Some(StopReason::FrontierExhausted);
                break;
            }
            continue;
        }
        if outcome.expansions_considered >= bounds.max_operator_calls {
            outcome.stop = Some(StopReason::OperatorCallBoundReached);
            break;
        }
        outcome.expansions_considered += 1;

        let mut kept = 0usize;
        for child_statement in children {
            if outcome.nodes.len() >= bounds.max_candidates {
                outcome.stop = Some(StopReason::CandidateBoundReached);
                break;
            }
            let key = dedup_key(&child_statement);
            if seen_keys.contains(&key) {
                // AT-031: paraphrase → expansion stops with reason.
                outcome.duplicates_strike += 1;
                if outcome.duplicates_strike >= bounds.max_duplicate_strikes {
                    outcome.stop = Some(StopReason::DuplicateBoundReached);
                }
                outcome.archive.push(ArchiveEntry {
                    node: SearchNode {
                        id: next_id,
                        parents: vec![node_id],
                        operator: "dedup_rejected".into(),
                        statement: child_statement.clone(),
                        evidence_snapshot: String::new(),
                        estimated_cost: 0.0,
                        depth: node_depth + 1,
                        rejection: Some("duplicate_generation".into()),
                    },
                    wave: node_depth + 1,
                    reason: "duplicate_generation".into(),
                });
                next_id += 1;
                continue;
            }
            seen_keys.insert(key);
            let child = SearchNode {
                id: next_id,
                parents: vec![node_id],
                operator: "expansion".into(),
                statement: child_statement.clone(),
                evidence_snapshot: node_evidence.clone(),
                estimated_cost: 1.0,
                depth: node_depth + 1,
                rejection: None,
            };
            next_id += 1;
            outcome.nodes.push(child.clone());
            if kept < beam_width {
                frontier.push(outcome.nodes.len() - 1);
                kept += 1;
            } else {
                outcome.archive.push(ArchiveEntry {
                    node: child,
                    wave: node_depth + 1,
                    reason: "below_beam_cut".into(),
                });
            }
        }
        frontier.remove(0);
        if outcome.stop.is_some() {
            break;
        }
    }

    sort_stable(&mut outcome);
    outcome
}

/// Full rejection audit (R-024): every archived entry, sorted for
/// determinism.
pub fn audit(outcome: &SearchOutcome) -> Vec<&ArchiveEntry> {
    outcome.archive.iter().collect()
}

/// Deterministic output ordering.
fn sort_stable(outcome: &mut SearchOutcome) {
    outcome.nodes.sort_by_key(|n| n.id);
    outcome.archive.sort_by_key(|e| (e.wave, e.node.id));
}

/// Lineage: walk parents from a node to its seed (§11:217). Returns the
/// path root-first.
pub fn lineage(outcome: &SearchOutcome, node_id: u64) -> Vec<u64> {
    let mut path = Vec::new();
    let mut current = Some(node_id);
    while let Some(id) = current {
        path.push(id);
        current = outcome
            .nodes
            .iter()
            .find(|n| n.id == id)
            .and_then(|n| n.parents.first().copied());
    }
    path.reverse();
    path
}
