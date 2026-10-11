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

/// One evidence requirement for a task context: exact source span
/// plus whether the span is mandatory (S2 enforces the budget rule).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRequirement {
    pub id: String,
    pub source: u64,
    pub start: usize,
    pub end: usize,
    pub mandatory: bool,
}

/// Intake request for one bounded task context (AC13): an authorized
/// snapshot, a task statement, evidence requirements, and a token
/// limit. Authorization is caller-evaluated; the flag carries it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextRequest {
    pub snapshot_id: String,
    pub snapshot_authorized: bool,
    pub task: String,
    pub requirements: Vec<EvidenceRequirement>,
    pub token_limit: usize,
}

/// One requirement that resolves to no span, with its cause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingEvidence {
    pub requirement_id: String,
    pub reason: String,
}

/// Intake result: resolved spans with provenance plus explicit
/// missing evidence. `tokens_used` sums resolved span byte lengths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssembledContext {
    pub spans: Vec<Span>,
    pub provenance: Vec<String>,
    pub missing: Vec<MissingEvidence>,
    pub tokens_used: usize,
}

/// Why intake refused. Missing spans never refuse; they list under
/// `missing` so absence stays explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextError {
    UnauthorizedSnapshot,
    EmptyTask,
    EmptyRequirements,
    InvalidTokenLimit,
}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnauthorizedSnapshot => write!(f, "snapshot not authorized"),
            Self::EmptyTask => write!(f, "task is empty"),
            Self::EmptyRequirements => write!(f, "requirements are empty"),
            Self::InvalidTokenLimit => write!(f, "token limit must be nonzero"),
        }
    }
}

impl std::error::Error for ContextError {}

/// Validate intake and resolve requirement spans against the corpus.
/// Quarantined, unknown, and out-of-bounds sources list as missing.
pub fn assemble_intake(
    corpus: &Corpus,
    request: &ContextRequest,
) -> Result<AssembledContext, ContextError> {
    if !request.snapshot_authorized {
        return Err(ContextError::UnauthorizedSnapshot);
    }
    if request.task.is_empty() {
        return Err(ContextError::EmptyTask);
    }
    if request.requirements.is_empty() {
        return Err(ContextError::EmptyRequirements);
    }
    if request.token_limit == 0 {
        return Err(ContextError::InvalidTokenLimit);
    }
    let mut spans = Vec::new();
    let mut provenance = Vec::new();
    let mut missing = Vec::new();
    for req in &request.requirements {
        match corpus.get(req.source) {
            None => missing.push(MissingEvidence {
                requirement_id: req.id.clone(),
                reason: format!("unknown source: {}", req.source),
            }),
            Some(src) if src.quarantined.is_some() => missing.push(MissingEvidence {
                requirement_id: req.id.clone(),
                reason: format!("quarantined source: {}", req.source),
            }),
            Some(src) if req.start > req.end || req.end > src.bytes.len() => {
                missing.push(MissingEvidence {
                    requirement_id: req.id.clone(),
                    reason: format!(
                        "span [{}, {}) outside {} captured bytes",
                        req.start,
                        req.end,
                        src.bytes.len()
                    ),
                });
            }
            Some(src) => {
                let text = String::from_utf8_lossy(&src.bytes[req.start..req.end]).into_owned();
                spans.push(Span {
                    source: req.source,
                    start: req.start,
                    end: req.end,
                    text,
                    transforms: Vec::new(),
                });
                provenance.push(format!(
                    "source:{}:[{}..{})@{}",
                    req.source, req.start, req.end, src.sha256
                ));
            }
        }
    }
    let tokens_used = spans.iter().map(|s| s.text.len()).sum();
    Ok(AssembledContext {
        spans,
        provenance,
        missing,
        tokens_used,
    })
}
