//! Prior-art investigation (T-018, R-028 (near-matching mechanism
//! comparison with difference account), R-030 (confidentiality of
//! external queries — redact_query), R-009/AT-009 (a deliberately
//! incomplete source yields no hits and never novelty language),
//! R-106, R-107, R-112/AT-112
//! (known/near-match/scoped-no-match labels + assessed claim chart),
//! MASTER_SPEC §10).
//!
//! Red-first integration tests at the public seam: `verify_span`,
//! `redact_query`, `investigate`, `claim_chart`, labeling.

use hephaestus::priorart::record::{
    AtomicClaim, Conclusion, DisclosureGate, PassageRef, RediscoveryLabel, SearchPlan,
};
use hephaestus::priorart::{claim_chart, investigate, label_for, redact_query, verify_span};

fn s(v: &str) -> String {
    v.to_string()
}

/// Known-mechanism claim (fixture): rediscovery expected.
fn known_claim() -> AtomicClaim {
    AtomicClaim {
        id: s("C1"),
        component: s("LRU cache"),
        mechanism: s("filename mtime invalidation"),
        use_context: s("build systems"),
        regime: s("single machine"),
        claimed_result: s("cache hits without stale reads"),
    }
}

/// Near-mechanism claim: overlaps but differs on the claimed result.
fn near_claim() -> AtomicClaim {
    AtomicClaim {
        id: s("C2"),
        component: s("content cache"),
        mechanism: s("hash keyed invalidation"),
        use_context: s("build systems"),
        regime: s("distributed"),
        claimed_result: s("provably correct incremental rebuilds"),
    }
}

fn corpus(name: &str, body: &str) -> (String, Vec<u8>) {
    (s(name), body.as_bytes().to_vec())
}

/// Adjudicated fixtures (R-106): known corpus, near-match corpus,
/// no-hit corpus.
fn corpora() -> Vec<(String, Vec<u8>)> {
    vec![
        corpus(
            "known",
            "LRU cache with filename mtime invalidation used in build systems; \
             cache hits without stale reads on a single machine.",
        ),
        corpus(
            "near",
            "content cache for distributed build \
             systems; a content-addressed store with lease-based eviction \
             yields provably correct incremental rebuilds across machines.",
        ),
        corpus("empty", "unrelated text about gardening and weather."),
    ]
}

fn plan() -> SearchPlan {
    SearchPlan {
        scopes: vec![s("in-repo-fixture")],
        query_families: vec![s("keyword"), s("synonym")],
        gate: DisclosureGate::default(),
    }
}

// ---- Ticket 01: conclusions, spans, redaction ----

#[test]
fn conclusion_enum_is_the_closed_five() {
    // §10:203: exactly these five, no sixth, no score.
    let all = [
        Conclusion::Known,
        Conclusion::NearMatch,
        Conclusion::NoMatchWithinSearchScope,
        Conclusion::Conflicting,
        Conclusion::Unresolved,
    ];
    assert_eq!(all.len(), 5);
}

#[test]
fn verify_span_passes_on_matching_bytes() {
    let corpora = corpora();
    let reference = PassageRef {
        source: s("known"),
        start: 0,
        end: 9,
        captured_text: s("LRU cache"),
    };
    assert!(
        verify_span(&reference, &corpora),
        "exact byte match verifies"
    );
}

#[test]
fn verify_span_rejects_mismatched_captured_text() {
    let corpora = corpora();
    // Locator-only reference: no captured text (R-107 rejection).
    let locator_only = PassageRef {
        source: s("known"),
        start: 0,
        end: 9,
        captured_text: String::new(),
    };
    assert!(!verify_span(&locator_only, &corpora));
    // Wrong captured text for the range.
    let wrong = PassageRef {
        source: s("known"),
        start: 0,
        end: 9,
        captured_text: s("zzz"),
    };
    assert!(!verify_span(&wrong, &corpora));
    // Out-of-bounds range.
    let oob = PassageRef {
        source: s("known"),
        start: 0,
        end: 99_999,
        captured_text: s("LRU cache"),
    };
    assert!(!verify_span(&oob, &corpora));
}

#[test]
fn redacted_queries_strip_mechanism_tokens() {
    let claim = known_claim();
    let tokens: Vec<&str> = claim.mechanism_tokens();
    let query = "filename mtime invalidation cache strategy";
    let redacted = redact_query(query, &tokens);
    assert!(
        !redacted.contains("invalidation"),
        "mechanism token stripped"
    );
    assert!(redacted.contains("cache"), "non-mechanism token kept");
    // Default gate: disclosure NOT approved.
    assert!(!DisclosureGate::default().full_disclosure_approved);
}

// ---- Ticket 02: two-stage investigation, charts, labels ----

#[test]
fn stage1_flags_known_rediscovery() {
    let report = investigate(&[known_claim()], &corpora(), &plan(), "2026-10-01");
    let (_id, conclusion) = report
        .conclusions
        .iter()
        .find(|(id, _)| id == "C1")
        .expect("C1 conclusion");
    assert_eq!(*conclusion, Conclusion::Known, "rediscovery flagged");
    // R-112 label: rediscovery -> validated_solution.
    assert_eq!(
        label_for(*conclusion, report.charts.as_slice(), "C1"),
        Some(RediscoveryLabel::ValidatedSolution)
    );
}

#[test]
fn stage2_produces_near_match_with_difference_account() {
    let report = investigate(&[near_claim()], &corpora(), &plan(), "2026-10-01");
    let (_id, conclusion) = report
        .conclusions
        .iter()
        .find(|(id, _)| id == "C2")
        .expect("C2 conclusion");
    assert_eq!(*conclusion, Conclusion::NearMatch);
    // Chart exists with rows and a difference account.
    let chart = report
        .charts
        .iter()
        .find(|c| c.claim_id == "C2")
        .expect("C2 chart");
    assert!(!chart.rows.is_empty());
    assert!(
        chart
            .rows
            .iter()
            .any(|r| !r.similar && !r.account.is_empty())
    );
}

#[test]
fn scoped_no_match_never_yields_novelty_language() {
    // Claim about something absent from all corpora.
    let claim = AtomicClaim {
        id: s("C3"),
        component: s("quantum flux"),
        mechanism: s("entangled phase reversal"),
        use_context: s("weather prediction"),
        regime: s("cryogenic"),
        claimed_result: s("predicts storms"),
    };
    let report = investigate(&[claim], &corpora(), &plan(), "2026-10-01");
    let (_, conclusion) = report
        .conclusions
        .iter()
        .find(|(id, _)| id == "C3")
        .expect("C3 conclusion");
    assert_eq!(*conclusion, Conclusion::NoMatchWithinSearchScope);
    // Meta carries scopes + missing access (R-029).
    assert!(!report.meta.scopes_covered.is_empty());
    assert!(!report.meta.query_families.is_empty());
}

#[test]
fn chart_refuses_unverified_span() {
    let corpora = corpora();
    // Passage ref with wrong captured text -> chart not produced.
    let bad_ref = PassageRef {
        source: s("known"),
        start: 0,
        end: 9,
        captured_text: s("WRONG"),
    };
    let claim = known_claim();
    let chart = claim_chart(&claim, bad_ref, &corpora);
    assert!(
        chart.is_none(),
        "chart with an unverified span is refused (R-107)"
    );
}

#[test]
fn validated_candidate_requires_verified_chart() {
    // Near-match with a verified chart + assessed differences -> candidate.
    let report = investigate(&[near_claim()], &corpora(), &plan(), "2026-10-01");
    let chart = report.charts.iter().find(|c| c.claim_id == "C2").unwrap();
    assert!(
        chart.differences_assessed,
        "candidate labeling requires assessed differences"
    );
    assert_eq!(
        label_for(Conclusion::NearMatch, report.charts.as_slice(), "C2"),
        Some(RediscoveryLabel::ValidatedCandidate)
    );
}

#[test]
fn twin_run_byte_identical() {
    let a = investigate(
        &[known_claim(), near_claim()],
        &corpora(),
        &plan(),
        "2026-10-01",
    );
    let b = investigate(
        &[known_claim(), near_claim()],
        &corpora(),
        &plan(),
        "2026-10-01",
    );
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
