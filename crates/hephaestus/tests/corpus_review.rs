//! T-013 Phase 7 fix-cycle tests (pass 1 findings).
//!
//! Transforms replay in verify_span; search spans index raw bytes;
//! supersedes stores the source id; coverage carries inaccessible.
//! R-088: verified source identity plus corrections of prior records
//! (`correct`); R-017/AT-017: an old claim plus its later correction
//! ingest side by side, supersede preserved.

use hephaestus::knowledge::{
    Corpus, EdgeType, ParserId, Span, corpus_coverage, correct, count_origins, ingest_bytes,
    link_edge, search, verify_span,
};

fn parser() -> ParserId {
    ParserId {
        name: "text-plain".to_string(),
        version: "1".to_string(),
    }
}

fn ingest(c: &mut Corpus, locator: &str, bytes: &[u8]) -> u64 {
    ingest_bytes(c, locator, bytes.to_vec(), "2026-10-01", parser())
}

#[test]
fn transforms_replay_against_captured_bytes() {
    let mut c = Corpus::new();
    let id = ingest(&mut c, "inline:case", b"  Hello World  ");
    // Recorded lowercase replays to the claimed text.
    verify_span(
        &c,
        &Span {
            source: id,
            start: 0,
            end: 15,
            text: "  hello world  ".to_string(),
            transforms: vec!["lowercase".to_string()],
        },
    )
    .expect("recorded transform replays");
    // Lying about a transform fails: replay differs.
    assert!(
        verify_span(
            &c,
            &Span {
                source: id,
                start: 0,
                end: 15,
                text: "  Hello World  ".to_string(),
                transforms: vec!["lowercase".to_string()],
            }
        )
        .is_err(),
        "unreplayed transform claim must fail"
    );
    // Unknown transform names fail instead of passing silently.
    assert!(
        verify_span(
            &c,
            &Span {
                source: id,
                start: 0,
                end: 15,
                text: "  Hello World  ".to_string(),
                transforms: vec!["stem-and-embed".to_string()],
            }
        )
        .is_err(),
        "unknown transform must fail"
    );
}

#[test]
fn trim_transform_replays() {
    let mut c = Corpus::new();
    let id = ingest(&mut c, "inline:pad", b"  padded  ");
    verify_span(
        &c,
        &Span {
            source: id,
            start: 0,
            end: 10,
            text: "padded".to_string(),
            transforms: vec!["trim".to_string()],
        },
    )
    .expect("recorded trim replays");
    // All-whitespace slice trims to empty.
    verify_span(
        &c,
        &Span {
            source: id,
            start: 0,
            end: 2,
            text: String::new(),
            transforms: vec!["trim".to_string()],
        },
    )
    .expect("all-whitespace trims to empty");
    // Untrimmed claim against a trim transform fails.
    assert!(
        verify_span(
            &c,
            &Span {
                source: id,
                start: 0,
                end: 10,
                text: "  padded  ".to_string(),
                transforms: vec!["trim".to_string()],
            }
        )
        .is_err(),
        "untrimmed text under trim must fail"
    );
}

#[test]
fn search_spans_index_raw_bytes() {
    let mut c = Corpus::new();
    // Invalid UTF-8 prefix: lossy decoding changes lengths, but hit
    // offsets must still index the captured bytes.
    let mut bytes = vec![0xFF, 0xFE, b'\n'];
    bytes.extend_from_slice(b"clean budget line here");
    let id = ingest(&mut c, "inline:binary", &bytes);
    let (hits, _) = search(&c, "budget");
    assert_eq!(hits.len(), 1, "one line matches, got: {hits:?}");
    let hit = &hits[0];
    let src = c.get(id).expect("source");
    assert_eq!(&src.bytes[hit.start..hit.end], b"clean budget line here");
}

#[test]
fn supersedes_points_at_the_source_id() {
    let mut c = Corpus::new();
    ingest(&mut c, "inline:first", b"original bytes");
    ingest(&mut c, "inline:second", b"other bytes");
    let v2 = correct(&mut c, 0, b"revised bytes".to_vec(), "2026-10-02", parser())
        .expect("correction lands");
    let new = c.get(v2).expect("new version");
    assert_eq!(
        new.supersedes,
        Some(0),
        "supersedes names the source, got: {:?}",
        new.supersedes
    );
    // The pointer resolves to the actual old source.
    let old = c.get(new.supersedes.expect("set")).expect("resolves");
    assert_eq!(old.bytes, b"original bytes");
}

#[test]
fn coverage_reports_inaccessible_as_zero() {
    let mut c = Corpus::new();
    ingest(&mut c, "inline:a", b"some text");
    let (_, coverage) = search(&c, "text");
    assert_eq!(coverage.inaccessible, 0);
}

#[test]
fn parser_identity_is_stored() {
    let mut c = Corpus::new();
    let id = ingest(&mut c, "inline:p", b"bytes");
    let src = c.get(id).expect("source");
    assert_eq!(src.parser.name, "text-plain");
    assert_eq!(src.parser.version, "1");
}

#[test]
fn ties_break_by_source_then_offset() {
    let mut d = Corpus::new();
    // Two sources, each with two equal-score lines: the only
    // deterministic order is (source asc, start asc) — pinned exactly.
    ingest(
        &mut d,
        "inline:doc-b",
        b"same words first\nsame words second",
    );
    ingest(
        &mut d,
        "inline:doc-a",
        b"same words third\nsame words fourth",
    );
    let (hits, _) = search(&d, "same words");
    assert_eq!(hits.len(), 4, "all four lines match: {hits:?}");
    let order: Vec<(u64, usize)> = hits.iter().map(|h| (h.source, h.start)).collect();
    assert_eq!(
        order,
        vec![(0, 0), (0, 17), (1, 0), (1, 17)],
        "exact (source, offset) sequence: {hits:?}"
    );
}

#[test]
fn all_four_edge_types_link() {
    let mut c = Corpus::new();
    ingest(&mut c, "inline:e", b"evidence material here");
    for edge in [
        EdgeType::Supports,
        EdgeType::Contradicts,
        EdgeType::Mentions,
        EdgeType::SharesOrigin,
    ] {
        link_edge(&mut c, "E-4", 0, 0, 8, edge.clone(), "t").expect("each type links");
    }
    assert_eq!(count_origins(&c, "E-4", &EdgeType::SharesOrigin), 1);
    assert_eq!(corpus_coverage(&c).documents, 1);
}
