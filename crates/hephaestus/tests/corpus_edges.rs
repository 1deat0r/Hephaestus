//! T-013 ticket 03: typed edges, shared-origin tallies, quarantine and
//! corrections (R-016, R-018/AT-018 (three papers on one dataset: shared
//! origins count once — count_origins), R-098).

use hephaestus::knowledge::{
    Corpus, EdgeType, ParserId, corpus_coverage, correct, count_origins, ingest_bytes, link_edge,
    quarantine, search,
};

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
        "inline:doc-a",
        b"reserves bound spending before dispatch".to_vec(),
        "2026-10-01",
        parser(),
    );
    ingest_bytes(
        &mut c,
        "inline:doc-b",
        b"dispatch commits reservations exactly once".to_vec(),
        "2026-10-01",
        parser(),
    );
    c
}

#[test]
fn edges_link_evidence_to_spans() {
    let mut c = corpus();
    let id = link_edge(&mut c, "E-1", 0, 0, 8, EdgeType::Supports, "test")
        .expect("edge links to a live source");
    let edge = c.edges.iter().find(|e| e.id == id).expect("edge stored");
    assert_eq!(edge.evidence_id, "E-1");
    assert!(!edge.invalidated);
    // Out-of-bounds and unknown sources refuse.
    assert!(link_edge(&mut c, "E-1", 0, 0, 9999, EdgeType::Supports, "t").is_err());
    assert!(link_edge(&mut c, "E-1", 4242, 0, 1, EdgeType::Supports, "t").is_err());
}

#[test]
fn tally_counts_distinct_origins_once() {
    let mut c = corpus();
    // Two spans, one source: a single support vote (R-018).
    link_edge(&mut c, "E-1", 0, 0, 8, EdgeType::Supports, "t").expect("edge a");
    link_edge(&mut c, "E-1", 0, 9, 15, EdgeType::Supports, "t").expect("edge b");
    assert_eq!(count_origins(&c, "E-1", &EdgeType::Supports), 1);
    // A second origin votes separately.
    link_edge(&mut c, "E-1", 1, 0, 8, EdgeType::Supports, "t").expect("edge c");
    assert_eq!(count_origins(&c, "E-1", &EdgeType::Supports), 2);
}

#[test]
fn contradicting_origins_tally_separately() {
    let mut c = corpus();
    link_edge(&mut c, "E-2", 0, 0, 8, EdgeType::Contradicts, "t").expect("edge a");
    link_edge(&mut c, "E-2", 1, 0, 8, EdgeType::Contradicts, "t").expect("edge b");
    assert_eq!(count_origins(&c, "E-2", &EdgeType::Contradicts), 2);
    assert_eq!(count_origins(&c, "E-2", &EdgeType::Supports), 0);
}

#[test]
fn quarantine_invalidates_edges_and_excludes_views() {
    let mut c = corpus();
    link_edge(&mut c, "E-1", 0, 0, 8, EdgeType::Supports, "t").expect("edge a");
    link_edge(&mut c, "E-1", 1, 0, 8, EdgeType::Supports, "t").expect("edge b");
    assert_eq!(count_origins(&c, "E-1", &EdgeType::Supports), 2);
    quarantine(&mut c, 0, "planted instruction payload").expect("quarantine lands");
    // Incident edge invalidated; tally drops to the surviving origin.
    assert_eq!(count_origins(&c, "E-1", &EdgeType::Supports), 1);
    assert!(
        c.edges
            .iter()
            .find(|e| e.source == 0)
            .expect("edge retained")
            .invalidated,
        "quarantined edges are flagged, not deleted"
    );
    // Search and coverage exclude the source.
    let (hits, coverage) = search(&c, "reserves");
    assert!(hits.iter().all(|h| h.source != 0));
    assert_eq!(coverage.quarantined_excluded, 1);
    assert_eq!(corpus_coverage(&c).quarantined, 1);
    // Linking to a quarantined source refuses.
    assert!(link_edge(&mut c, "E-9", 0, 0, 1, EdgeType::Mentions, "t").is_err());
}

#[test]
fn correction_supersedes_without_mutating() {
    let mut c = corpus();
    let v2 = correct(
        &mut c,
        0,
        b"reserves bound spending before dispatch, revised".to_vec(),
        "2026-10-02",
        parser(),
    )
    .expect("correction mints a version");
    assert_ne!(v2, 0);
    let (old, new) = (
        c.get(0).expect("old retained"),
        c.get(v2).expect("new stored"),
    );
    assert_eq!(old.version, 1);
    assert_eq!(new.version, 2);
    assert_eq!(new.supersedes, Some(0));
    assert_eq!(corpus_coverage(&c).superseded, 1);
    // Old bytes still verify.
    hephaestus::knowledge::verify_span(
        &c,
        &hephaestus::knowledge::Span {
            source: 0,
            start: 0,
            end: 8,
            text: "reserves".to_string(),
            transforms: vec![],
        },
    )
    .expect("original bytes stay verifiable");
}
