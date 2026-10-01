//! Hypothesis compiler + semantic validators (T-016, R-025/026/027).
//!
//! Integration tests at the public seam: `genesis::hypothesis::compile` /
//! `revise` and `genesis::validators::validate`. Test-ready status requires
//! an operational discriminator, falsifiers, competitors, and a comparator;
//! thresholds never come from the compiler; revision lineage blocks silent
//! inheritance of confirmatory support.

use hephaestus::genesis::hypothesis::{InheritedSupport, Readiness, Threshold, compile, revise};
use hephaestus::genesis::record::MechanismRecord;
use hephaestus::genesis::validators::validate;

fn s(v: &str) -> String {
    v.to_string()
}

fn mechanism() -> MechanismRecord {
    MechanismRecord::new(
        s("cache context by exact dependency hash"),
        vec![s("context_build_time"), s("dependency_hash")],
        vec![s("build_time decreases as unchanged deps skip rebuild")],
        vec![s("dependency graph extractable from imports")],
        vec![s("build_time halves on warm cache with <10% file churn")],
        s("repository-scale code changes; single-machine workloads"),
        vec![s("stale invalidation if hash omits transitive imports")],
        s("hash imports at task start; key the cache on the hash"),
    )
    .expect("genuine mechanism")
}

// ---- Ticket 01: Hypothesis record + compiler + semantic validators ----

#[test]
fn compile_produces_all_spec_fields() {
    // MASTER_SPEC:178: context, intervention, comparator, proposed
    // mechanism, measurable predictions, estimands, meaningful effect
    // bounds, boundary conditions, competing explanations, required
    // observations, analysis requirements, outcomes that would count
    // against the claim.
    let h = compile(&mechanism());
    assert!(!h.context.is_empty(), "context present");
    assert!(!h.intervention.is_empty(), "intervention present");
    assert!(!h.comparator.is_empty(), "comparator present");
    assert!(
        !h.proposed_mechanism.statement.is_empty(),
        "mechanism present"
    );
    assert!(!h.predictions.is_empty(), "predictions present");
    assert!(!h.estimands.is_empty(), "estimands present");
    assert!(
        !h.boundary_conditions.is_empty(),
        "boundary conditions present"
    );
    assert!(!h.competing_explanations.is_empty(), "competitors present");
    assert!(!h.required_observations.is_empty(), "observations present");
    assert!(
        !h.analysis_requirements.is_empty(),
        "analysis requirements present"
    );
    assert!(
        !h.falsifiers.is_empty(),
        "outcomes counting against the claim present"
    );
    // Mechanism claim and engineering target are SEPARATE (§9:180).
    assert!(
        h.engineering_target.is_none(),
        "no target invented by default"
    );
}

#[test]
fn stripped_falsifiers_or_competitors_denies_test_ready() {
    // AT-025 negative: removing falsifiers and competing explanations
    // denies test-ready promotion.
    let mut h = compile(&mechanism());
    h.falsifiers.clear();
    let verdict = validate(&h);
    assert!(
        !matches!(verdict.readiness, Readiness::TestReady),
        "no falsifiers -> not test-ready"
    );
    assert!(
        verdict.denials.iter().any(|d| d.contains("falsifier")),
        "denial names the falsifier requirement: {:?}",
        verdict.denials
    );

    let mut h2 = compile(&mechanism());
    h2.competing_explanations.clear();
    let verdict2 = validate(&h2);
    assert!(
        !matches!(verdict2.readiness, Readiness::TestReady),
        "no competitors -> not test-ready"
    );
    assert!(
        verdict2.denials.iter().any(|d| d.contains("competing")),
        "denial names the competitor requirement: {:?}",
        verdict2.denials
    );
}

#[test]
fn no_discriminator_stays_exploratory_never_deleted() {
    // MASTER_SPEC:182: "no credible discriminator, no test-ready
    // hypothesis" — but the hypothesis is preserved as EXPLORATORY.
    let h = compile(&mechanism());
    let mut untestable = h.clone();
    untestable.discriminator = None;
    let verdict = validate(&untestable);
    assert_eq!(
        verdict.readiness,
        Readiness::Exploratory,
        "no discriminator -> exploratory, preserved"
    );
}

#[test]
fn unprovenanced_threshold_is_returned_for_completion() {
    // AT-026 negative: an arbitrary target without a reason or comparator
    // is returned for completion, never accepted as justified.
    let mut h = compile(&mechanism());
    h.engineering_target = Some(Threshold {
        description: s("20% faster"),
        provenance: String::new(), // NO provenance
        comparator: String::new(),
    });
    let verdict = validate(&h);
    assert!(
        matches!(verdict.readiness, Readiness::BlockedTestability),
        "unprovenanced target -> blocked, returned for completion"
    );
    assert!(
        verdict.denials.iter().any(|d| d.contains("provenance")),
        "denial names threshold provenance: {:?}",
        verdict.denials
    );
}

#[test]
fn provenanced_threshold_is_accepted() {
    let mut h = compile(&mechanism());
    h.engineering_target = Some(Threshold {
        description: s("20% faster than strongest baseline"),
        provenance: s("user requirement R-0xx; explicitly provisional design choice"),
        comparator: s("strongest implemented baseline, same fixture"),
    });
    let verdict = validate(&h);
    assert!(
        !matches!(verdict.readiness, Readiness::BlockedTestability),
        "provenanced target accepted: {:?}",
        verdict.denials
    );
}

#[test]
fn self_fulfilling_metric_is_denied() {
    // MASTER_SPEC:184: "Does the hypothesis merely restate its own
    // success metric?"
    let mut h = compile(&mechanism());
    h.falsifiers = vec![s("the cache is faster, which would mean the cache works")];
    h.competing_explanations = vec![s("measurement noise")];
    let verdict = validate(&h);
    assert!(
        !matches!(verdict.readiness, Readiness::TestReady),
        "self-fulfilling falsifier denied"
    );
    assert!(
        verdict.denials.iter().any(|d| d.contains("own success")),
        "denial names the self-fulfilling check: {:?}",
        verdict.denials
    );
}

#[test]
fn complete_hypothesis_validates_test_ready() {
    let h = compile(&mechanism());
    let verdict = validate(&h);
    assert_eq!(
        verdict.readiness,
        Readiness::TestReady,
        "complete compiled hypothesis passes: {:?}",
        verdict.denials
    );
}

#[test]
fn twin_run_compile_byte_identical() {
    let a = compile(&mechanism());
    let b = compile(&mechanism());
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
    // InheritedSupport is part of the lineage contract (used by ticket 02).
    let _ = InheritedSupport::Inherits;
}

// ---- Ticket 02: revision lineage (R-027/AT-027) ----

#[test]
fn revise_mints_new_version_with_lineage() {
    let h1 = compile(&mechanism());
    let mut h2 = h1.clone();
    h2.context = s("revised context after pilot");
    let revised = revise(&h1, h2);
    assert_eq!(revised.version, 2);
    assert_eq!(revised.supersedes, Some(1));
    assert_eq!(revised.inherited_support, InheritedSupport::Inherits);
}

#[test]
fn changed_mechanism_blocks_silent_inheritance() {
    // AT-027 negative: change a mechanism after observing favorable
    // results — the new version cannot inherit confirmatory support
    // silently.
    let h1 = compile(&mechanism());
    let mut h2 = h1.clone();
    h2.proposed_mechanism.statement = s("cache context by filename mtime instead");
    let revised = revise(&h1, h2);
    assert_eq!(
        revised.inherited_support,
        InheritedSupport::NeedsReassessment
    );
    // And semantic validation refuses test-ready on un-reassessed support.
    let verdict = validate(&revised);
    assert!(
        !matches!(verdict.readiness, Readiness::TestReady),
        "un-reassessed substantive change cannot promote"
    );
    assert!(
        verdict.denials.iter().any(|d| d.contains("reassessment")),
        "denial names the reassessment requirement: {:?}",
        verdict.denials
    );
}

#[test]
fn changed_falsifier_or_endpoint_or_boundary_blocks_too() {
    let h1 = compile(&mechanism());
    // Falsifier change
    let mut h2 = h1.clone();
    h2.falsifiers = vec![s("different falsifier")];
    assert_eq!(
        revise(&h1, h2).inherited_support,
        InheritedSupport::NeedsReassessment
    );
    // Boundary change
    let mut h3 = h1.clone();
    h3.boundary_conditions = vec![s("different regime")];
    assert_eq!(
        revise(&h1, h3).inherited_support,
        InheritedSupport::NeedsReassessment
    );
    // Endpoint (engineering target) change
    let mut h4 = h1.clone();
    h4.engineering_target = Some(Threshold {
        description: s("30% faster"),
        provenance: s("prior study X"),
        comparator: s("baseline Y"),
    });
    assert_eq!(
        revise(&h1, h4).inherited_support,
        InheritedSupport::NeedsReassessment
    );
}

#[test]
fn revise_requires_revalidation_and_twin_run_identical() {
    let h1 = compile(&mechanism());
    let mut h2 = h1.clone();
    h2.context = s("clarified context");
    let r1 = revise(&h1, h2.clone());
    let r2 = revise(&h1, h2);
    assert_eq!(
        serde_json::to_string(&r1).unwrap(),
        serde_json::to_string(&r2).unwrap()
    );
    // Revised readiness is cleared: re-validated, never carried.
    assert_eq!(r1.readiness, None);
}
