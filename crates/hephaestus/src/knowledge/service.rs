//! Knowledge service (T-013): pure ingest, verify, search, edge, and
//! lifecycle operations over a `Corpus`.
//!
//! Everything here is deterministic: same bytes in, same outputs out. The
//! tokenizer is fixed (lowercase alphanumeric split); ranking is
//! matched-token count with ties broken by (source id, byte offset).

use std::collections::HashSet;

use sha2::{Digest, Sha256};

use super::record::{
    CapturedSource, Corpus, CorpusCoverage, Edge, EdgeError, EdgeType, ParserId, QueryCoverage,
    SearchHit, Span, SpanError,
};

/// Lowercase hex sha256 of raw bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Fixed tokenizer: lowercase, split on non-alphanumeric runs.
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect()
}

/// Ingest raw bytes as a new version-1 source. `validity` is the caller-
/// declared capture instant and doubles as the recorded ingestion time:
/// the corpus keeps no clock, so twin runs stay byte-identical and the
/// caller owns the honesty of the timestamp (R-017).
pub fn ingest_bytes(
    corpus: &mut Corpus,
    locator: &str,
    bytes: Vec<u8>,
    validity: &str,
    parser: ParserId,
) -> u64 {
    ingest_version(corpus, locator, bytes, validity, parser, 1, None)
}

fn ingest_version(
    corpus: &mut Corpus,
    locator: &str,
    bytes: Vec<u8>,
    validity: &str,
    parser: ParserId,
    version: u64,
    supersedes: Option<u64>,
) -> u64 {
    let id = corpus.next_id;
    corpus.next_id += 1;
    let sha256 = sha256_hex(&bytes);
    corpus.sources.push(CapturedSource {
        id,
        locator: locator.to_string(),
        bytes,
        sha256,
        validity_time: validity.to_string(),
        ingested_at: validity.to_string(),
        parser,
        version,
        supersedes,
        quarantined: None,
    });
    id
}

/// Verify a span against captured bytes (R-107). Replays the recorded
/// transforms over the byte slice and compares to the claimed text, so
/// unrecorded transforms fail and unknown names error. Also fails closed
/// on unknown sources, out-of-bounds ranges, byte mismatch, and
/// quarantined sources. Live files are never consulted.
pub fn verify_span(corpus: &Corpus, span: &Span) -> Result<(), SpanError> {
    let src = corpus
        .get(span.source)
        .ok_or(SpanError::UnknownSource(span.source))?;
    if src.quarantined.is_some() {
        return Err(SpanError::QuarantinedSource(span.source));
    }
    if span.start > span.end || span.end > src.bytes.len() {
        return Err(SpanError::OutOfBounds {
            start: span.start,
            end: span.end,
            len: src.bytes.len(),
        });
    }
    let mut current = src.bytes[span.start..span.end].to_vec();
    for transform in &span.transforms {
        match transform.as_str() {
            "lowercase" => current = current.iter().map(|b| b.to_ascii_lowercase()).collect(),
            "trim" => {
                let start = current
                    .iter()
                    .position(|b| !b.is_ascii_whitespace())
                    .unwrap_or(current.len());
                let end = current
                    .iter()
                    .rposition(|b| !b.is_ascii_whitespace())
                    .map(|i| i + 1)
                    .unwrap_or(0);
                current = current[start.min(end)..end].to_vec();
            }
            other => return Err(SpanError::UnknownTransform(other.to_string())),
        }
    }
    if current != span.text.as_bytes() {
        return Err(SpanError::Mismatch);
    }
    Ok(())
}

/// Deterministic full-text search: per-line token overlap. Lines split on
/// raw `b'\n'` so hit offsets always index the captured bytes, even for
/// non-UTF8 captures (each line is decoded lossy for tokens only). A line
/// scores its count of distinct query tokens present. Hits sort by (score
/// desc, source asc, start asc). Quarantined sources are excluded and
/// counted, never searched. Empty results are empty data with coverage —
/// never a claim (R-009).
pub fn search(corpus: &Corpus, query: &str) -> (Vec<SearchHit>, QueryCoverage) {
    let query_tokens: HashSet<String> = tokenize(query).into_iter().collect();
    let mut hits = Vec::new();
    let mut matched_sources = HashSet::new();
    let mut quarantined_excluded = 0usize;
    let mut searched = 0usize;
    for src in &corpus.sources {
        if src.quarantined.is_some() {
            quarantined_excluded += 1;
            continue;
        }
        searched += 1;
        if query_tokens.is_empty() {
            continue;
        }
        let mut offset = 0usize;
        for raw_line in src.bytes.split(|b| *b == b'\n') {
            let line = String::from_utf8_lossy(raw_line);
            let line_tokens: HashSet<String> = tokenize(&line).into_iter().collect();
            let score = query_tokens.intersection(&line_tokens).count();
            if score > 0 {
                matched_sources.insert(src.id);
                hits.push(SearchHit {
                    source: src.id,
                    start: offset,
                    end: offset + raw_line.len(),
                    text: line.into_owned(),
                    score,
                });
            }
            offset += raw_line.len() + 1;
        }
    }
    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.source.cmp(&b.source))
            .then_with(|| a.start.cmp(&b.start))
    });
    let spans = hits.len();
    let coverage = QueryCoverage {
        searched,
        matched: matched_sources.len(),
        spans,
        quarantined_excluded,
        inaccessible: 0,
    };
    (hits, coverage)
}

/// Per-corpus coverage: documents, bytes, quarantined, superseded counts.
pub fn corpus_coverage(corpus: &Corpus) -> CorpusCoverage {
    CorpusCoverage {
        documents: corpus.sources.len(),
        bytes: corpus.sources.iter().map(|s| s.bytes.len() as u64).sum(),
        quarantined: corpus
            .sources
            .iter()
            .filter(|s| s.quarantined.is_some())
            .count(),
        superseded: corpus
            .sources
            .iter()
            .filter(|s| s.supersedes.is_some())
            .count(),
    }
}

/// Link a typed evidence edge to span coordinates (R-016). Refuses unknown
/// sources, out-of-bounds ranges, and already-quarantined sources.
pub fn link_edge(
    corpus: &mut Corpus,
    evidence_id: &str,
    source: u64,
    start: usize,
    end: usize,
    edge: EdgeType,
    provenance: &str,
) -> Result<u64, EdgeError> {
    let src = corpus.get(source).ok_or(EdgeError::UnknownSource(source))?;
    if src.quarantined.is_some() {
        return Err(EdgeError::QuarantinedSource(source));
    }
    if start > end || end > src.bytes.len() {
        return Err(EdgeError::OutOfBounds {
            start,
            end,
            len: src.bytes.len(),
        });
    }
    let id = corpus.next_id;
    corpus.next_id += 1;
    corpus.edges.push(Edge {
        id,
        evidence_id: evidence_id.to_string(),
        source,
        start,
        end,
        edge,
        provenance: provenance.to_string(),
        invalidated: false,
    });
    Ok(id)
}

/// Count distinct origins behind live edges of one type for one evidence
/// record (R-018): two spans from the same source cast a single vote.
pub fn count_origins(corpus: &Corpus, evidence_id: &str, edge: &EdgeType) -> usize {
    corpus
        .edges
        .iter()
        .filter(|e| e.evidence_id == *evidence_id && e.edge == *edge && !e.invalidated)
        .map(|e| e.source)
        .collect::<HashSet<_>>()
        .len()
}

/// Quarantine a source with reason (R-098): flags the source and flips
/// every incident edge to invalidated. Bytes are retained for audit.
pub fn quarantine(corpus: &mut Corpus, source: u64, reason: &str) -> Result<(), EdgeError> {
    let slot = corpus
        .sources
        .iter_mut()
        .find(|s| s.id == source)
        .ok_or(EdgeError::UnknownSource(source))?;
    slot.quarantined = Some(reason.to_string());
    for edge in corpus.edges.iter_mut().filter(|e| e.source == source) {
        edge.invalidated = true;
    }
    Ok(())
}

/// Correct a source: mints a new version whose `supersedes` names the old
/// source id (R-017), so the pointer always resolves via `get`. The old
/// bytes stay verifiable; quarantine does not transfer.
pub fn correct(
    corpus: &mut Corpus,
    source: u64,
    new_bytes: Vec<u8>,
    validity: &str,
    parser: ParserId,
) -> Result<u64, EdgeError> {
    let (locator, version) = {
        let src = corpus.get(source).ok_or(EdgeError::UnknownSource(source))?;
        (src.locator.clone(), src.version)
    };
    Ok(ingest_version(
        corpus,
        &locator,
        new_bytes,
        validity,
        parser,
        version + 1,
        Some(source),
    ))
}

/// Why adapter ingestion refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    Io(String),
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "adapter read failed: {e}"),
        }
    }
}

impl std::error::Error for AdapterError {}

/// A source of real bytes. Authorization of *which* paths may be read
/// belongs to the caller: the adapter reads exactly what it is handed.
pub trait SourceAdapter {
    fn ingest_file(
        &self,
        corpus: &mut Corpus,
        path: &str,
        validity: &str,
    ) -> Result<u64, AdapterError>;
}

/// Local-file adapter: real authorized files through the real ingest path.
/// No network, no synthesis (T-013 "local authorized files" + one real
/// adapter, rule-6(d) consent absent for off-machine reads).
pub struct LocalFileAdapter {
    pub parser: ParserId,
}

impl LocalFileAdapter {
    pub fn new(parser: ParserId) -> Self {
        Self { parser }
    }
}

impl SourceAdapter for LocalFileAdapter {
    fn ingest_file(
        &self,
        corpus: &mut Corpus,
        path: &str,
        validity: &str,
    ) -> Result<u64, AdapterError> {
        let bytes = std::fs::read(path).map_err(|e| AdapterError::Io(format!("{path}: {e}")))?;
        Ok(ingest_bytes(
            corpus,
            path,
            bytes,
            validity,
            self.parser.clone(),
        ))
    }
}

/// Evidence-class attestation (R-007/AT-007, MASTER_SPEC §3,
/// Verification service): an Observation must be grounded in a source
/// span — repetition of a model statement is seen and recorded in the
/// rejection, never a promotion path. Other declared kinds stand
/// ungrounded: they are what they are, however often they re-appear.
pub fn attest_evidence_class(
    declared: crate::contracts::generated::EvidenceEvidenceType,
    ingestion: &super::record::StatementIngestion,
) -> Result<(), super::record::ClassifyError> {
    use crate::contracts::generated::EvidenceEvidenceType as K;
    if declared == K::Observation && ingestion.grounded_span.is_none() {
        return Err(super::record::ClassifyError::UngroundedObservation {
            ingestions: ingestion.ingestions,
        });
    }
    Ok(())
}
