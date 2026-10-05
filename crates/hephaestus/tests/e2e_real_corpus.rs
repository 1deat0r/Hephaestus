//! Real-corpus grounded mission (dev-roadmap ticket 02; Covers AC 2).
//!
//! Ingests a fixed list of repository documents as the authorized
//! corpus, builds span-cited duration records from their real bytes,
//! and asserts the bottleneck operator emits one grounded,
//! evidence-backed opportunity plus twin-run byte identity.

use hephaestus::discovery::{Opportunity, TraceRecord, TraceVal, Validity, analyze};
use hephaestus::knowledge::{Corpus, ParserId, Span, ingest_bytes, search, verify_span};

/// Fixed authorized document list (relative to the crate directory).
const DOCS: &[&str] = &[
    "tests/fixtures/e2e-trace.log",
    "../../GLOSSARY.md",
    "../../MASTER_SPEC.md",
];

fn parser() -> ParserId {
    ParserId {
        name: "text-plain".to_string(),
        version: "1".to_string(),
    }
}

fn ingest_fixed_docs() -> Corpus {
    let mut corpus = Corpus::new();
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for doc in DOCS {
        let bytes = std::fs::read(base.join(doc)).expect("fixed doc readable");
        ingest_bytes(&mut corpus, doc, bytes, "2026-10-05", parser());
    }
    corpus
}

/// Span over the first occurrence of `needle` in `source`.
fn span_of(corpus: &Corpus, source: u64, needle: &str) -> Span {
    let src = corpus.get(source).expect("source present");
    let text = std::str::from_utf8(&src.bytes).expect("doc text");
    let start = text.find(needle).expect("needle present");
    Span {
        source,
        start,
        end: start + needle.len(),
        text: needle.to_string(),
        transforms: vec![],
    }
}

/// Duration records grounded in real corpus bytes: light vs heavy
/// trace lines cited from the ingested trace document.
fn grounded_records(corpus: &Corpus) -> Vec<TraceRecord> {
    let light = span_of(corpus, 0, "step=light dur_ms=1");
    let heavy = span_of(corpus, 0, "step=heavy dur_ms=401");
    verify_span(corpus, &light).expect("light span verifies");
    verify_span(corpus, &heavy).expect("heavy span verifies");
    vec![
        TraceRecord {
            id: "light-1".to_string(),
            kind: "duration".to_string(),
            name: "light".to_string(),
            value: TraceVal::DurationMs(10),
            evidence: Some(light.clone()),
        },
        TraceRecord {
            id: "light-2".to_string(),
            kind: "duration".to_string(),
            name: "light".to_string(),
            value: TraceVal::DurationMs(20),
            evidence: Some(light),
        },
        TraceRecord {
            id: "heavy-1".to_string(),
            kind: "duration".to_string(),
            name: "heavy".to_string(),
            value: TraceVal::DurationMs(800),
            evidence: Some(heavy.clone()),
        },
        TraceRecord {
            id: "heavy-2".to_string(),
            kind: "duration".to_string(),
            name: "heavy".to_string(),
            value: TraceVal::DurationMs(900),
            evidence: Some(heavy),
        },
    ]
}

fn grounded_opportunities() -> Vec<Opportunity> {
    let corpus = ingest_fixed_docs();
    analyze(&corpus, &grounded_records(&corpus)).opportunities
}

#[test]
fn real_corpus_ingest_covers_the_fixed_doc_list() {
    let corpus = ingest_fixed_docs();
    assert_eq!(corpus.sources.len(), DOCS.len());
    for (i, doc) in DOCS.iter().enumerate() {
        assert_eq!(corpus.sources[i as u64 as usize].locator, *doc);
    }
    // The corpus is searchable over real bytes (never novelty).
    let (hits, _) = search(&corpus, "step heavy");
    assert!(!hits.is_empty(), "trace lines are retrievable");
    assert_eq!(hits[0].source, 0);
}

#[test]
fn grounded_records_yield_one_evidence_backed_opportunity() {
    let opps = grounded_opportunities();
    assert_eq!(opps.len(), 1, "one bottleneck: {opps:?}");
    let opp = &opps[0];
    assert_eq!(opp.validity, Validity::EvidenceBacked);
    assert!(!opp.speculative);
    assert!(!opp.evidence_references.is_empty());
    // Every cited span verifies against the captured bytes.
    let corpus = ingest_fixed_docs();
    for span in &opp.evidence_references {
        verify_span(&corpus, span).expect("opportunity span verifies");
    }
}

#[test]
fn twin_missions_are_byte_identical() {
    let a = grounded_opportunities();
    let b = grounded_opportunities();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap(),
        "deterministic mission"
    );
}
