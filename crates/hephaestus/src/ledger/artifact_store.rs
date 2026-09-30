//! Content-addressed artifact storage: stage, commit, verify, sweep.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use crate::security::artifact::ArtifactDigest;

use super::LedgerError;

/// Counter making staged file names unique within a process.
static STAGE_SEQ: AtomicU64 = AtomicU64::new(0);

/// A staged, not-yet-addressable artifact.
#[derive(Debug, Clone)]
pub struct StagedArtifact {
    digest: ArtifactDigest,
    path: PathBuf,
}

impl StagedArtifact {
    /// The digest these bytes will carry once committed.
    pub fn digest(&self) -> &str {
        self.digest.as_str()
    }
}

/// Content-addressed artifact storage (T-006).
///
/// Bytes are staged outside the object namespace, then become addressable
/// only through one atomic rename. Committed objects are immutable and never
/// garbage-collected; only staging entries are GC candidates.
#[derive(Debug, Clone)]
pub struct ArtifactStore {
    root: PathBuf,
}

impl ArtifactStore {
    /// Open (creating if absent) a store rooted at `root`.
    pub fn open(root: &Path) -> Result<Self, LedgerError> {
        std::fs::create_dir_all(root.join("staging"))?;
        std::fs::create_dir_all(root.join("objects"))?;
        Ok(ArtifactStore {
            root: root.to_path_buf(),
        })
    }

    /// Stage `bytes`; durable on return, but not yet addressable.
    pub fn stage(&self, bytes: &[u8]) -> Result<StagedArtifact, LedgerError> {
        let digest = ArtifactDigest::of_bytes(bytes);
        let unique = STAGE_SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = self.root.join("staging").join(format!(
            "{}-{}-{}.part",
            digest.as_str(),
            nanos,
            unique
        ));
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        file.write_all(bytes)?;
        file.flush()?;
        file.sync_data()?;
        Ok(StagedArtifact { digest, path })
    }

    /// Commit `staged` by atomic rename into the object namespace.
    ///
    /// The staging file is consumed. Callers append their referencing ledger
    /// event only after this succeeds — a crash in between leaves a
    /// content-addressed orphan, which is safe and retained.
    pub fn commit(&self, staged: &StagedArtifact) -> Result<String, LedgerError> {
        let hex = staged.digest.as_str().to_string();
        let dir = self.root.join("objects").join(&hex[..2]);
        std::fs::create_dir_all(&dir)?;
        std::fs::rename(&staged.path, dir.join(&hex))?;
        Ok(hex)
    }

    /// Read the object at `digest`, re-hashing the bytes to verify them.
    pub fn read(&self, digest: &str) -> Result<Vec<u8>, LedgerError> {
        let expected = ArtifactDigest::parse(digest).map_err(|_| LedgerError::BadDigest {
            value: digest.to_string(),
        })?;
        let path = self
            .root
            .join("objects")
            .join(&expected.as_str()[..2])
            .join(expected.as_str());
        let bytes = std::fs::read(path)?;
        let actual = ArtifactDigest::of_bytes(&bytes);
        if actual.as_str() != expected.as_str() {
            return Err(LedgerError::DigestMismatch {
                expected: expected.as_str().to_string(),
                actual: actual.as_str().to_string(),
            });
        }
        Ok(bytes)
    }

    /// Remove staging entries last modified before `cutoff`.
    ///
    /// Age alone is sufficient: a successful commit *moves* the staging file
    /// into the object namespace, so anything still in `staging/` was never
    /// committed, and removing it cannot lose data (committed objects live
    /// elsewhere and are never candidates). That safety rests on the caller
    /// convention documented on [`ArtifactStore::commit`] (reference event
    /// only *after* the rename): the store cannot see the ledger, so the
    /// ordering is enforced by contract and tests, not by types. Entries
    /// newer than `cutoff` — a live writer's in-flight file — are protected.
    pub fn gc(&self, cutoff: SystemTime) -> Result<Vec<String>, LedgerError> {
        let mut removed = Vec::new();
        for entry in std::fs::read_dir(self.root.join("staging"))? {
            let entry = entry?;
            if entry.metadata()?.modified()? < cutoff {
                std::fs::remove_file(entry.path())?;
                removed.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        Ok(removed)
    }
}
