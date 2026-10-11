//! Trust-origin propagation (T-043, R-108).
//!
//! Derived records — summaries, caches, knowledge records, memory
//! artifacts, graph edges — must INHERIT the trust origin of their
//! sources with a minimum-of-inputs rule: one untrusted input
//! contaminates the derivation, and trust is never laundered by
//! deriving through a trusted co-reference.

use crate::contracts::generated::TrustOrigin;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Trust ranking (weakest first): synthetic fixtures prove
/// construction, not observation, so they rank BELOW observed tool
/// output and above untrusted model output.
pub fn trust_rank(origin: &TrustOrigin) -> u8 {
    match origin {
        TrustOrigin::UntrustedSource => 0,
        TrustOrigin::UntrustedModel => 1,
        TrustOrigin::SyntheticFixture => 2,
        TrustOrigin::ObservedTool => 3,
        TrustOrigin::Owner => 4,
        TrustOrigin::ProtectedService => 5,
    }
}

/// The inherited trust of a derivation: the MINIMUM of its inputs'
/// origins. Empty input -> UntrustedSource (nothing known = not
/// trusted), never Owner.
pub fn derive_origin(inputs: &[TrustOrigin]) -> TrustOrigin {
    let mut min = TrustOrigin::UntrustedSource;
    let mut min_rank = 0u8;
    let mut saw_any = false;
    for o in inputs {
        let r = trust_rank(o);
        if !saw_any || r < min_rank {
            min_rank = r;
            min = *o;
        }
        saw_any = true;
    }
    if saw_any {
        min
    } else {
        TrustOrigin::UntrustedSource
    }
}

/// A stamp attached to any derived record: what it was derived from
/// (source refs) and the inherited origin. Serializable so the stamp
/// survives ledgers, caches, and cross-session memory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedTrust {
    pub source_refs: Vec<String>,
    pub inherited_origin: TrustOrigin,
    /// The derivation kind, for audit (e.g. "summary", "cache",
    /// "graph-edge", "cross-session").
    pub derivation: String,
}

impl DerivedTrust {
    /// Build a stamp from sources, applying the min-of-inputs rule.
    pub fn derive(source_refs: &[&str], source_origins: &[TrustOrigin], derivation: &str) -> Self {
        DerivedTrust {
            source_refs: source_refs.iter().map(|s| s.to_string()).collect(),
            inherited_origin: derive_origin(source_origins),
            derivation: derivation.to_string(),
        }
    }
}

/// One recorded input edge: derivative version built from an exact
/// artifact version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyLink {
    /// The derived artifact id.
    pub derivative: String,
    /// The derived artifact version.
    pub derivative_version: u64,
    /// The input artifact id.
    pub artifact: String,
    /// The exact input artifact version the derivation consumed.
    pub artifact_version: u64,
}

/// Why dependency recording refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexError {
    /// The derivative version already records different inputs:
    /// conflicting provenance, never merged.
    ConflictingInputs {
        /// The derivative id.
        derivative: String,
        /// The derivative version.
        version: u64,
    },
}

impl std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConflictingInputs {
                derivative,
                version,
            } => {
                write!(f, "conflicting inputs for {derivative} v{version}")
            }
        }
    }
}

impl std::error::Error for IndexError {}

/// Sort versioned nodes and drop duplicates, for deterministic output.
fn sorted_unique(mut nodes: Vec<(String, u64)>) -> Vec<(String, u64)> {
    nodes.sort();
    nodes.dedup();
    nodes
}

/// Rebuildable reverse-dependency index over versioned artifacts.
///
/// Nodes are (id, version) pairs: a derivative of artifact v1 is
/// unaffected by artifact v2. Forward edges map a derivative version to
/// the exact input versions it consumed; reverse edges map an artifact
/// version to its direct dependents. Traversal terminates on cycles via
/// a visited set and counts multiply-reached derivatives once. The
/// exported link list rebuilds an identical index.
#[derive(Debug, Clone, Default)]
pub struct DependencyIndex {
    forward: HashMap<(String, u64), Vec<(String, u64)>>,
    reverse: HashMap<(String, u64), Vec<(String, u64)>>,
}

impl DependencyIndex {
    /// Empty index.
    pub fn new() -> Self {
        DependencyIndex {
            forward: HashMap::new(),
            reverse: HashMap::new(),
        }
    }

    /// Rebuild an index from an exported link list. Conflicting links
    /// refuse exactly as direct recording does.
    pub fn rebuild(links: &[DependencyLink]) -> Result<Self, IndexError> {
        let mut grouped: HashMap<(String, u64), Vec<(String, u64)>> = HashMap::new();
        for link in links {
            grouped
                .entry((link.derivative.clone(), link.derivative_version))
                .or_default()
                .push((link.artifact.clone(), link.artifact_version));
        }
        let mut index = DependencyIndex::new();
        for ((derivative, version), inputs) in &grouped {
            let refs: Vec<(&str, u64)> = inputs
                .iter()
                .map(|(id, version)| (id.as_str(), *version))
                .collect();
            index.record(derivative, *version, &refs)?;
        }
        Ok(index)
    }

    /// Record that a derivative version consumed exact input versions.
    /// Re-recording identical inputs is idempotent; differing inputs
    /// refuse as conflicting provenance.
    pub fn record(
        &mut self,
        derivative: &str,
        derivative_version: u64,
        inputs: &[(&str, u64)],
    ) -> Result<(), IndexError> {
        let key = (derivative.to_string(), derivative_version);
        let normalized = sorted_unique(
            inputs
                .iter()
                .map(|(id, version)| (id.to_string(), *version))
                .collect(),
        );
        if let Some(existing) = self.forward.get(&key) {
            if *existing == normalized {
                return Ok(());
            }
            return Err(IndexError::ConflictingInputs {
                derivative: derivative.to_string(),
                version: derivative_version,
            });
        }
        for (artifact, version) in &normalized {
            self.reverse
                .entry((artifact.clone(), *version))
                .or_default()
                .push(key.clone());
        }
        self.forward.insert(key, normalized);
        Ok(())
    }

    /// Exact input versions a derivative version consumed, sorted.
    pub fn direct_inputs(&self, derivative: &str, version: u64) -> Vec<(String, u64)> {
        self.forward
            .get(&(derivative.to_string(), version))
            .cloned()
            .unwrap_or_default()
    }

    /// Derivative versions built directly from an artifact version, sorted.
    pub fn direct_dependents(&self, artifact: &str, version: u64) -> Vec<(String, u64)> {
        sorted_unique(
            self.reverse
                .get(&(artifact.to_string(), version))
                .cloned()
                .unwrap_or_default(),
        )
    }

    /// All transitively affected derivative versions, sorted and deduped.
    /// The queried node itself is excluded. Cycles terminate via the
    /// visited set: each node expands once.
    pub fn affected(&self, artifact: &str, version: u64) -> Vec<(String, u64)> {
        let start = (artifact.to_string(), version);
        let mut visited: HashSet<(String, u64)> = HashSet::from([start]);
        let mut queue: VecDeque<(String, u64)> = self
            .direct_dependents(artifact, version)
            .into_iter()
            .collect();
        while let Some(node) = queue.pop_front() {
            if !visited.insert(node.clone()) {
                continue;
            }
            for next in self.direct_dependents(&node.0, node.1) {
                if !visited.contains(&next) {
                    queue.push_back(next);
                }
            }
        }
        visited.remove(&(artifact.to_string(), version));
        sorted_unique(visited.into_iter().collect())
    }

    /// Every recorded input edge, sorted. Feeds `rebuild` for a
    /// persistence round-trip.
    pub fn links(&self) -> Vec<DependencyLink> {
        let mut out = Vec::new();
        for ((derivative, derivative_version), inputs) in &self.forward {
            for (artifact, artifact_version) in inputs {
                out.push(DependencyLink {
                    derivative: derivative.clone(),
                    derivative_version: *derivative_version,
                    artifact: artifact.clone(),
                    artifact_version: *artifact_version,
                });
            }
        }
        out.sort_by(|a, b| {
            (
                &a.derivative,
                a.derivative_version,
                &a.artifact,
                a.artifact_version,
            )
                .cmp(&(
                    &b.derivative,
                    b.derivative_version,
                    &b.artifact,
                    b.artifact_version,
                ))
        });
        out
    }
}
