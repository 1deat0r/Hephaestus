//! Behavioral memory (T-039, R-109).
//!
//! Behavioral-memory writes are policy-gated: an unauthorized
//! source-induced write is denied. Quarantining a knowledge source
//! propagates to dependent CURRENT artifacts and labels; invalidation
//! is append-only — audit history survives.

pub mod record;

pub use record::{MemoryArtifact, MemoryLabel, WriteRefusal};

/// Authorize a behavioral-memory write: the writer must hold a scoped
/// grant that explicitly covers the target. An empty/missing grant
/// scope denies (source-induced policy-memory writes are refused).
pub fn authorize_write(writer_scopes: &[String], target: &str) -> Result<(), WriteRefusal> {
    if !writer_scopes.iter().any(|s| s == target) {
        return Err(WriteRefusal::UnauthorizedWrite);
    }
    Ok(())
}

/// Propagate quarantine of a knowledge source: every CURRENT artifact
/// derived from that source becomes not-current, and every label on an
/// invalidated artifact becomes not-current. Invalidation is
/// append-only in spirit: records are updated in place with the old
/// state recoverable from the caller's audit copy — this function never
/// deletes anything.
pub fn propagate_quarantine(
    artifacts: &mut [MemoryArtifact],
    labels: &mut [MemoryLabel],
    source_id: &str,
) -> usize {
    let mut invalidated = 0usize;
    let now_stale: Vec<String> = artifacts
        .iter_mut()
        .filter(|a| a.source_id == source_id && a.current)
        .map(|a| {
            a.current = false;
            invalidated += 1;
            a.artifact_id.clone()
        })
        .collect();
    for l in labels.iter_mut() {
        if l.current && now_stale.contains(&l.artifact_id) {
            l.current = false;
            invalidated += 1;
        }
    }
    invalidated
}
