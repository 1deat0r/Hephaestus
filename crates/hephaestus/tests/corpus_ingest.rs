//! T-013 ticket 01: ingest local files, capture bytes, verify spans (R-017, R-107).
//!
//! Red-first seam tests against `hephaestus::knowledge::{...}`.

use hephaestus::knowledge::{
    Corpus, LocalFileAdapter, ParserId, SourceAdapter, Span, ingest_bytes, sha256_hex, verify_span,
};

fn parser() -> ParserId {
    ParserId {
        name: "text-plain".to_string(),
        version: "1".to_string(),
    }
}

#[test]
fn ingest_captures_bytes_and_hash() {
    let mut corpus = Corpus::new();
    let bytes = b"budget envelopes bound reservations".to_vec();
    let id = ingest_bytes(
        &mut corpus,
        "inline:fixture-a",
        bytes.clone(),
        "2026-10-01",
        parser(),
    );
    let src = corpus.get(id).expect("ingested source retrievable");
    assert_eq!(src.bytes, bytes);
    assert_eq!(src.sha256, sha256_hex(&bytes));
    assert_eq!(src.sha256.len(), 64);
    assert_eq!(src.validity_time, "2026-10-01");
    assert!(!src.ingested_at.is_empty());
}

#[test]
fn adapter_reads_real_files_through_the_same_path() {
    let mut corpus = Corpus::new();
    let adapter = LocalFileAdapter::new(parser());
    let id = adapter
        .ingest_file(&mut corpus, "../../GLOSSARY.md", "2026-10-01")
        .expect("adapter ingests repo file");
    let src = corpus.get(id).expect("source retrievable");
    let on_disk = std::fs::read("../../GLOSSARY.md").expect("glossary readable");
    assert_eq!(src.bytes, on_disk);
    assert_eq!(src.locator, "../../GLOSSARY.md");
}

#[test]
fn exact_span_verifies_tampered_span_fails() {
    let mut corpus = Corpus::new();
    let bytes = b"alpha beta gamma".to_vec();
    let id = ingest_bytes(
        &mut corpus,
        "inline:fixture-b",
        bytes,
        "2026-10-01",
        parser(),
    );
    let good = Span {
        source: id,
        start: 6,
        end: 10,
        text: "beta".to_string(),
        transforms: vec![],
    };
    verify_span(&corpus, &good).expect("exact slice verifies");
    for bad in [
        Span {
            source: id,
            start: 6,
            end: 10,
            text: "BETA".to_string(),
            transforms: vec![],
        },
        Span {
            source: id,
            start: 6,
            end: 10,
            text: "betX".to_string(),
            transforms: vec![],
        },
        Span {
            source: id,
            start: 14,
            end: 99,
            text: "past the end".to_string(),
            transforms: vec![],
        },
        Span {
            source: 9999,
            start: 0,
            end: 1,
            text: "a".to_string(),
            transforms: vec![],
        },
    ] {
        assert!(
            verify_span(&corpus, &bad).is_err(),
            "must fail closed: {bad:?}"
        );
    }
}

#[test]
fn spans_verify_against_capture_not_live_files() {
    let dir = std::env::temp_dir();
    let path = dir.join("hephaestus_t013_probe.txt");
    std::fs::write(&path, b"stable captured content").expect("probe written");
    let mut corpus = Corpus::new();
    let adapter = LocalFileAdapter::new(parser());
    let id = adapter
        .ingest_file(&mut corpus, path.to_str().expect("utf8 path"), "2026-10-01")
        .expect("adapter ingests probe");
    // Mutate the live file AFTER ingest: the span must still verify,
    // because verification reads captured bytes, never the file.
    std::fs::write(&path, b"CHANGED live content!!!!").expect("probe mutated");
    let span = Span {
        source: id,
        start: 0,
        end: 6,
        text: "stable".to_string(),
        transforms: vec![],
    };
    verify_span(&corpus, &span).expect("capture-backed span survives live mutation");
    std::fs::remove_file(&path).ok();
}

#[test]
fn twin_ingest_is_byte_identical() {
    let mut a = Corpus::new();
    let mut b = Corpus::new();
    let bytes = b"deterministic capture".to_vec();
    let ida = ingest_bytes(&mut a, "inline:twin", bytes.clone(), "2026-10-01", parser());
    let idb = ingest_bytes(&mut b, "inline:twin", bytes.clone(), "2026-10-01", parser());
    assert_eq!(ida, idb);
    assert_eq!(a.get(ida).expect("a").sha256, b.get(idb).expect("b").sha256);
}
