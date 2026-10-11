//! AE-05 S1 — context assembly interface (AC13).

use hephaestus::knowledge::{
    ContextError, ContextRequest, Corpus, EvidenceRequirement, ParserId, assemble_intake,
    ingest_bytes,
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
