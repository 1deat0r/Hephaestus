//! E2E mission driver — M2 chain (T-061 ticket 01; IMPLEMENTATION_PLAN:64).
//!
//! goal -> mission compile -> hidden-world fixture traces -> discovery ->
//! genesis -> test-ready hypothesis. Receipts for the M2-M4 exit
//! assessment. NO truth reads: the test never calls `evaluate()` or
//! `truth_at()` — observations are data; verdicts arrive in M3 (e2e_m3).

use hephaestus::contracts::generated::Money;
use hephaestus::discovery::{TraceRecord, TraceVal, analyze};
use hephaestus::genesis::hypothesis::{Readiness, compile};
use hephaestus::genesis::registry::apply_all;
use hephaestus::genesis::validators::validate;
use hephaestus::knowledge::{Corpus, ParserId, ingest_bytes};
use hephaestus::mission::{
    AutonomyProfile, Compiled, Intake, ResourceEnvelope, ResourceProfile,
    compile as mission_compile,
};

fn intake(goal: &str) -> Intake {
    Intake {
        goal: goal.to_string(),
        beneficiary: "fixture lab".to_string(),
        standing_priorities: vec!["reduce benchmark cost".to_string()],
        resource: ResourceProfile {
            envelope: ResourceEnvelope {
                limit: Money {
                    currency: "USD".to_string(),
                    minor_units: 10_000,
                },
                pricing_unknown: false,
            },
            spending_approved: true,
        },
        requested_profile: AutonomyProfile::LocalResearch,
        caller_hypothesis: None,
    }
}

/// Parse the checked-in world-derived trace fixture (data only — the
/// control plane never links the hidden evaluator). Returns the text
/// plus per-line (span range, stage name, duration) from the file itself.
fn fixture_trace() -> (String, Vec<(std::ops::Range<usize>, String, u64)>) {
    let text = include_str!("fixtures/e2e-trace.log").to_string();
    let mut out = Vec::new();
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let range = offset..offset + line.len();
        offset += line.len();
        let rest = line
            .trim_end()
            .strip_prefix("step=")
            .unwrap_or_else(|| panic!("fixture line shape: {line:?}"));
        let (name, dur) = rest
            .split_once(" dur_ms=")
            .unwrap_or_else(|| panic!("fixture line shape: {line:?}"));
        out.push((range, name.to_string(), dur.parse().expect("dur_ms")));
    }
    (text, out)
}

fn ingest(text: &str) -> (Corpus, u64) {
    let mut corpus = Corpus::new();
    let src = ingest_bytes(
        &mut corpus,
        "fixture://hidden-world/trace.log",
        text.as_bytes().to_vec(),
        "2026-10-02T00:00:00Z",
        ParserId {
            name: "trace".into(),
            version: "1".into(),
        },
    );
    (corpus, src)
}

#[test]
fn e2e_m2_goal_reaches_a_test_ready_hypothesis_on_hidden_world_data() {
    // 1. Mission compiles from a broad domain-only goal (no seed hypothesis).
    let compiled = mission_compile(&intake(
        "reduce the cost of building correct coding-agent context",
    ))
    .expect("fixture intake compiles");
    let mission = match compiled {
        Compiled::Mission(m) => m,
        Compiled::NeedsAuthorization(r) => panic!("fixture intake must be pre-approved: {r:?}"),
    };
    assert!(
        !mission.objective.is_empty(),
        "mission carries the objective"
    );

    // 2. Hidden-world fixture traces ingested with span-cited evidence.
    let (text, spans) = fixture_trace();
    let (mut corpus, src) = ingest(&text);
    let records: Vec<TraceRecord> = spans
        .iter()
        .enumerate()
        .map(|(i, (range, name, dur))| {
            let mut s = hephaestus::discovery::Span {
                source: src,
                start: range.start,
                end: range.end,
                text: text[range.clone()].to_string(),
                transforms: vec![],
            };
            s.text = text[range.clone()].to_string();
            TraceRecord {
                id: format!("t{i}"),
                kind: "duration".to_string(),
                name: name.clone(),
                value: TraceVal::DurationMs(*dur),
                evidence: Some(s),
            }
        })
        .collect();
    assert!(!records.is_empty());
    let _ = &mut corpus;

    // 3. Discovery: grounded opportunity(s) — evidence spans attached,
    //    never speculative, and NO truth is read to get here.
    let outcome = analyze(&corpus, &records);
    assert!(
        !outcome.opportunities.is_empty(),
        "the hidden-world trace yields at least one pressure point"
    );
    let grounded = outcome
        .opportunities
        .iter()
        .filter(|o| !o.evidence_references.is_empty())
        .count();
    assert!(
        grounded >= 1,
        "opportunities are span-grounded: {:?}",
        outcome.opportunities
    );

    // 4. Genesis: operators emit mechanism records (deterministic, bounded).
    let opp = &outcome.opportunities[0];
    let sweep = apply_all(
        opp,
        vec![
            "the heavy stage dominates observed duration".to_string(),
            "splitting the heavy stage could cut tail latency".to_string(),
        ],
        vec![],
        8,
    );
    assert!(
        !sweep.mechanisms.is_empty(),
        "operators produce mechanisms; rejections: {:?}",
        sweep.rejected_applicability
    );
    let mechanism = &sweep.mechanisms[0];
    assert!(
        !mechanism.failure_modes.is_empty(),
        "mechanism carries failure modes (recorded, M2 exit)"
    );

    // 5. Hypothesis compile + semantic validation -> TestReady (never
    //    self-declared: readiness comes from the validators).
    let hypothesis = compile(mechanism);
    let verdict = validate(&hypothesis);
    assert_eq!(
        verdict.readiness,
        Readiness::TestReady,
        "complete compiled hypothesis is test-ready: {:?}",
        verdict.denials
    );

    // 6. Generation is RECORDABLE (M2 exit): the whole chain serializes
    //    byte-identically across two runs over the same seeded world.
    let run = || -> String {
        let (t, sp) = fixture_trace();
        let (c, s) = ingest(&t);
        let recs: Vec<TraceRecord> = sp
            .iter()
            .enumerate()
            .map(|(i, (r, name, d))| TraceRecord {
                id: format!("t{i}"),
                kind: "duration".to_string(),
                name: name.clone(),
                value: TraceVal::DurationMs(*d),
                evidence: Some({
                    let mut e = hephaestus::discovery::Span {
                        source: s,
                        start: r.start,
                        end: r.end,
                        text: t[r.clone()].to_string(),
                        transforms: vec![],
                    };
                    e.text = t[r.clone()].to_string();
                    e
                }),
            })
            .collect();
        let o = analyze(&c, &recs);
        let sweep = apply_all(
            &o.opportunities[0],
            vec![
                "the heavy stage dominates observed duration".to_string(),
                "splitting the heavy stage could cut tail latency".to_string(),
            ],
            vec![],
            8,
        );
        let h = compile(&sweep.mechanisms[0]);
        let v = validate(&h);
        serde_json::json!({
            "opportunities": o.opportunities.len(),
            "grounded": grounded,
            "mechanisms": sweep.mechanisms.len(),
            "mechanism": sweep.mechanisms[0],
            "hypothesis": h,
            "readiness": format!("{:?}", v.readiness),
            "denials": v.denials,
        })
        .to_string()
    };
    assert_eq!(
        run(),
        run(),
        "twin runs are byte-identical (recordable generation)"
    );
}
