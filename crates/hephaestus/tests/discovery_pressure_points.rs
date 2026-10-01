//! Discovery pressure-point operators (T-014, R-019/020/021).
//!
//! Integration tests at the public seam: `discovery::analyze(&Corpus,
//! &[TraceRecord]) -> AnalysisOutcome`. Every emitted opportunity carries
//! evidence or explicit uncertainty; every rejection carries a reason.

use hephaestus::discovery::{PressureKind, TraceRecord, TraceVal, analyze};
use hephaestus::knowledge::{ParserId, Span, ingest_bytes};

/// Build a corpus with one captured source containing `text`, returning
/// (corpus, source_id, byte length).
fn corpus_with(text: &str) -> (hephaestus::knowledge::Corpus, u64) {
    let mut corpus = hephaestus::knowledge::Corpus::new();
    let src = ingest_bytes(
        &mut corpus,
        "trace.log",
        text.as_bytes().to_vec(),
        "2026-10-01T00:00:00Z",
        ParserId {
            name: "trace".into(),
            version: "1".into(),
        },
    );
    (corpus, src)
}

/// Span helper: byte offsets into the captured text.
fn span(source: u64, start: usize, end: usize) -> Span {
    Span {
        source,
        start,
        end,
        text: String::new(),
        transforms: vec![],
    }
}

#[test]
fn hidden_bottleneck_becomes_cited_opportunity() {
    let text = "step=parse dur_ms=120\nstep=render dur_ms=2\nstep=parse dur_ms=131\nstep=render dur_ms=1\n";
    let (corpus, src) = corpus_with(text);
    // The parse steps are the bottleneck; each trace record binds to the
    // span of its line in the captured source.
    let records = vec![
        TraceRecord {
            id: "t1".into(),
            kind: "duration".into(),
            name: "parse".into(),
            value: TraceVal::DurationMs(120),
            evidence: Some(span(src, 0, 21)),
        },
        TraceRecord {
            id: "t2".into(),
            kind: "duration".into(),
            name: "render".into(),
            value: TraceVal::DurationMs(2),
            evidence: Some(span(src, 22, 42)),
        },
        TraceRecord {
            id: "t3".into(),
            kind: "duration".into(),
            name: "parse".into(),
            value: TraceVal::DurationMs(131),
            evidence: Some(span(src, 43, 65)),
        },
        TraceRecord {
            id: "t4".into(),
            kind: "duration".into(),
            name: "render".into(),
            value: TraceVal::DurationMs(1),
            evidence: Some(span(src, 66, 88)),
        },
    ];
    let outcome = analyze(&corpus, &records);
    assert_eq!(outcome.opportunities.len(), 1, "exactly one bottleneck");
    let opp = &outcome.opportunities[0];
    assert_eq!(opp.kind, PressureKind::Bottleneck);
    assert_eq!(opp.suspected_bottleneck.as_deref(), Some("parse"));
    // Evidence must be present and cite the slow span.
    assert!(!opp.evidence_references.is_empty(), "AT-019: cited");
    assert!(opp.evidence_references.iter().any(|s| s.source == src));
    // Validity is evidence-backed, not speculative.
    assert!(!matches!(
        opp.validity,
        hephaestus::discovery::Validity::Speculative
    ));
    assert!(
        outcome.rejected.is_empty(),
        "honest empty: nothing rejected"
    );
}

#[test]
fn no_bottleneck_emits_nothing() {
    let text = "step=a dur_ms=10\nstep=a dur_ms=11\n";
    let (corpus, src) = corpus_with(text);
    let records = vec![
        TraceRecord {
            id: "t1".into(),
            kind: "duration".into(),
            name: "a".into(),
            value: TraceVal::DurationMs(10),
            evidence: Some(span(src, 0, 17)),
        },
        TraceRecord {
            id: "t2".into(),
            kind: "duration".into(),
            name: "a".into(),
            value: TraceVal::DurationMs(11),
            evidence: Some(span(src, 18, 35)),
        },
    ];
    let outcome = analyze(&corpus, &records);
    assert!(outcome.opportunities.is_empty());
    assert!(outcome.rejected.is_empty());
}

#[test]
fn twin_runs_byte_identical() {
    let text = "step=parse dur_ms=120\nstep=render dur_ms=2\nstep=parse dur_ms=131\n";
    let (corpus, src) = corpus_with(text);
    let make = || {
        vec![
            TraceRecord {
                id: "t1".into(),
                kind: "duration".into(),
                name: "parse".into(),
                value: TraceVal::DurationMs(120),
                evidence: Some(span(src, 0, 21)),
            },
            TraceRecord {
                id: "t2".into(),
                kind: "duration".into(),
                name: "render".into(),
                value: TraceVal::DurationMs(2),
                evidence: Some(span(src, 22, 42)),
            },
            TraceRecord {
                id: "t3".into(),
                kind: "duration".into(),
                name: "parse".into(),
                value: TraceVal::DurationMs(131),
                evidence: Some(span(src, 43, 65)),
            },
        ]
    };
    let a = analyze(&corpus, &make());
    let b = analyze(&corpus, &make());
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap(),
        "determinism"
    );
}

#[test]
fn unevidenced_trace_records_are_not_bottleneck_sources() {
    // Records without span evidence cannot ground a bottleneck: the
    // operator must not fabricate citations. Unevidenced records are
    // skipped (uncertainty is explicit, never fabricated evidence).
    let text = "step=parse dur_ms=120\n";
    let (corpus, _src) = corpus_with(text);
    let records = vec![TraceRecord {
        id: "t1".into(),
        kind: "duration".into(),
        name: "parse".into(),
        value: TraceVal::DurationMs(999_999),
        evidence: None,
    }];
    let outcome = analyze(&corpus, &records);
    assert!(
        outcome.opportunities.is_empty(),
        "no evidence -> no emitted bottleneck"
    );
}

// ---- Ticket 02: anomaly operator with the unit gate (R-020/AT-020) ----

use hephaestus::discovery::TraceVal::DurationMs;

fn prediction(id: &str, v: u64, evidence: Option<Span>) -> TraceRecord {
    TraceRecord {
        id: id.into(),
        kind: "prediction".into(),
        name: "latency".into(),
        value: DurationMs(v),
        evidence,
    }
}

fn observation(id: &str, v: u64, evidence: Option<Span>) -> TraceRecord {
    TraceRecord {
        id: id.into(),
        kind: "observation".into(),
        name: "latency".into(),
        value: DurationMs(v),
        evidence,
    }
}

#[test]
fn genuine_discrepancy_becomes_anomaly_opportunity_with_uncertainty() {
    let text = "predicted latency 50ms; observed latency 500ms under identical load\n";
    let (corpus, src) = corpus_with(text);
    let records = vec![
        prediction("p1", 50, Some(span(src, 0, 22))),
        observation("o1", 500, Some(span(src, 23, 62))),
    ];
    let outcome = analyze(&corpus, &records);
    assert_eq!(outcome.opportunities.len(), 1);
    let opp = &outcome.opportunities[0];
    assert_eq!(opp.kind, PressureKind::Anomaly);
    // Uncertainty must be explicit, not two bare numbers.
    assert!(
        !opp.causal_uncertainty.is_empty(),
        "anomaly states its uncertainty"
    );
    assert!(!opp.evidence_references.is_empty());
    assert!(outcome.rejected.is_empty());
}

#[test]
fn unit_mismatch_is_rejected_not_emitted() {
    // AT-020 negative case: prediction in ms, "observation" in seconds
    // stored as a raw count — incompatible units must never form an
    // anomaly.
    let text = "prediction 50; observation 5\n";
    let (corpus, src) = corpus_with(text);
    let mut p = prediction("p1", 50, Some(span(src, 0, 13)));
    p.value = TraceVal::DurationMs(50);
    let mut o = observation("o1", 5, Some(span(src, 14, 28)));
    o.value = TraceVal::Count(5); // different unit, same trace
    let outcome = analyze(&corpus, &[p, o]);
    assert!(
        outcome.opportunities.is_empty(),
        "unit mismatch must not emit"
    );
    assert_eq!(outcome.rejected.len(), 1);
    assert!(
        outcome.rejected[0].reason.contains("unit"),
        "rejection reason names the unit gate: {}",
        outcome.rejected[0].reason
    );
}

#[test]
fn missing_prediction_is_rejected() {
    // "Two isolated numbers from different conditions" is not an anomaly
    // (MASTER_SPEC:144): an observation with no prediction record.
    let text = "observed latency 500ms\n";
    let (corpus, src) = corpus_with(text);
    let records = vec![observation("o1", 500, Some(span(src, 0, 22)))];
    let outcome = analyze(&corpus, &records);
    assert!(outcome.opportunities.is_empty());
    assert_eq!(outcome.rejected.len(), 1);
    assert!(
        outcome.rejected[0].reason.contains("prediction"),
        "rejection names the missing prediction: {}",
        outcome.rejected[0].reason
    );
}

#[test]
fn incomparable_conditions_are_rejected() {
    // Same unit family but the observation measures a DIFFERENT quantity
    // than any prediction — no name-match pairing exists, so the pair is
    // refused rather than force-paired (MASTER_SPEC:144).
    let text = "predicted under cold cache; observed under warm cache\n";
    let (corpus, src) = corpus_with(text);
    let mut p = prediction("p1", 50, Some(span(src, 0, 27)));
    p.name = "latency@cold".into();
    let mut o = observation("o1", 500, Some(span(src, 28, 53)));
    o.name = "latency@warm".into();
    let outcome = analyze(&corpus, &[p, o]);
    assert!(outcome.opportunities.is_empty());
    assert_eq!(outcome.rejected.len(), 1);
    assert!(
        outcome.rejected[0].reason.contains("prediction"),
        "unmatched quantity is refused, not force-paired: {}",
        outcome.rejected[0].reason
    );
}

#[test]
fn within_uncertainty_discrepancy_is_not_an_anomaly() {
    // Observation within the stated uncertainty band: no opportunity, no
    // rejection (the comparison simply holds).
    let text = "predicted latency 50ms; observed latency 55ms\n";
    let (corpus, src) = corpus_with(text);
    let records = vec![
        prediction("p1", 50, Some(span(src, 0, 22))),
        observation("o1", 55, Some(span(src, 23, 44))),
    ];
    let outcome = analyze(&corpus, &records);
    assert!(outcome.opportunities.is_empty(), "no material discrepancy");
    assert!(
        outcome.rejected.is_empty(),
        "comparison held, nothing rejected"
    );
}

// ---- Ticket 03: conflicting objectives + failure patterns ----

fn objective(id: &str, name: &str, v: f64, evidence: Option<Span>) -> TraceRecord {
    TraceRecord {
        id: id.into(),
        kind: "objective".into(),
        name: name.into(),
        value: TraceVal::Ratio(v),
        evidence,
    }
}

fn failure(id: &str, name: &str, evidence: Option<Span>) -> TraceRecord {
    TraceRecord {
        id: id.into(),
        kind: "event".into(),
        name: name.into(),
        value: TraceVal::Count(1),
        evidence,
    }
}

#[test]
fn coupled_counter_movement_becomes_tradeoff_opportunity() {
    // Throughput rises while a quality metric (coverage) falls across
    // matched samples, with an explicit coupling record in-regime.
    let text = "t1: throughput 100, coverage 0.9\nt2: throughput 200, coverage 0.6\ncoupling: coverage falls as throughput rises in this regime\n";
    let (corpus, src) = corpus_with(text);
    let records = vec![
        objective("obj1", "throughput", 100.0, Some(span(src, 0, 31))),
        objective("obj2", "coverage", 0.9, Some(span(src, 5, 31))),
        objective("obj3", "throughput", 200.0, Some(span(src, 32, 64))),
        objective("obj4", "coverage", 0.6, Some(span(src, 37, 64))),
        TraceRecord {
            id: "c1".into(),
            kind: "coupling".into(),
            name: "throughput-coverage".into(),
            value: TraceVal::Ratio(1.0),
            evidence: Some(span(src, 65, 122)),
        },
    ];
    let outcome = analyze(&corpus, &records);
    assert_eq!(outcome.opportunities.len(), 1, "one trade-off");
    let opp = &outcome.opportunities[0];
    assert_eq!(opp.kind, PressureKind::ConflictingObjectives);
    assert!(!opp.evidence_references.is_empty());
    assert!(outcome.rejected.is_empty());
}

#[test]
fn bare_counter_movement_without_coupling_is_rejected() {
    // Metrics move against each other but no coupling evidence in-regime:
    // MASTER_SPEC:144 — rejected, not emitted.
    let text = "throughput up, coverage down\n";
    let (corpus, src) = corpus_with(text);
    let records = vec![
        objective("obj1", "throughput", 100.0, Some(span(src, 0, 12))),
        objective("obj2", "coverage", 0.9, Some(span(src, 13, 28))),
        objective("obj3", "throughput", 200.0, Some(span(src, 0, 12))),
        objective("obj4", "coverage", 0.6, Some(span(src, 13, 28))),
    ];
    let outcome = analyze(&corpus, &records);
    assert!(outcome.opportunities.is_empty());
    assert_eq!(outcome.rejected.len(), 1);
    assert!(
        outcome.rejected[0].reason.contains("coupling"),
        "rejection names the coupling requirement: {}",
        outcome.rejected[0].reason
    );
}

#[test]
fn recurring_failures_form_one_pattern_opportunity() {
    // Three same-kind failures -> ONE pattern opportunity citing the
    // distinct failing records.
    let text = "fail: timeout on shard-a\nfail: timeout on shard-b\nfail: timeout on shard-c\n";
    let (corpus, src) = corpus_with(text);
    let records = vec![
        failure("f1", "timeout", Some(span(src, 0, 24))),
        failure("f2", "timeout", Some(span(src, 25, 49))),
        failure("f3", "timeout", Some(span(src, 50, 74))),
    ];
    let outcome = analyze(&corpus, &records);
    assert_eq!(outcome.opportunities.len(), 1, "one pattern, not three");
    let opp = &outcome.opportunities[0];
    assert_eq!(opp.kind, PressureKind::FailurePattern);
    assert_eq!(
        opp.evidence_references.len(),
        3,
        "cites the distinct failing records"
    );
    assert!(outcome.rejected.is_empty());
}

#[test]
fn single_failure_is_not_a_pattern() {
    let text = "fail: timeout on shard-a\n";
    let (corpus, src) = corpus_with(text);
    let records = vec![failure("f1", "timeout", Some(span(src, 0, 24)))];
    let outcome = analyze(&corpus, &records);
    assert!(outcome.opportunities.is_empty());
    assert_eq!(outcome.rejected.len(), 1);
    assert!(
        outcome.rejected[0].reason.contains("pattern"),
        "rejection names the pattern threshold: {}",
        outcome.rejected[0].reason
    );
}

// ---- Ticket 04: assumptions + changed capabilities + R-021 validity ----

#[test]
fn declared_assumption_becomes_opportunity_with_uncertainty() {
    // An `assumption` record: an unstated assumption declared in trace
    // metadata surfaces as an opportunity carrying explicit uncertainty.
    let text = "assumed: cache stays warm between steps\n";
    let (corpus, src) = corpus_with(text);
    let records = vec![TraceRecord {
        id: "a1".into(),
        kind: "assumption".into(),
        name: "warm cache".into(),
        value: TraceVal::Count(1),
        evidence: Some(span(src, 0, 38)),
    }];
    let outcome = analyze(&corpus, &records);
    assert_eq!(outcome.opportunities.len(), 1);
    let opp = &outcome.opportunities[0];
    assert_eq!(opp.kind, PressureKind::Assumption);
    assert!(
        !opp.causal_uncertainty.is_empty(),
        "assumption carries explicit uncertainty"
    );
    assert!(!opp.evidence_references.is_empty());
    assert!(outcome.rejected.is_empty());
}

#[test]
fn verified_capability_with_constraint_account_becomes_opportunity() {
    // A verified capability WITH a concrete account of the prior
    // constraint it changes (MASTER_SPEC:144).
    let text = "capability: bwrap sandbox verified available; changes prior constraint that execution had no isolation\n";
    let (corpus, src) = corpus_with(text);
    let records = vec![TraceRecord {
        id: "cap1".into(),
        kind: "capability".into(),
        name: "bwrap".into(),
        value: TraceVal::Count(1),
        evidence: Some(span(src, 0, 101)),
    }];
    // The constraint account rides in the record's name→constraint map:
    // the operator requires a matching `constraint` record.
    let records_with_constraint = {
        let mut v = records.clone();
        v.push(TraceRecord {
            id: "con1".into(),
            kind: "constraint".into(),
            name: "bwrap".into(),
            value: TraceVal::Count(1),
            evidence: Some(span(src, 12, 101)),
        });
        v
    };
    let outcome = analyze(&corpus, &records_with_constraint);
    assert_eq!(outcome.opportunities.len(), 1);
    assert_eq!(
        outcome.opportunities[0].kind,
        PressureKind::ChangedCapability
    );
    assert!(outcome.rejected.is_empty());
}

#[test]
fn capability_without_constraint_account_is_rejected() {
    // Availability without a concrete changed-constraint account:
    // rejected (MASTER_SPEC:144).
    let text = "capability: bwrap verified available\n";
    let (corpus, src) = corpus_with(text);
    let records = vec![TraceRecord {
        id: "cap1".into(),
        kind: "capability".into(),
        name: "bwrap".into(),
        value: TraceVal::Count(1),
        evidence: Some(span(src, 0, 36)),
    }];
    let outcome = analyze(&corpus, &records);
    assert!(outcome.opportunities.is_empty());
    assert_eq!(outcome.rejected.len(), 1);
    assert!(
        outcome.rejected[0].reason.contains("constraint"),
        "rejection names the constraint-account requirement: {}",
        outcome.rejected[0].reason
    );
}

#[test]
fn polished_unevidenced_candidate_stays_speculative() {
    // AT-021: narrative completeness NEVER lifts validity. A fully
    // polished candidate with no evidence is speculative with low
    // validity regardless of its polish count.
    let mut opp = hephaestus::discovery::Opportunity::speculative(PressureKind::Bottleneck);
    opp.problem_statement = "an extremely well-written, complete narrative".into();
    opp.beneficiary = "everyone".into();
    opp.context = "thorough context".into();
    opp.feasibility_envelope = "detailed feasibility".into();
    opp.prior_art_query_plan = vec!["query".into()];
    opp.unanswered_questions = vec!["none".into()];
    // Polish: count of filled narrative fields (independent accessor).
    let polish = hephaestus::discovery::narrative_polish(&opp);
    assert!(polish >= 6, "narrative is complete: {}", polish);
    assert_eq!(opp.validity, hephaestus::discovery::Validity::Speculative);
    assert!(opp.evidence_references.is_empty());
    // Validity is computed from evidence, never from polish:
    let validity = hephaestus::discovery::assess_validity(&opp);
    assert_eq!(
        validity,
        hephaestus::discovery::Validity::Speculative,
        "AT-021: polish does not lift validity"
    );
}

#[test]
fn evidence_backed_validity_tracks_evidence_not_narrative() {
    // The mirror case: strong evidence + thin narrative is still
    // evidence-backed.
    let text = "step=parse dur_ms=120\nstep=render dur_ms=2\nstep=parse dur_ms=131\nstep=render dur_ms=1\n";
    let (corpus, src) = corpus_with(text);
    let records = vec![
        TraceRecord {
            id: "t1".into(),
            kind: "duration".into(),
            name: "parse".into(),
            value: TraceVal::DurationMs(120),
            evidence: Some(span(src, 0, 21)),
        },
        TraceRecord {
            id: "t2".into(),
            kind: "duration".into(),
            name: "render".into(),
            value: TraceVal::DurationMs(2),
            evidence: Some(span(src, 22, 42)),
        },
        TraceRecord {
            id: "t3".into(),
            kind: "duration".into(),
            name: "parse".into(),
            value: TraceVal::DurationMs(131),
            evidence: Some(span(src, 43, 65)),
        },
        TraceRecord {
            id: "t4".into(),
            kind: "duration".into(),
            name: "render".into(),
            value: TraceVal::DurationMs(1),
            evidence: Some(span(src, 66, 88)),
        },
    ];
    let outcome = analyze(&corpus, &records);
    assert_eq!(outcome.opportunities.len(), 1);
    let opp = &outcome.opportunities[0];
    let polish = hephaestus::discovery::narrative_polish(opp);
    assert!(polish < 8, "narrative is thin: {}", polish);
    assert_eq!(
        hephaestus::discovery::assess_validity(opp),
        hephaestus::discovery::Validity::EvidenceBacked,
        "validity follows evidence"
    );
}

#[test]
fn all_six_operators_wired_through_analyze() {
    // One analyze call whose records trigger every operator kind; the
    // outcome contains all six pressure kinds exactly once each.
    let text = "step=parse dur_ms=120\nstep=render dur_ms=2\nstep=parse dur_ms=131\nstep=render dur_ms=1\npredicted 50; observed 500\ncoupling present\nfail a\nfail a\nfail a\nassumed: warm cache\ncapability: verified\nconstraint: prior limit\n";
    let (corpus, src) = corpus_with(text);
    let records = vec![
        TraceRecord {
            id: "t1".into(),
            kind: "duration".into(),
            name: "parse".into(),
            value: TraceVal::DurationMs(120),
            evidence: Some(span(src, 0, 21)),
        },
        TraceRecord {
            id: "t2".into(),
            kind: "duration".into(),
            name: "render".into(),
            value: TraceVal::DurationMs(2),
            evidence: Some(span(src, 22, 42)),
        },
        TraceRecord {
            id: "t3".into(),
            kind: "duration".into(),
            name: "parse".into(),
            value: TraceVal::DurationMs(131),
            evidence: Some(span(src, 43, 65)),
        },
        TraceRecord {
            id: "t4".into(),
            kind: "duration".into(),
            name: "render".into(),
            value: TraceVal::DurationMs(1),
            evidence: Some(span(src, 66, 88)),
        },
        TraceRecord {
            id: "p1".into(),
            kind: "prediction".into(),
            name: "latency".into(),
            value: TraceVal::DurationMs(50),
            evidence: Some(span(src, 89, 104)),
        },
        TraceRecord {
            id: "o1".into(),
            kind: "observation".into(),
            name: "latency".into(),
            value: TraceVal::DurationMs(500),
            evidence: Some(span(src, 105, 117)),
        },
        TraceRecord {
            id: "c1".into(),
            kind: "coupling".into(),
            name: "throughput-coverage".into(),
            value: TraceVal::Ratio(1.0),
            evidence: Some(span(src, 118, 133)),
        },
        TraceRecord {
            id: "obj1".into(),
            kind: "objective".into(),
            name: "throughput".into(),
            value: TraceVal::Ratio(100.0),
            evidence: Some(span(src, 118, 133)),
        },
        TraceRecord {
            id: "obj2".into(),
            kind: "objective".into(),
            name: "coverage".into(),
            value: TraceVal::Ratio(0.9),
            evidence: Some(span(src, 118, 133)),
        },
        TraceRecord {
            id: "obj3".into(),
            kind: "objective".into(),
            name: "throughput".into(),
            value: TraceVal::Ratio(200.0),
            evidence: Some(span(src, 118, 133)),
        },
        TraceRecord {
            id: "obj4".into(),
            kind: "objective".into(),
            name: "coverage".into(),
            value: TraceVal::Ratio(0.6),
            evidence: Some(span(src, 118, 133)),
        },
        TraceRecord {
            id: "f1".into(),
            kind: "event".into(),
            name: "fail".into(),
            value: TraceVal::Count(1),
            evidence: Some(span(src, 134, 140)),
        },
        TraceRecord {
            id: "f2".into(),
            kind: "event".into(),
            name: "fail".into(),
            value: TraceVal::Count(1),
            evidence: Some(span(src, 141, 147)),
        },
        TraceRecord {
            id: "f3".into(),
            kind: "event".into(),
            name: "fail".into(),
            value: TraceVal::Count(1),
            evidence: Some(span(src, 148, 154)),
        },
        TraceRecord {
            id: "a1".into(),
            kind: "assumption".into(),
            name: "warm cache".into(),
            value: TraceVal::Count(1),
            evidence: Some(span(src, 155, 175)),
        },
        TraceRecord {
            id: "cap1".into(),
            kind: "capability".into(),
            name: "bwrap".into(),
            value: TraceVal::Count(1),
            evidence: Some(span(src, 176, 194)),
        },
        TraceRecord {
            id: "con1".into(),
            kind: "constraint".into(),
            name: "bwrap".into(),
            value: TraceVal::Count(1),
            evidence: Some(span(src, 176, 194)),
        },
    ];
    let outcome = analyze(&corpus, &records);
    let mut kinds: Vec<&PressureKind> = outcome.opportunities.iter().map(|o| &o.kind).collect();
    kinds.sort_by_key(|k| k.kind_name());
    kinds.dedup_by(|a, b| a == b);
    let want = [
        PressureKind::Anomaly,
        PressureKind::Assumption,
        PressureKind::Bottleneck,
        PressureKind::ChangedCapability,
        PressureKind::ConflictingObjectives,
        PressureKind::FailurePattern,
    ];
    assert_eq!(
        kinds.len(),
        6,
        "all six kinds present (multiples allowed per kind), got {:?}",
        kinds
    );
    for (got, want) in kinds.iter().zip(want.iter()) {
        assert_eq!(*got, want, "kind set mismatch");
    }
}
