//! Knowledge records (T-013): captured sources, spans, edges, coverage.
//!
//! A `Corpus` owns immutable captured bytes. Every derived view — search
//! hits, coverage reports, support tallies — rebuilds deterministically
//! from those bytes (R-015). Corrections supersede, quarantine flags;
//! neither mutates originals.

use serde::{Deserialize, Serialize};

/// Parser identity: name + version. Span coordinates are only meaningful
/// together with the parser and transform list that produced them (R-107).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParserId {
    pub name: String,
    pub version: String,
}

/// One ingested document: immutable captured bytes plus provenance.
/// `version` starts at 1; corrections mint higher versions with `supersedes`
/// set. The old bytes stay for audit (R-017).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapturedSource {
    pub id: u64,
    pub locator: String,
    pub bytes: Vec<u8>,
    pub sha256: String,
    pub validity_time: String,
    pub ingested_at: String,
    pub parser: ParserId,
    pub version: u64,
    pub supersedes: Option<u64>,
    /// Quarantine reason when set (R-098). Bytes are retained; views
    /// exclude the source and incident edges flip to invalidated.
    pub quarantined: Option<String>,
}

/// A byte-offset span into a source's captured bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub source: u64,
    pub start: usize,
    pub end: usize,
    pub text: String,
    /// Parser transformations replayed by verification between raw bytes
    /// and `text`, in order. Supported: `lowercase` (ASCII), `trim`
    /// (ASCII whitespace). Empty means identity. Unknown names fail;
    /// unrecorded transforms fail because the replay will not match.
    pub transforms: Vec<String>,
}

/// Why span verification refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanError {
    UnknownSource(u64),
    OutOfBounds {
        start: usize,
        end: usize,
        len: usize,
    },
    /// Slice of captured bytes differs from the claimed text (after
    /// replaying the recorded transforms).
    Mismatch,
    /// A transform name outside the supported replay vocabulary.
    UnknownTransform(String),
    /// Source is quarantined: spans no longer attest anything.
    QuarantinedSource(u64),
}

impl std::fmt::Display for SpanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownSource(id) => write!(f, "unknown source: {id}"),
            Self::OutOfBounds { start, end, len } => {
                write!(f, "span [{start},{end}) outside {len} captured bytes")
            }
            Self::Mismatch => write!(f, "span text differs from captured bytes"),
            Self::UnknownTransform(t) => write!(f, "unsupported transform: {t}"),
            Self::QuarantinedSource(id) => write!(f, "source quarantined: {id}"),
        }
    }
}

impl std::error::Error for SpanError {}

/// Typed evidence edge (R-016): binds an evidence id to span coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    Supports,
    Contradicts,
    Mentions,
    SharesOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub id: u64,
    pub evidence_id: String,
    pub source: u64,
    pub start: usize,
    pub end: usize,
    pub edge: EdgeType,
    pub provenance: String,
    /// Set by quarantine of the source (R-098). Invalidated edges are
    /// retained for audit but excluded from tallies.
    pub invalidated: bool,
}

/// Why edge linking refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeError {
    UnknownSource(u64),
    OutOfBounds {
        start: usize,
        end: usize,
        len: usize,
    },
    QuarantinedSource(u64),
}

impl std::fmt::Display for EdgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownSource(id) => write!(f, "unknown source: {id}"),
            Self::OutOfBounds { start, end, len } => {
                write!(f, "edge [{start},{end}) outside {len} captured bytes")
            }
            Self::QuarantinedSource(id) => write!(f, "source quarantined: {id}"),
        }
    }
}

impl std::error::Error for EdgeError {}

/// One ranked search hit: the matching line with its byte span.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchHit {
    pub source: u64,
    pub start: usize,
    pub end: usize,
    pub text: String,
    /// Distinct query tokens present in this line.
    pub score: usize,
}

/// Per-query coverage (R-106): what the search saw and skipped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryCoverage {
    pub searched: usize,
    pub matched: usize,
    pub spans: usize,
    pub quarantined_excluded: usize,
    /// Sources the adapter could not read at all. In-memory search never
    /// produces these (always 0 here); future adapters (network) report
    /// them, and readiness policy branches on the count.
    pub inaccessible: usize,
}

/// Per-corpus coverage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorpusCoverage {
    pub documents: usize,
    pub bytes: u64,
    pub quarantined: usize,
    pub superseded: usize,
}

/// The corpus: owner of captured bytes plus derived edges.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Corpus {
    pub sources: Vec<CapturedSource>,
    pub edges: Vec<Edge>,
    pub next_id: u64,
}

impl Corpus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, id: u64) -> Option<&CapturedSource> {
        self.sources.iter().find(|s| s.id == id)
    }
}

/// One statement as the verification service sees it (R-007): its
/// text, how often it has been ingested (repetition is DATA, never a
/// promotion path), and whether a grounded source span backs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementIngestion {
    pub text: String,
    pub ingestions: usize,
    pub grounded_span: Option<Span>,
}

/// Why evidence-class attestation refused (typed-rejection
/// convention, R-007/AT-007).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassifyError {
    /// Declared Observation without a grounded span — the seen
    /// repetition count is recorded in the rejection precisely because
    /// it must never be the basis of promotion.
    UngroundedObservation { ingestions: usize },
}
