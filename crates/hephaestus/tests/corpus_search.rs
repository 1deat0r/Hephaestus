//! T-013 ticket 02: deterministic search and coverage reports (R-106/AT-106
//! (distinct retrieval metrics against reference sets — never global
//! completeness), R-015).
//!
//! Red-first seam tests against `search`, `corpus_coverage`.

use hephaestus::knowledge::{Corpus, ParserId, corpus_coverage, ingest_bytes, search};

fn parser() -> ParserId {
    ParserId {
        name: "text-plain".to_string(),
        version: "1".to_string(),
    }
}

fn corpus() -> Corpus {
    let mut c = Corpus::new();
    ingest_bytes(
        &mut c,
        "inline:budgets",
        b"budget envelopes bound reservations and spend".to_vec(),
        "2026-10-01",
        parser(),
    );
    ingest_bytes(
        &mut c,
        "inline:scheduler",
        b"the scheduler reserves budget before every dispatch".to_vec(),
        "2026-10-01",
        parser(),
    );
    ingest_bytes(
        &mut c,
        "inline: unrelated",
        b"nothing relevant in this document at all".to_vec(),
        "2026-10-01",
        parser(),
    );
    c
}

#[test]
fn search_ranks_by_token_overlap() {
    let c = corpus();
    let (hits, _) = search(&c, "budget reservations");
    assert!(
        hits.len() >= 2,
        "both budget documents must hit, got: {hits:?}"
    );
    assert_eq!(hits[0].source, 0, "two-token line outranks one-token line");
    assert_eq!(hits[0].score, 2);
    assert!(hits[0].text.contains("budget"));
}

#[test]
fn search_is_deterministic() {
    let c = corpus();
    let (a, _) = search(&c, "budget dispatch");
    let (b, _) = search(&c, "budget dispatch");
    assert_eq!(a, b, "twin searches must be identical");
}

#[test]
fn empty_results_stay_empty_with_coverage() {
    let c = corpus();
    let (hits, coverage) = search(&c, "zebras orbit quasars");
    assert!(hits.is_empty(), "no match means no hits");
    assert_eq!(coverage.searched, 3);
    assert_eq!(coverage.matched, 0);
    assert_eq!(coverage.spans, 0);
}

#[test]
fn query_coverage_counts_are_exact() {
    let c = corpus();
    let (_, coverage) = search(&c, "budget");
    assert_eq!(coverage.searched, 3);
    assert_eq!(coverage.matched, 2);
    assert_eq!(coverage.spans, 2);
    assert_eq!(coverage.quarantined_excluded, 0);
}

#[test]
fn corpus_coverage_counts_are_exact() {
    let c = corpus();
    let coverage = corpus_coverage(&c);
    assert_eq!(coverage.documents, 3);
    assert_eq!(
        coverage.bytes,
        (b"budget envelopes bound reservations and spend".len()
            + b"the scheduler reserves budget before every dispatch".len()
            + b"nothing relevant in this document at all".len()) as u64
    );
    assert_eq!(coverage.quarantined, 0);
    assert_eq!(coverage.superseded, 0);
}

#[test]
fn views_rebuild_from_captured_bytes_alone() {
    let c = corpus();
    // Rebuild: fresh corpus, same bytes, no derived state carried over.
    let mut rebuilt = Corpus::new();
    for src in &c.sources {
        ingest_bytes(
            &mut rebuilt,
            &src.locator,
            src.bytes.clone(),
            &src.validity_time,
            src.parser.clone(),
        );
    }
    let (a, ca) = search(&c, "budget");
    let (b, cb) = search(&rebuilt, "budget");
    assert_eq!(a, b);
    assert_eq!(ca, cb);
    assert_eq!(corpus_coverage(&c), corpus_coverage(&rebuilt));
}
