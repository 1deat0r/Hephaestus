//! Genesis mechanism-operator registry (T-015, R-022/023/024).
//!
//! Integration tests at the public seam: `registry::apply` /
//! `registry::apply_all`. Mechanism records refuse placeholder language;
//! operators emit records with provenance or structured
//! rejected-applicability reasons; the sweep is deterministic and bounded.

use hephaestus::discovery::{Opportunity, Span, TraceRecord, TraceVal, analyze};
use hephaestus::genesis::record::{MechanismError, MechanismRecord};
use hephaestus::genesis::registry::{OperatorOutcome, apply, apply_all};
use hephaestus::knowledge::{ParserId, ingest_bytes};

fn s(v: &str) -> String {
    v.to_string()
}

/// An evidence-backed bottleneck opportunity built through the real
/// discovery path (T-013 corpus + T-014 operators).
fn bottleneck_opportunity() -> Opportunity {
    let text = "step=parse dur_ms=120\nstep=render dur_ms=2\nstep=parse dur_ms=131\nstep=render dur_ms=1\n";
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
    let sp = |start: usize, end: usize| Span {
        source: src,
        start,
        end,
        text: String::new(),
        transforms: vec![],
    };
    let records = vec![
        TraceRecord {
            id: "t1".into(),
            kind: "duration".into(),
            name: "parse".into(),
            value: TraceVal::DurationMs(120),
            evidence: Some(sp(0, 21)),
        },
        TraceRecord {
            id: "t2".into(),
            kind: "duration".into(),
            name: "render".into(),
            value: TraceVal::DurationMs(2),
            evidence: Some(sp(22, 42)),
        },
        TraceRecord {
            id: "t3".into(),
            kind: "duration".into(),
            name: "parse".into(),
            value: TraceVal::DurationMs(131),
            evidence: Some(sp(43, 65)),
        },
        TraceRecord {
            id: "t4".into(),
            kind: "duration".into(),
            name: "render".into(),
            value: TraceVal::DurationMs(1),
            evidence: Some(sp(66, 88)),
        },
    ];
    let outcome = analyze(&corpus, &records);
    outcome
        .opportunities
        .into_iter()
        .next()
        .expect("bottleneck")
}

// ---- Ticket 01: MechanismRecord + registry + abduction ----

#[test]
fn placeholder_language_is_refused_at_construction() {
    // AT-022 negative: a candidate that only says "add AI" is not a
    // mechanism — construction refuses with a reason naming the
    // mechanism requirement.
    for phrase in ["use ai", "add a graph", "make it adaptive", "use AI"] {
        let err = MechanismRecord::new(
            s(phrase),
            vec![],
            vec![s("would produce the observed pressure point")],
            vec![],
            vec![],
            String::new(),
            vec![],
            s("test"),
        )
        .expect_err("placeholder must be refused");
        assert!(
            matches!(err, MechanismError::PlaceholderLanguage(_)),
            "expected PlaceholderLanguage for '{phrase}'"
        );
    }
}

#[test]
fn genuine_mechanism_constructs_and_carries_all_fields() {
    let rec = MechanismRecord::new(
        s("cache context by exact dependency hash"),
        vec![s("context_build_time"), s("dependency_hash")],
        vec![s("build_time decreases as unchanged deps skip rebuild")],
        vec![s("dependency graph extractable from imports")],
        vec![s("build_time halves on warm cache with <10% file churn")],
        s("repository-scale code changes; single-machine workloads"),
        vec![s("stale invalidation if hash omits transitive imports")],
        s("hash imports at task start; key the cache on the hash"),
    )
    .expect("genuine mechanism");
    assert!(!rec.realization.is_empty());
    assert!(!rec.variables.is_empty());
    assert!(!rec.relationships.is_empty());
}

#[test]
fn abduction_single_explanation_is_rejected() {
    // MASTER_SPEC:162: abduction MUST enumerate at least one competing
    // explanation.
    let opp = bottleneck_opportunity();
    let outcome: OperatorOutcome = apply(
        "abduction",
        &opp,
        vec![s("hash collision in parser")],
        vec![],
    );
    assert!(
        outcome.mechanisms.is_empty(),
        "single explanation must not emit"
    );
    assert_eq!(outcome.rejected_applicability.len(), 1);
    assert!(
        outcome.rejected_applicability[0]
            .reason
            .contains("competing"),
        "rejection names the competing-explanation requirement: {}",
        outcome.rejected_applicability[0].reason
    );
}

#[test]
fn abduction_with_competing_explanations_emits_mechanism() {
    let opp = bottleneck_opportunity();
    let outcome = apply(
        "abduction",
        &opp,
        vec![
            s("parse step does redundant work per invocation"),
            s("parse step allocates unboundedly under load"),
        ],
        vec![],
    );
    assert_eq!(
        outcome.mechanisms.len(),
        2,
        "each valid competing explanation becomes a candidate mechanism"
    );
    let rec = &outcome.mechanisms[0];
    assert!(
        !rec.provenance.operator.is_empty() && !rec.provenance.source_opportunity.is_empty(),
        "mechanism carries operator + opportunity provenance"
    );
    assert!(outcome.rejected_applicability.is_empty());
}

#[test]
fn low_plausibility_valid_mechanism_stays_eligible() {
    // AT-024 negative: an unusual but valid mechanism with low
    // plausibility is NOT eliminated — advisory flag only, and the
    // heuristic rejection is auditable.
    let opp = bottleneck_opportunity();
    let outcome = apply(
        "abduction",
        &opp,
        vec![
            s("parse step is rate-limited by an external token bucket"),
            s("planetary alignment perturbs the scheduler"),
        ],
        vec![s("planetary alignment perturbs the scheduler")],
    );
    // Both explanations survive (valid records, advisory-flagged).
    assert_eq!(outcome.mechanisms.len(), 2);
    let flagged: Vec<_> = outcome
        .mechanisms
        .iter()
        .filter(|m| m.plausibility.is_some())
        .collect();
    assert!(!flagged.is_empty(), "low-plausibility mechanism flagged");
    assert!(
        outcome
            .heuristic_rejection_audit
            .iter()
            .any(|r| r.contains("planetary")),
        "heuristic rejection auditable: {:?}",
        outcome.heuristic_rejection_audit
    );
}

#[test]
fn registry_sweep_is_deterministic_and_bounded() {
    let opp = bottleneck_opportunity();
    let a = apply_all(
        &opp,
        vec![s("explanation A"), s("explanation B")],
        vec![],
        8,
    );
    let b = apply_all(
        &opp,
        vec![s("explanation A"), s("explanation B")],
        vec![],
        8,
    );
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap(),
        "twin-run byte-identical"
    );
    // Bounded output: the sweep never returns more than the bound.
    assert!(a.mechanisms.len() <= 8);
}

// ---- Ticket 02: contradiction resolution + structural transfer ----

#[test]
fn contradiction_resolution_requires_named_decoupling_axis() {
    let opp = bottleneck_opportunity();
    // No axis named -> rejected applicability.
    let outcome = apply(
        "contradiction_resolution",
        &opp,
        vec![s("decouple parse from render")],
        vec![],
    );
    assert!(outcome.mechanisms.is_empty(), "no axis -> no mechanism");
    assert_eq!(outcome.rejected_applicability.len(), 1);
    assert!(
        outcome.rejected_applicability[0]
            .reason
            .contains("decoupling_axis"),
        "rejection names the axis requirement: {}",
        outcome.rejected_applicability[0].reason
    );
}

#[test]
fn contradiction_resolution_with_axis_emits_record() {
    let opp = bottleneck_opportunity();
    let outcome = apply(
        "contradiction_resolution",
        &opp,
        vec![s(
            "decouple parse from render; axis=temporal separation: parse at commit time, render at query time",
        )],
        vec![],
    );
    assert_eq!(outcome.mechanisms.len(), 1);
    let rec = &outcome.mechanisms[0];
    // The record states which decoupling axis it uses (checkable claim).
    assert!(
        !rec.relationships.is_empty(),
        "decoupling relationship recorded"
    );
    assert!(outcome.rejected_applicability.is_empty());
}

#[test]
fn structural_transfer_requires_relations_not_vocabulary() {
    let opp = bottleneck_opportunity();
    // Vocabulary-only transfer (no relational structure) -> rejected.
    let outcome = apply(
        "structural_transfer",
        &opp,
        vec![s(
            "target=build-system; relations=; boundaries=same repo scale",
        )],
        vec![],
    );
    assert!(
        outcome.mechanisms.is_empty(),
        "vocabulary-only transfer must not emit (MASTER_SPEC:162)"
    );
    assert_eq!(outcome.rejected_applicability.len(), 1);
    assert!(
        outcome.rejected_applicability[0]
            .reason
            .contains("relational"),
        "rejection names the relational-structure requirement: {}",
        outcome.rejected_applicability[0].reason
    );
}

#[test]
fn structural_transfer_with_relations_carries_broken_relations_and_boundaries() {
    let opp = bottleneck_opportunity();
    let outcome = apply(
        "structural_transfer",
        &opp,
        vec![s(
            "target=build-system; relations=source{A} -> dependency{B}: rebuild B when A changes; relations=cache{C} keyed-by hash{D}: reuse C when D unchanged; broken=none; boundaries=single-machine repo; regime match",
        )],
        vec![],
    );
    assert_eq!(outcome.mechanisms.len(), 1);
    let rec = &outcome.mechanisms[0];
    assert!(!rec.prerequisites.is_empty(), "boundary conditions present");
    assert!(outcome.rejected_applicability.is_empty());
}

#[test]
fn incompatible_regime_transfer_is_flagged_with_boundary_reasons() {
    // AT-023 negative: transfer between incompatible regimes is flagged
    // with boundary-specific reasons.
    let opp = bottleneck_opportunity();
    let outcome = apply(
        "structural_transfer",
        &opp,
        vec![s(
            "target=distributed-cluster; relations=source{A} -> dependency{B}: rebuild B when A changes; relations=cache{C} keyed-by hash{D}: reuse C when D unchanged; broken=consistency semantics differ across nodes; boundaries=multi-node network; regime mismatch",
        )],
        vec![],
    );
    // The transfer emits ONLY as flagged/limited or is rejected with
    // boundary-specific reasons; it is never silently accepted.
    let rejected_boundary = outcome
        .rejected_applicability
        .iter()
        .any(|r| r.reason.contains("regime") || r.detail.contains("regime"));
    let flagged_record = outcome
        .mechanisms
        .iter()
        .any(|m| m.plausibility.is_some() || !m.operating_regime.is_empty());
    assert!(
        rejected_boundary || !outcome.mechanisms.is_empty(),
        "boundary-specific handling present"
    );
    if !outcome.mechanisms.is_empty() {
        assert!(
            flagged_record,
            "an emitted cross-regime transfer carries its boundary conditions"
        );
    }
}

// ---- Ticket 03: composition + subtraction + failure resurrection ----

#[test]
fn composition_rejects_unit_or_interface_mismatch() {
    let opp = bottleneck_opportunity();
    // Mismatched parts -> rejected applicability.
    let outcome = apply(
        "composition",
        &opp,
        vec![s(
            "parts=cache[A] + indexer[B]; units=ms vs count; interfaces=pipe; budget=within",
        )],
        vec![],
    );
    assert!(outcome.mechanisms.is_empty(), "mismatch -> no record");
    assert_eq!(outcome.rejected_applicability.len(), 1);
    assert!(
        outcome.rejected_applicability[0]
            .reason
            .contains("compatib"),
        "rejection names compatibility: {}",
        outcome.rejected_applicability[0].reason
    );
}

#[test]
fn composition_compatible_parts_emit_record() {
    let opp = bottleneck_opportunity();
    let outcome = apply(
        "composition",
        &opp,
        vec![s(
            "parts=cache[A] + indexer[B]; units=ms vs ms; interfaces=pipe[A.out->B.in]; budget=within; interactions=none-known",
        )],
        vec![],
    );
    assert_eq!(outcome.mechanisms.len(), 1);
    assert!(outcome.rejected_applicability.is_empty());
}

#[test]
fn subtraction_states_exactly_one_three_way_outcome() {
    let opp = bottleneck_opportunity();
    let outcome = apply(
        "subtraction",
        &opp,
        vec![s(
            "component=render; outcome=moves-elsewhere: render deferred to consumer",
        )],
        vec![],
    );
    assert_eq!(outcome.mechanisms.len(), 1);
    let rec = &outcome.mechanisms[0];
    // Exactly one of the three outcomes stated in the record.
    let text = format!(
        "{} {} {}",
        rec.statement,
        rec.expected_effects.join(" "),
        rec.realization
    );
    let outcomes = [
        text.contains("disappears"),
        text.contains("moves-elsewhere"),
        text.contains("never-necessary"),
    ];
    assert_eq!(
        outcomes.iter().filter(|b| **b).count(),
        1,
        "exactly one outcome: {text}"
    );
}

#[test]
fn subtraction_without_outcome_is_rejected() {
    let opp = bottleneck_opportunity();
    let outcome = apply("subtraction", &opp, vec![s("component=render")], vec![]);
    assert!(outcome.mechanisms.is_empty());
    assert_eq!(outcome.rejected_applicability.len(), 1);
    assert!(
        outcome.rejected_applicability[0].reason.contains("outcome"),
        "rejection names the three-way outcome requirement: {}",
        outcome.rejected_applicability[0].reason
    );
}

#[test]
fn failure_resurrection_requires_changed_boundary_conditions() {
    let opp = bottleneck_opportunity();
    // Unchanged conditions -> rejected (never blindly retried).
    let outcome = apply(
        "failure_resurrection",
        &opp,
        vec![s(
            "earlier_failure=exact-dep-cache; conditions=unchanged; evidence=none",
        )],
        vec![],
    );
    assert!(outcome.mechanisms.is_empty(), "unchanged -> no retry");
    assert_eq!(outcome.rejected_applicability.len(), 1);
    assert!(
        outcome.rejected_applicability[0].reason.contains("changed"),
        "rejection names changed-conditions requirement: {}",
        outcome.rejected_applicability[0].reason
    );
}

#[test]
fn failure_resurrection_with_changed_conditions_emits_record() {
    let opp = bottleneck_opportunity();
    let outcome = apply(
        "failure_resurrection",
        &opp,
        vec![s(
            "earlier_failure=exact-dep-cache; conditions=changed: dependency-graph extraction now exists; evidence=span-cited",
        )],
        vec![],
    );
    assert_eq!(outcome.mechanisms.len(), 1);
    let rec = &outcome.mechanisms[0];
    assert!(
        !rec.prerequisites.is_empty(),
        "changed conditions recorded as prerequisites"
    );
    assert!(outcome.rejected_applicability.is_empty());
}

#[test]
fn apply_all_sweep_exercises_all_six_operators() {
    let opp = bottleneck_opportunity();
    let outcome = apply_all(
        &opp,
        vec![
            s("explanation A"),
            s("explanation B"),
            s("decouple parse from render; axis=temporal separation"),
            s(
                "target=build-system; relations=cache{C} keyed-by hash{D}: reuse C when D unchanged; broken=none; boundaries=single-machine; regime match",
            ),
            s(
                "parts=cache[A] + indexer[B]; units=ms vs ms; interfaces=pipe[A.out->B.in]; budget=within; interactions=none-known",
            ),
            s("component=render; outcome=never-necessary: output renders identical without it"),
            s(
                "earlier_failure=exact-dep-cache; conditions=changed: dependency-graph extraction now exists; evidence=span-cited",
            ),
        ],
        vec![],
        8,
    );
    let mut ops: Vec<String> = outcome
        .mechanisms
        .iter()
        .map(|m| m.provenance.operator.clone())
        .collect();
    ops.sort();
    ops.dedup();
    assert_eq!(
        ops.len(),
        6,
        "all six operators produced mechanisms: {:?}",
        ops
    );
}
