//! Bounded search orchestrator (T-017, R-031/AT-031, R-024).
//!
//! Integration tests at the public seam: `orchestrator::run`, `audit`,
//! `lineage`, `dedup_key`. Dedup stops paraphrase loops; bounds stop
//! expansion with recorded reasons; the beam keeps ≤ width per wave; the
//! archive retains everything non-kept; the allocation policy is visible.

use hephaestus::discovery::{Opportunity, Span};
use hephaestus::orchestrator::record::{AllocationPolicy, SearchBounds, StopReason};
use hephaestus::orchestrator::{audit, dedup_key, lineage, run};
use std::collections::BTreeMap;

fn s(v: &str) -> String {
    v.to_string()
}

/// A minimal evidence-carrying seed opportunity (no corpus round-trip
/// needed: the orchestrator consumes the Opportunity's span references).
fn seed_opp() -> Opportunity {
    let mut opp = Opportunity::grounded(
        hephaestus::discovery::PressureKind::Bottleneck,
        vec![Span {
            source: 1,
            start: 0,
            end: 10,
            text: String::new(),
            transforms: vec![],
        }],
    );
    opp.problem_statement = s("parse step is repeated and material");
    opp
}

fn bounds() -> SearchBounds {
    SearchBounds {
        max_depth: 4,
        max_candidates: 64,
        max_operator_calls: 16,
        max_duplicate_strikes: 3,
    }
}

/// Expansion map: each parent statement maps to its children.
fn expansions(pairs: Vec<(&str, Vec<&str>)>) -> BTreeMap<String, Vec<String>> {
    pairs
        .into_iter()
        .map(|(k, v)| (dedup_key(k), v.into_iter().map(s).collect()))
        .collect()
}

#[test]
fn endless_paraphrases_stop_with_duplicate_reason() {
    // AT-031 negative: a generator returning endless paraphrases is
    // stopped by dedup + configured limits, with a reason. The children
    // carry genuinely-new tokens (a child restating the seed would be a
    // seed rediscovery, which is a different case).
    let exp = expansions(vec![(
        "parse step is repeated and material",
        vec![
            "indexer rebuild skips unchanged files", // unique: kept
            "indexer rebuild skips unchanged files", // dup strike 1
            "skips indexer rebuild unchanged files", // dup strike 2 (same tokens)
            "unchanged files skips indexer rebuild", // dup strike 3 (same tokens)
        ],
    )]);
    let outcome = run(
        &seed_opp(),
        &exp,
        &bounds(),
        2,
        &AllocationPolicy::default(),
    );
    // The two children share the same dedup key -> duplicate strikes.
    assert!(
        outcome.duplicates_strike >= 1,
        "paraphrase detected: {}",
        outcome.duplicates_strike
    );
    assert!(
        outcome
            .archive
            .iter()
            .any(|e| e.reason == "duplicate_generation"),
        "duplicate archived with reason"
    );
    assert_eq!(
        outcome.stop,
        Some(StopReason::DuplicateBoundReached),
        "endless paraphrases hit the duplicate bound"
    );
}

#[test]
fn candidate_bound_stops_expansion() {
    let exp = expansions(vec![(
        "parse step is repeated and material",
        vec![
            "child one unique idea",
            "child two unique idea",
            "child three unique idea",
        ],
    )]);
    let tight = SearchBounds {
        max_depth: 4,
        max_candidates: 3, // seed + 1 child, then stop
        max_operator_calls: 16,
        max_duplicate_strikes: 3,
    };
    let outcome = run(&seed_opp(), &exp, &tight, 2, &AllocationPolicy::default());
    assert_eq!(
        outcome.stop,
        Some(StopReason::CandidateBoundReached),
        "candidate bound recorded"
    );
    assert!(outcome.nodes.len() <= 3);
}

#[test]
fn beam_keeps_at_most_width_per_wave_and_archives_rest() {
    let exp = expansions(vec![(
        "parse step is repeated and material",
        vec![
            "child one unique idea",
            "child two unique idea",
            "child three unique idea",
            "child four unique idea",
            "child five unique idea",
        ],
    )]);
    let outcome = run(
        &seed_opp(),
        &exp,
        &bounds(),
        2,
        &AllocationPolicy::default(),
    );
    // Wave-1 children: all 5 exist as nodes; at most `beam_width` remain in
    // the search frontier (kept), the rest are archived with reasons.
    let wave1_kept = outcome.nodes.iter().filter(|n| n.depth == 1).count();
    let wave1_archived = outcome
        .archive
        .iter()
        .filter(|e| e.wave == 1 && e.reason == "below_beam_cut")
        .count();
    assert_eq!(
        wave1_kept, 5,
        "all children are nodes (the archive is a VIEW over non-kept ones)"
    );
    assert!(
        wave1_archived >= 3,
        "beam of 2 archives at least 3 of 5: {}",
        wave1_archived
    );
    assert!(
        wave1_kept - wave1_archived <= 2,
        "kept-in-frontier ({} - {}) respects the beam width 2",
        wave1_kept,
        wave1_archived
    );
    // Archived entries carry reasons (R-024 audit substrate).
    assert!(
        outcome.archive.iter().all(|e| !e.reason.is_empty()),
        "every archive entry has a reason"
    );
}

#[test]
fn lineage_walks_to_seed() {
    let exp = expansions(vec![(
        "parse step is repeated and material",
        vec!["child one unique idea"],
    )]);
    let outcome = run(
        &seed_opp(),
        &exp,
        &bounds(),
        2,
        &AllocationPolicy::default(),
    );
    let child = outcome.nodes.iter().find(|n| n.depth == 1).expect("child");
    let path = lineage(&outcome, child.id);
    assert_eq!(path, vec![0, child.id], "seed-first lineage path");
}

#[test]
fn allocation_policy_is_versioned_and_visible() {
    // §11:221: provisional 55/25/15/5, changeable only via policy version.
    let policy = AllocationPolicy::default();
    assert_eq!(
        (
            policy.version,
            policy.exploit_pct,
            policy.diversify_pct,
            policy.replicate_pct,
            policy.audit_pct
        ),
        (1, 55, 25, 15, 5)
    );
    let exp = expansions(vec![]);
    let outcome = run(&seed_opp(), &exp, &bounds(), 2, &policy);
    assert_eq!(
        outcome.allocation_policy, policy,
        "policy visible in outcome"
    );
}

#[test]
fn audit_returns_all_rejections_with_reasons() {
    let exp = expansions(vec![(
        "parse step is repeated and material",
        vec![
            "child one unique idea",
            "child two unique idea",
            "child three unique idea",
            "child four unique idea",
        ],
    )]);
    let outcome = run(
        &seed_opp(),
        &exp,
        &bounds(),
        1,
        &AllocationPolicy::default(),
    );
    let audit_entries = audit(&outcome);
    assert!(!audit_entries.is_empty(), "beam of 1 archives the rest");
    assert!(
        audit_entries.iter().all(|e| !e.reason.is_empty()),
        "every rejection carries a reason (R-024)"
    );
}

#[test]
fn twin_run_byte_identical() {
    let exp = expansions(vec![(
        "parse step is repeated and material",
        vec!["child one unique idea", "child two unique idea"],
    )]);
    let a = run(
        &seed_opp(),
        &exp,
        &bounds(),
        2,
        &AllocationPolicy::default(),
    );
    let b = run(
        &seed_opp(),
        &exp,
        &bounds(),
        2,
        &AllocationPolicy::default(),
    );
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}

// ---- Ticket 02: archive/policy/lineage/audit deep coverage ----

#[test]
fn archived_rejections_carry_node_and_wave() {
    // audit() entries expose the rejected candidate itself + wave number.
    let exp = expansions(vec![(
        "parse step is repeated and material",
        vec![
            "child one unique idea",
            "child two unique idea",
            "child three unique idea",
        ],
    )]);
    let outcome = run(
        &seed_opp(),
        &exp,
        &bounds(),
        1,
        &AllocationPolicy::default(),
    );
    let entries = audit(&outcome);
    assert!(entries.iter().all(|e| e.wave >= 1), "wave recorded");
    assert!(
        entries.iter().all(|e| !e.node.statement.is_empty()),
        "candidate retained in the archive entry"
    );
    // Duplicate entries are auditable too (AT-031 + R-024 together).
    let exp2 = expansions(vec![(
        "parse step is repeated and material",
        vec![
            "indexer rebuild skips unchanged files",
            "indexer rebuild skips unchanged files",
            "skips indexer rebuild unchanged files",
            "unchanged files skips indexer rebuild",
        ],
    )]);
    let outcome2 = run(
        &seed_opp(),
        &exp2,
        &bounds(),
        2,
        &AllocationPolicy::default(),
    );
    assert!(
        audit(&outcome2)
            .iter()
            .any(|e| e.reason == "duplicate_generation"),
        "duplicate rejections auditable"
    );
}

#[test]
fn lineage_through_multiple_generations() {
    let exp = expansions(vec![
        (
            "parse step is repeated and material",
            vec!["child one unique idea"],
        ),
        ("child one unique idea", vec!["grandchild unique idea"]),
    ]);
    let outcome = run(
        &seed_opp(),
        &exp,
        &bounds(),
        2,
        &AllocationPolicy::default(),
    );
    let grandchild = outcome
        .nodes
        .iter()
        .find(|n| n.depth == 2)
        .expect("grandchild exists");
    let path = lineage(&outcome, grandchild.id);
    assert_eq!(path.len(), 3, "seed -> child -> grandchild");
    assert_eq!(path[0], 0, "rooted at seed");
    // Each node records its operator.
    let gc = &outcome.nodes[grandchild.id as usize];
    assert!(!gc.operator.is_empty() && !gc.parents.is_empty());
}

#[test]
fn frontier_exhaustion_stops_cleanly() {
    let exp = expansions(vec![]);
    let outcome = run(
        &seed_opp(),
        &exp,
        &bounds(),
        2,
        &AllocationPolicy::default(),
    );
    assert_eq!(outcome.stop, Some(StopReason::FrontierExhausted));
    assert_eq!(outcome.nodes.len(), 1, "only the seed");
}
