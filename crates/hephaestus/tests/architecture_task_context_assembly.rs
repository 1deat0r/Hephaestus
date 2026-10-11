//! AE-05 S1 — context assembly interface (AC13).

use hephaestus::knowledge::{
    AssemblyError, CandidateContext, CandidateEdge, ContextCorrectness, ContextError,
    ContextRequest, Corpus, EvidenceRequirement, ParserId, assemble_bounded_context,
    assemble_intake, ingest_bytes, verify_dependency_context,
};

fn parser() -> ParserId {
    ParserId {
        name: "text-plain".to_string(),
        version: "1".to_string(),
    }
}

fn corpus_two() -> Corpus {
    let mut corpus = Corpus::new();
    ingest_bytes(
        &mut corpus,
        "inline:a",
        b"alpha evidence span".to_vec(),
        "2026-10-11",
        parser(),
    );
    ingest_bytes(
        &mut corpus,
        "inline:b",
        b"beta counterevidence here".to_vec(),
        "2026-10-11",
        parser(),
    );
    corpus
}

fn req(id: &str, source: u64, start: usize, end: usize) -> EvidenceRequirement {
    EvidenceRequirement {
        id: id.to_string(),
        source,
        start,
        end,
        mandatory: true,
    }
}

fn request(reqs: Vec<EvidenceRequirement>) -> ContextRequest {
    ContextRequest {
        snapshot_id: "snap-1".to_string(),
        snapshot_authorized: true,
        task: "summarize pressure points".to_string(),
        requirements: reqs,
        token_limit: 1000,
    }
}

#[test]
fn s1_interface_returns_spans_with_provenance() {
    let corpus = corpus_two();
    let ctx = assemble_intake(
        &corpus,
        &request(vec![req("r1", 0, 0, 5), req("r2", 1, 0, 4)]),
    )
    .expect("intake accepts");
    assert_eq!(ctx.spans.len(), 2);
    assert_eq!(ctx.missing.len(), 0);
    assert_eq!(ctx.provenance.len(), 2);
    assert!(ctx.tokens_used > 0);
}

#[test]
fn s1_interface_refuses_unauthorized_and_bad_intake() {
    let corpus = corpus_two();
    let mut bad = request(vec![req("r1", 0, 0, 5)]);
    bad.snapshot_authorized = false;
    assert_eq!(
        assemble_intake(&corpus, &bad),
        Err(ContextError::UnauthorizedSnapshot)
    );
    let mut bad = request(vec![req("r1", 0, 0, 5)]);
    bad.task = "".to_string();
    assert_eq!(assemble_intake(&corpus, &bad), Err(ContextError::EmptyTask));
    let mut bad = request(vec![req("r1", 0, 0, 5)]);
    bad.token_limit = 0;
    assert_eq!(
        assemble_intake(&corpus, &bad),
        Err(ContextError::InvalidTokenLimit)
    );
    assert_eq!(
        assemble_intake(&corpus, &request(vec![])),
        Err(ContextError::EmptyRequirements)
    );
}

#[test]
fn s1_interface_reports_missing_evidence_explicitly() {
    let corpus = corpus_two();
    let ctx = assemble_intake(
        &corpus,
        &request(vec![
            req("good", 0, 0, 5),
            req("unknown", 99, 0, 5),
            req("oob", 0, 0, 9999),
        ]),
    )
    .expect("intake lists missing instead of failing");
    assert_eq!(ctx.spans.len(), 1);
    assert_eq!(ctx.missing.len(), 2);
    let ids: Vec<&str> = ctx
        .missing
        .iter()
        .map(|m| m.requirement_id.as_str())
        .collect();
    assert!(ids.contains(&"unknown"));
    assert!(ids.contains(&"oob"));
}

fn opt_req(id: &str, source: u64, start: usize, end: usize) -> EvidenceRequirement {
    EvidenceRequirement {
        id: id.to_string(),
        source,
        start,
        end,
        mandatory: false,
    }
}

#[test]
fn s2_assembly_deduplicates_identical_spans() {
    // AC 14 positive: twin requirements for one span resolve once.
    let corpus = corpus_two();
    let ctx = assemble_bounded_context(
        &corpus,
        &request(vec![
            req("r1", 0, 0, 5),
            req("r1-dup", 0, 0, 5),
            req("r2", 1, 0, 4),
        ]),
    )
    .expect("assembly accepts");
    assert_eq!(ctx.spans.len(), 2);
    assert_eq!(ctx.dropped_duplicates, 1);
    assert_eq!(ctx.missing.len(), 0);
}

#[test]
fn s2_assembly_retains_counterevidence_and_exact_spans() {
    // AC 14 positive: same text from two sources stays twice;
    // coordinates and bytes stay exact.
    let mut corpus = Corpus::new();
    ingest_bytes(
        &mut corpus,
        "inline:x",
        b"same words".to_vec(),
        "2026-10-11",
        parser(),
    );
    ingest_bytes(
        &mut corpus,
        "inline:y",
        b"same words".to_vec(),
        "2026-10-11",
        parser(),
    );
    let ctx = assemble_bounded_context(
        &corpus,
        &request(vec![req("a", 0, 0, 10), req("b", 1, 0, 10)]),
    )
    .expect("assembly accepts");
    assert_eq!(ctx.spans.len(), 2, "counterevidence never dedupes");
    for span in &ctx.spans {
        let src = corpus.get(span.source).expect("source");
        assert_eq!(&src.bytes[span.start..span.end], span.text.as_bytes());
    }
}

#[test]
fn s2_assembly_refuses_budget_that_omits_mandatory() {
    // AC 14 refusal: a budget below a mandatory span refuses with its id.
    let corpus = corpus_two();
    let mut small = request(vec![req("must", 0, 0, 19)]);
    small.token_limit = 5;
    match assemble_bounded_context(&corpus, &small) {
        Err(AssemblyError::MandatoryOmitted { requirement_id, .. }) => {
            assert_eq!(requirement_id, "must");
        }
        other => panic!("expected MandatoryOmitted, got: {other:?}"),
    }
}

#[test]
fn s2_assembly_omits_optional_over_budget() {
    // AC 14 positive: optional spans past the budget drop silently;
    // mandatory spans stay.
    let corpus = corpus_two();
    let mut tight = request(vec![req("must", 0, 0, 5), opt_req("nice", 1, 0, 24)]);
    tight.token_limit = 8;
    let ctx = assemble_bounded_context(&corpus, &tight).expect("optional drops");
    assert_eq!(ctx.spans.len(), 1);
    assert_eq!(ctx.spans[0].source, 0);
}

fn edge(from: &str, target: &str) -> CandidateEdge {
    CandidateEdge {
        from_module: from.to_string(),
        target: target.to_string(),
        alias: None,
    }
}

fn context(declared: &[&str], edges: Vec<CandidateEdge>) -> CandidateContext {
    CandidateContext {
        declared: declared.iter().map(|s| s.to_string()).collect(),
        edges,
    }
}

#[test]
fn s3_correctness_match_accepts() {
    // AC 15 positive: declared closure equals the resolved graph.
    let verdict = verify_dependency_context(&context(
        &["crate::a", "crate::b"],
        vec![edge("crate::a", "crate::b")],
    ));
    match verdict {
        ContextCorrectness::Matched { cycles } => assert!(cycles.is_empty()),
        other => panic!("expected Matched, got: {other:?}"),
    }
}

#[test]
fn s3_correctness_cycles_report_and_terminate() {
    // AC 15 positive: a two-module cycle terminates and is reported;
    // the verdict still accepts the (completely declared) closure.
    let verdict = verify_dependency_context(&context(
        &["crate::x", "crate::y"],
        vec![edge("crate::x", "crate::y"), edge("crate::y", "crate::x")],
    ));
    match verdict {
        ContextCorrectness::Matched { cycles } => {
            assert_eq!(cycles.len(), 1);
            assert_eq!(
                cycles[0].modules,
                vec!["crate::x".to_string(), "crate::y".to_string()]
            );
        }
        other => panic!("expected Matched with cycle, got: {other:?}"),
    }
}

#[test]
fn s3_correctness_deletion_reports_stale() {
    // AC 15 positive: a declared module with no resolved edge
    // (deleted or renamed away) reports stale, never silently drops.
    let verdict = verify_dependency_context(&context(
        &["crate::a", "crate::b", "crate::gone"],
        vec![edge("crate::a", "crate::b")],
    ));
    match verdict {
        ContextCorrectness::Mismatched { stale, missing, .. } => {
            assert_eq!(stale, vec!["crate::gone".to_string()]);
            assert!(missing.is_empty());
        }
        other => panic!("expected Mismatched stale, got: {other:?}"),
    }
}

#[test]
fn s3_correctness_missing_reports_missing() {
    // AC 15 positive: a resolved module absent from the declaration
    // reports missing.
    let verdict =
        verify_dependency_context(&context(&["crate::a"], vec![edge("crate::a", "crate::b")]));
    match verdict {
        ContextCorrectness::Mismatched { stale, missing, .. } => {
            assert!(stale.is_empty());
            assert_eq!(missing, vec!["crate::b".to_string()]);
        }
        other => panic!("expected Mismatched missing, got: {other:?}"),
    }
}

#[test]
fn s3_correctness_unsupported_blocks_conservatively() {
    // AC 15 refusal: a dynamic import blocks without omission — the
    // verdict names the entry and accepts nothing.
    let verdict = verify_dependency_context(&context(
        &["crate::a"],
        vec![edge("crate::a", "dyn::plugin")],
    ));
    match verdict {
        ContextCorrectness::BlockedUnsupported { entries } => {
            assert_eq!(entries.len(), 1);
            assert!(entries[0].contains("dyn::plugin"));
        }
        other => panic!("expected BlockedUnsupported, got: {other:?}"),
    }
}

#[test]
fn s2_assembly_mandatory_unresolvable_refuses() {
    // AC 14 refusal: a mandatory requirement with no resolvable bytes
    // (out-of-bounds, unknown source) refuses by identity instead of
    // vanishing; optional ghosts still list missing and pass.
    let corpus = corpus_two();
    match assemble_bounded_context(&corpus, &request(vec![req("must-oob", 0, 0, 9999)])) {
        Err(AssemblyError::MandatoryUnresolvable { requirement_id, .. }) => {
            assert_eq!(requirement_id, "must-oob");
        }
        other => panic!("expected MandatoryUnresolvable, got: {other:?}"),
    }
    match assemble_bounded_context(&corpus, &request(vec![req("must-ghost", 77, 0, 5)])) {
        Err(AssemblyError::MandatoryUnresolvable { requirement_id, .. }) => {
            assert_eq!(requirement_id, "must-ghost");
        }
        other => panic!("expected MandatoryUnresolvable, got: {other:?}"),
    }
    let ctx = assemble_bounded_context(
        &corpus,
        &request(vec![req("must", 0, 0, 5), opt_req("nice-ghost", 77, 0, 5)]),
    )
    .expect("optional ghost lists missing");
    assert_eq!(ctx.spans.len(), 1);
    assert_eq!(ctx.missing.len(), 1);
    assert_eq!(ctx.missing[0].requirement_id, "nice-ghost");
}

#[test]
fn s1_interface_quarantined_source_lists_missing() {
    // AC 13 regression: a quarantined source resolves to missing with
    // a named cause, never to a span.
    use hephaestus::knowledge::quarantine;
    let mut corpus = corpus_two();
    quarantine(&mut corpus, 0, "suspect provenance").expect("quarantine");
    let ctx = assemble_intake(&corpus, &request(vec![req("q", 0, 0, 5)]))
        .expect("quarantine lists missing");
    assert!(ctx.spans.is_empty());
    assert_eq!(ctx.missing.len(), 1);
    assert!(ctx.missing[0].reason.contains("quarantined"));
}
