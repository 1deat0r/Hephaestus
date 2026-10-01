//! Method registry + preregistration (T-021, R-100/R-103, campaign M3).

use hephaestus::methods::record::{
    ClippingPolicy, Estimand, MethodSpec, MissingnessPolicy, RegistryError, bonferroni_alpha,
    zero_failure_bound,
};
use hephaestus::methods::{CampaignManifest, PreregError, ProtectedRegistry, preregister};

fn s(v: &str) -> String {
    v.to_string()
}

/// The campaign's paired repository-level fixed-sample method.
fn paired_method() -> MethodSpec {
    MethodSpec {
        name: s("paired-repo-difference"),
        version: s("1.0.0"),
        estimand: Estimand {
            quantity: s("difference in useful-outcome yield between arms"),
            unit: s("repository-level"),
            denominator: s("all assigned missions incl. failed/blocked/inconclusive"),
            cost_fields: vec![
                s("acquisition"),
                s("generation"),
                s("failed_invalid_blocked_runs"),
                s("evaluator_and_reproduction"),
                s("human_intervention"),
            ],
        },
        assumptions: vec![s("paired independence across repositories")],
        alpha_allocations: bonferroni_alpha(2, 0.05),
        missingness: MissingnessPolicy::FailedOrMissingIsFailure,
        clipping: ClippingPolicy {
            lower: Some(0.0),
            upper: Some(1.0),
            report_clipped_rates: true,
        },
        implementation_digest: s("impl-digest-1"),
    }
}

fn manifest() -> CampaignManifest {
    CampaignManifest {
        n: Some(299),
        method_name: Some(s("paired-repo-difference")),
        method_version: Some(s("1.0.0")),
        oracle_resolved: true,
        n_calculation_reference: Some(s("pilot variance bound, doc section 4")),
        alpha_allocation: Some(0.025),
        reference_distribution: Some(s("normal approximation, paired")),
        power_precision_target: Some(s("80% power at owner-smallest effect")),
        sensitivity_assumptions: Some(s("sensitivity to dependence across episodes")),
    }
}

// ---- Ticket 01: registry, allocation, zero-failure bound ----

#[test]
fn registry_binds_all_spec_fields_and_rejects_duplicates() {
    let registry = ProtectedRegistry::new();
    registry
        .register(paired_method())
        .expect("first registration ok");
    // Duplicate (name, version) refused: entries immutable.
    assert_eq!(
        registry.register(paired_method()),
        Err(RegistryError::DuplicateMethodVersion)
    );
    // A change is a NEW version.
    let mut v2 = paired_method();
    v2.version = s("1.1.0");
    registry.register(v2).expect("new version ok");
    let fetched = registry
        .qualified("paired-repo-difference", "1.0.0")
        .expect("qualified");
    assert_eq!(fetched.estimand.unit, "repository-level");
    assert_eq!(
        fetched.missingness,
        MissingnessPolicy::FailedOrMissingIsFailure
    );
    assert!(fetched.clipping.report_clipped_rates);
    assert_eq!(fetched.implementation_digest, "impl-digest-1");
}

#[test]
fn registry_rejects_missing_identity_and_bad_allocations() {
    let registry = ProtectedRegistry::new();
    let mut spec = paired_method();
    spec.implementation_digest = String::new();
    assert_eq!(
        registry.register(spec),
        Err(RegistryError::MissingImplementationIdentity)
    );
    let mut spec = paired_method();
    spec.alpha_allocations = vec![-0.1, 0.05];
    assert_eq!(
        registry.register(spec),
        Err(RegistryError::InvalidAllocation)
    );
}

#[test]
fn bonferroni_splits_equally() {
    let shares = bonferroni_alpha(4, 0.05);
    assert_eq!(shares.len(), 4);
    let sum: f64 = shares.iter().sum();
    assert!((sum - 0.05).abs() < 1e-12, "sums to the family alpha");
    assert!(shares.iter().all(|a| *a == 0.0125));
}

#[test]
fn zero_failure_bound_matches_campaign_example() {
    // Campaign: n=299, alpha=0.05 -> bound <= 0.01. Planning-only.
    let bound = zero_failure_bound(299, 0.05);
    assert!(bound <= 0.01, "bound {bound} exceeds 1%");
    // Sanity: more samples tighten the bound.
    assert!(zero_failure_bound(1000, 0.05) < bound);
}

// ---- Ticket 02: preregistration gate ----

#[test]
fn preregistration_refusals_are_named() {
    let registry = ProtectedRegistry::new();
    registry.register(paired_method()).unwrap();
    // n = null refused.
    let mut m = manifest();
    m.n = None;
    assert_eq!(preregister(&m, &registry), Err(PreregError::NullSampleSize));
    // Unqualified method refused.
    let mut m = manifest();
    m.method_version = Some(s("9.9.9"));
    assert_eq!(
        preregister(&m, &registry),
        Err(PreregError::UnqualifiedMethod)
    );
    // Unresolved oracle refused.
    let mut m = manifest();
    m.oracle_resolved = false;
    assert_eq!(
        preregister(&m, &registry),
        Err(PreregError::UnresolvedOracle)
    );
    // Missing MUST-include entries refused by name.
    for strip in [
        |m: &mut CampaignManifest| m.n_calculation_reference = None,
        |m: &mut CampaignManifest| m.alpha_allocation = None,
        |m: &mut CampaignManifest| m.reference_distribution = None,
        |m: &mut CampaignManifest| m.power_precision_target = None,
        |m: &mut CampaignManifest| m.sensitivity_assumptions = None,
    ] {
        let mut m = manifest();
        strip(&mut m);
        assert!(
            preregister(&m, &registry).is_err(),
            "must-include gap refused"
        );
    }
}

#[test]
fn complete_manifest_freezes_immutable_preregistration() {
    let registry = ProtectedRegistry::new();
    registry.register(paired_method()).unwrap();
    let pre = preregister(&manifest(), &registry).expect("gates");
    assert_eq!(pre.n, 299);
    assert_eq!(pre.method.name, "paired-repo-difference");
    assert_eq!(pre.alpha_allocation, 0.025);
    // The frozen record carries the manifest verbatim.
    assert_eq!(pre.manifest, manifest());
}

#[test]
fn twin_run_byte_identical() {
    let registry = ProtectedRegistry::new();
    registry.register(paired_method()).unwrap();
    let a = preregister(&manifest(), &registry).unwrap();
    let b = preregister(&manifest(), &registry).unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
