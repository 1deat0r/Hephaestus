//! Corpus ingestion, source-span provenance, retrieval, and evidence
//! edges (T-013, M2 Knowledge service).
//!
//! Implements IMPLEMENTATION_PLAN T-013 over local authorized files:
//! ingestion into immutable captured bytes (R-017), spans verified against
//! those bytes (R-107), deterministic full-text search with coverage (R-106),
//! rebuildable views (R-015), typed evidence edges (R-016) with
//! shared-origin tallies (R-018), and quarantine/corrections (R-098).
//!
//! Scope note: the corpus is in-memory and pure — no I/O except the
//! local-file adapter reading exactly the paths its caller authorizes, no
//! network, no embeddings, no learned components. Callers persist what they
//! accept; retrieval reports coverage, never novelty (R-009 boundary).

pub mod record;
pub mod service;
pub use record::{
    CapturedSource, Corpus, CorpusCoverage, Edge, EdgeError, EdgeType, ParserId, QueryCoverage,
    SearchHit, Span, SpanError,
};
pub use service::{
    AdapterError, LocalFileAdapter, SourceAdapter, corpus_coverage, correct, count_origins,
    ingest_bytes, link_edge, quarantine, search, sha256_hex, tokenize, verify_span,
};
