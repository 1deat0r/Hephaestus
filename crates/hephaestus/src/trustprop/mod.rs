//! Trust-origin propagation (T-043, R-108).
//!
//! Derived records — summaries, caches, knowledge records, memory
//! artifacts, graph edges — must INHERIT the trust origin of their
//! sources with a minimum-of-inputs rule: one untrusted input
//! contaminates the derivation, and trust is never laundered by
//! deriving through a trusted co-reference.

use crate::contracts::generated::TrustOrigin;
use serde::{Deserialize, Serialize};

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
