//! Content-addressed artifact storage: stage, commit, verify, sweep.

use std::collections::{HashMap, VecDeque};
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

/// Bounded least-recently-used cache over committed store objects.
///
/// Resident bytes never exceed `capacity_bytes`: on a miss the cache evicts
/// the least-recently-used entries until the incoming object fits. Eviction
/// drops memory only — committed objects reload from disk, so evicted data
/// is always reloadable and originals stay retained. Objects larger than the
/// whole capacity bypass the cache: they are served verified but uncached
/// and counted as `uncached`. Counters record cold loads (`misses`), warm
/// reads (`hits`), evictions, bypasses, current residence, and the resident
/// high-water mark (`peak_bytes`).
#[derive(Debug)]
pub struct ObjectCache {
    store: ArtifactStore,
    capacity_bytes: u64,
    entries: HashMap<String, Vec<u8>>,
    /// Identities from least- to most-recently-used.
    order: VecDeque<String>,
    resident_bytes: u64,
    peak_bytes: u64,
    hits: u64,
    misses: u64,
    evictions: u64,
    uncached: u64,
}

impl ObjectCache {
    /// Open an empty cache over `store` with a resident-byte capacity.
    pub fn new(store: ArtifactStore, capacity_bytes: u64) -> Self {
        ObjectCache {
            store,
            capacity_bytes,
            entries: HashMap::new(),
            order: VecDeque::new(),
            resident_bytes: 0,
            peak_bytes: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
            uncached: 0,
        }
    }

    /// The backing content-addressed store.
    pub fn store(&self) -> &ArtifactStore {
        &self.store
    }

    /// Resident cached bytes. Never exceeds the configured capacity.
    pub fn resident_bytes(&self) -> u64 {
        self.resident_bytes
    }

    /// Highest resident bytes observed since construction or `clear`.
    pub fn peak_bytes(&self) -> u64 {
        self.peak_bytes
    }

    /// Warm reads served from memory.
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// Cold loads read from disk into the cache.
    pub fn misses(&self) -> u64 {
        self.misses
    }

    /// Entries evicted to hold the capacity bound.
    pub fn evictions(&self) -> u64 {
        self.evictions
    }

    /// Reads served verified but uncached (object exceeds capacity).
    pub fn uncached(&self) -> u64 {
        self.uncached
    }

    /// Drop all entries. Counters for hits, misses, evictions, and bypasses
    /// are retained; residence and peak restart for a new cold measurement.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
        self.resident_bytes = 0;
        self.peak_bytes = 0;
    }

    /// Read the object at `digest`: a warm hit returns the resident copy;
    /// a cold miss loads the verified bytes, evicts as needed, and caches
    /// them. Store verification (digest format, hash match) applies on
    /// every path — the cache never launders it.
    pub fn read(&mut self, digest: &str) -> Result<Vec<u8>, LedgerError> {
        if let Some(bytes) = self.entries.get(digest).cloned() {
            self.touch(digest);
            self.hits += 1;
            return Ok(bytes);
        }
        let bytes = self.store.read(digest)?;
        // Every disk load is a cold load, cached or bypassed.
        self.misses += 1;
        if bytes.len() as u64 > self.capacity_bytes {
            self.uncached += 1;
            return Ok(bytes);
        }
        while self.resident_bytes + bytes.len() as u64 > self.capacity_bytes {
            self.evict_lru();
        }
        self.resident_bytes += bytes.len() as u64;
        self.peak_bytes = self.peak_bytes.max(self.resident_bytes);
        self.entries.insert(digest.to_string(), bytes.clone());
        self.order.push_back(digest.to_string());
        Ok(bytes)
    }

    /// Move `digest` to the most-recently-used end.
    fn touch(&mut self, digest: &str) {
        if let Some(pos) = self.order.iter().position(|id| id == digest) {
            self.order.remove(pos);
        }
        self.order.push_back(digest.to_string());
    }

    /// Drop the least-recently-used entry. The cache never holds an empty
    /// slot it cannot evict: callers only invoke this while full, and a
    /// full cache always holds at least one entry.
    fn evict_lru(&mut self) {
        if let Some(id) = self.order.pop_front()
            && let Some(bytes) = self.entries.remove(&id)
        {
            self.resident_bytes -= bytes.len() as u64;
            self.evictions += 1;
        }
    }
}

/// Process high-water resident memory in bytes, read from
/// `/proc/self/status` (`VmHWM`). Returns `None` where the platform
/// exposes no such file. The mark never recedes within a process, so a
/// later reading is always greater than or equal to an earlier one.
pub fn process_peak_bytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        let rest = line.strip_prefix("VmHWM:")?;
        let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
        Some(kb * 1024)
    })
}
