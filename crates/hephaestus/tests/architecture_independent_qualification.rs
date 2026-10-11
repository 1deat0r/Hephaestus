//! AE-07 S1 — protect generation and evaluation separation.
//!
//! Generators never reach protected evaluation artifacts.
//! Advisory-only authority stays intact. Qualification gates bind
//! the current artifact version; stale versions refuse.

use hephaestus::security::sealed::{
    AccessError, AccessRequest, ArtifactRef, Role, authorize_access,
};

fn artifact(id: &str, version: i64, protected: bool) -> ArtifactRef {
    ArtifactRef {
        id: id.to_string(),
        version,
        protected,
    }
}

fn req(role: Role, artifact: ArtifactRef, current: i64) -> AccessRequest {
    AccessRequest {
        role,
        artifact,
        current_version: current,
    }
}

#[test]
fn s1_isolation_allows_legitimate_access() {
    // AC 19 positive: evaluators and reviewers read protected
    // current artifacts; generators read unprotected current work.
    assert!(authorize_access(&req(Role::Evaluator, artifact("eval-bundle", 3, true), 3,)).is_ok());
    assert!(authorize_access(&req(Role::Reviewer, artifact("eval-bundle", 3, true), 3,)).is_ok());
    assert!(authorize_access(&req(Role::Generator, artifact("work-draft", 7, false), 7,)).is_ok());
}

#[test]
fn s1_isolation_refuses_generator_and_stale_versions() {
    // AC 19 refusal: generator access to protected artifacts fails
    // with the artifact named, at any version.
    match authorize_access(&req(Role::Generator, artifact("eval-bundle", 3, true), 3)) {
        Err(AccessError::GeneratorDeniedProtected { artifact: id }) => {
            assert_eq!(id, "eval-bundle");
        }
        other => panic!("expected GeneratorDeniedProtected, got: {other:?}"),
    }
    // Stale versions refuse for every role, with versions named.
    for role in [Role::Generator, Role::Evaluator, Role::Reviewer] {
        match authorize_access(&req(role, artifact("work-draft", 6, false), 7)) {
            Err(AccessError::StaleVersion { requested, current }) => {
                assert_eq!((requested, current), (6, 7));
            }
            other => panic!("expected StaleVersion, got: {other:?}"),
        }
    }
}

#[test]
fn s1_isolation_denial_ignores_version_currency() {
    // AC 19 regression: role denial precedes version checks, so a
    // generator never learns whether its version is current; twin
    // calls agree on every outcome.
    let stale = req(Role::Generator, artifact("eval-bundle", 2, true), 3);
    let first = authorize_access(&stale);
    let second = authorize_access(&stale);
    assert_eq!(first, second);
    assert!(matches!(
        first,
        Err(AccessError::GeneratorDeniedProtected { .. })
    ));
}

use hephaestus::knowledge::Span;
/// AE-07 S2 — verify independent evidence checks (R-106, R-107).
/// Claims check against captured spans. Shared origins count once.
/// Disagreements and inconclusive outcomes stay in the record.
use hephaestus::knowledge::{
    ClaimCase, ClaimEvidence, ClaimVerdict, Corpus, EvidenceItem, ParserId, Stance,
    check_claim_evidence, ingest_bytes,
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

fn span(source: u64, start: usize, end: usize, text: &str) -> Span {
    Span {
        source,
        start,
        end,
        text: text.to_string(),
        transforms: Vec::new(),
    }
}

fn item(id: &str, span: Span, stance: Stance) -> EvidenceItem {
    EvidenceItem {
        item_id: id.to_string(),
        span,
        stance,
    }
}

#[test]
fn s2_evidence_supports_and_counts_origins_once() {
    // AC 20 positive: verified spans support the claim; two spans
    // from one source count as one origin.
    let corpus = corpus_two();
    let case = ClaimCase {
        claim_id: "c1".to_string(),
        items: vec![
            item("s1", span(0, 0, 5, "alpha"), Stance::Support),
            item("s2", span(0, 6, 14, "evidence"), Stance::Support),
            item("s3", span(1, 0, 4, "beta"), Stance::Support),
        ],
    };
    let out: ClaimEvidence = check_claim_evidence(&corpus, &case);
    assert_eq!(out.verdict, ClaimVerdict::Supported);
    assert_eq!(out.verified_spans, 3);
    assert_eq!(out.distinct_origins, 2);
    assert!(out.disagreements.is_empty());
    assert!(out.unresolved.is_empty());
}

#[test]
fn s2_evidence_contest_defeats_and_mismatch_stays_unresolved() {
    // AC 20 refusal: a verified contesting span defeats support and
    // stays listed; a fabricated span never counts as support.
    let corpus = corpus_two();
    let case = ClaimCase {
        claim_id: "c2".to_string(),
        items: vec![
            item("s1", span(0, 0, 5, "alpha"), Stance::Support),
            item("d1", span(1, 0, 4, "beta"), Stance::Contest),
            item("f1", span(0, 0, 5, "WRONG"), Stance::Support),
        ],
    };
    let out: ClaimEvidence = check_claim_evidence(&corpus, &case);
    assert_eq!(out.verdict, ClaimVerdict::Contested);
    assert_eq!(out.disagreements, vec!["d1".to_string()]);
    assert_eq!(out.unresolved, vec!["f1".to_string()]);
    // Only verified spans feed the counts.
    assert_eq!(out.verified_spans, 2);
    assert_eq!(out.distinct_origins, 2);
    // Nothing verifies: inconclusive, never support.
    let empty = ClaimCase {
        claim_id: "c3".to_string(),
        items: vec![item("f1", span(0, 0, 5, "WRONG"), Stance::Support)],
    };
    let out: ClaimEvidence = check_claim_evidence(&corpus, &empty);
    assert_eq!(out.verdict, ClaimVerdict::Inconclusive);
    assert_eq!(out.unresolved, vec!["f1".to_string()]);
}

#[test]
fn s2_evidence_replays_deterministically() {
    // AC 20 regression: twin checks agree exactly, including
    // retained disagreements and unresolved items.
    let corpus = corpus_two();
    let case = ClaimCase {
        claim_id: "c4".to_string(),
        items: vec![
            item("s1", span(0, 0, 5, "alpha"), Stance::Support),
            item("d1", span(1, 5, 20, "counterevidence"), Stance::Contest),
            item("f1", span(9, 0, 3, "zzz"), Stance::Support),
        ],
    };
    let first = check_claim_evidence(&corpus, &case);
    let second = check_claim_evidence(&corpus, &case);
    assert_eq!(first, second);
    assert_eq!(first.verdict, ClaimVerdict::Contested);
}

use hephaestus::evalsuite::Envelope;
/// AE-07 S3 — measure qualified efficiency (R-074, R-106).
/// Baseline and candidate compare at matched envelopes. The record
/// carries correctness, false promotion, tokens, cost, latency,
/// throughput, and peak memory. Missing measures stay explicit.
use hephaestus::evalsuite::{
    ArmMeasurement, ComparisonError, RunMeasurement, compare_qualified_efficiency,
};

fn envelope(budget: u64) -> Envelope {
    Envelope {
        compute_budget: budget,
        tool_access: vec!["search".to_string(), "files".to_string()],
        model_access: "none".to_string(),
        tuning_record: "2h".to_string(),
    }
}

fn run(correct: bool, promoted: bool, tokens: u64, cost: f64) -> RunMeasurement {
    RunMeasurement {
        correct,
        promoted,
        tokens,
        cost,
        latency_ms: 100,
        throughput: 10.0,
        peak_memory_bytes: 512,
    }
}

fn arm(id: &str, budget: u64, runs: Vec<RunMeasurement>) -> ArmMeasurement {
    ArmMeasurement {
        arm_id: id.to_string(),
        envelope: envelope(budget),
        runs,
    }
}

#[test]
fn s3_measurement_compares_matched_arms() {
    // AC 21 positive: matched envelopes compare; every recorded
    // dimension aggregates deterministically.
    let baseline = arm(
        "baseline",
        100,
        vec![run(true, false, 10, 1.0), run(false, false, 20, 2.0)],
    );
    let candidate = arm(
        "candidate",
        100,
        vec![run(true, true, 5, 0.5), run(false, true, 5, 0.5)],
    );
    let out = compare_qualified_efficiency(&baseline, &candidate).expect("matched");
    assert_eq!(out.baseline_id, "baseline");
    assert_eq!(out.candidate_id, "candidate");
    assert!((out.baseline_correctness - 0.5).abs() < 1e-12);
    assert!((out.candidate_correctness - 0.5).abs() < 1e-12);
    // The candidate promoted an incorrect run: one false promotion.
    assert_eq!(out.false_promotions, 1);
    assert_eq!(out.baseline_tokens, 30);
    assert_eq!(out.candidate_tokens, 10);
    assert!((out.candidate_cost - 1.0).abs() < 1e-12);
    assert_eq!(out.candidate_latency_ms, 200);
    assert!((out.candidate_throughput - 10.0).abs() < 1e-12);
    assert_eq!(out.candidate_peak_memory_bytes, 512);
    assert_eq!(out.baseline_peak_memory_bytes, 512);
}

#[test]
fn s3_measurement_refuses_unmatched_and_invalid() {
    // AC 21 refusal: unmatched envelopes never compare.
    let baseline = arm("baseline", 100, vec![run(true, false, 10, 1.0)]);
    let richer = arm("candidate", 200, vec![run(true, true, 10, 1.0)]);
    assert!(matches!(
        compare_qualified_efficiency(&baseline, &richer),
        Err(ComparisonError::UnmatchedEnvelopes { .. })
    ));
    // Empty runs measure nothing.
    let empty = arm("candidate", 100, vec![]);
    assert!(matches!(
        compare_qualified_efficiency(&baseline, &empty),
        Err(ComparisonError::EmptyRuns { .. })
    ));
    // Unusable measures refuse with the reason named.
    let mut bad = run(true, true, 10, 1.0);
    bad.cost = f64::NAN;
    let bad_arm = arm("candidate", 100, vec![bad]);
    assert!(matches!(
        compare_qualified_efficiency(&baseline, &bad_arm),
        Err(ComparisonError::InvalidMeasurement { .. })
    ));
}

#[test]
fn s3_measurement_ignores_run_order() {
    // AC 21 regression: run order never changes the comparison;
    // twin calls agree exactly.
    let baseline = arm("baseline", 100, vec![run(true, false, 10, 1.0)]);
    let runs_a = vec![run(true, true, 5, 0.5), run(false, false, 15, 1.5)];
    let mut runs_b = runs_a.clone();
    runs_b.reverse();
    let first =
        compare_qualified_efficiency(&baseline, &arm("candidate", 100, runs_a)).expect("matched");
    let second =
        compare_qualified_efficiency(&baseline, &arm("candidate", 100, runs_b)).expect("matched");
    assert_eq!(first, second);
    assert_eq!(first.false_promotions, 0);
}
