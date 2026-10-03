//! Amendment gates + holdout discipline (T-035, R-041/AT-041 (family
//! binding preserves selection lineage, family identity, and error
//! allocation; a challenger needs a fresh or separately qualified
//! holdout; exposure stays monotonic)).
//!
//! Integration tests at the public seam: `check_migration`,
//! `record_access`, `admit_for_challenger`, `check_family`.

use hephaestus::amendment::record::{Holdout, HoldoutError, MigrationError, MigrationGates};
use hephaestus::amendment::{admit_for_challenger, check_family, check_migration, record_access};

fn s(v: &str) -> String {
    v.to_string()
}

fn gates() -> MigrationGates {
    MigrationGates {
        old_version: s("domainpack-1.0.0"),
        new_version: s("domainpack-1.1.0"),
        schema_compatible: true,
        policy_migration_declared: true,
    }
}

fn holdout(exposed: bool, fresh: bool, qualified: bool) -> Holdout {
    Holdout {
        holdout_id: s("holdout-1"),
        exposed,
        fresh,
        separately_qualified: qualified,
        last_accessor: None,
    }
}

#[test]
fn migration_requires_declared_gates_and_preserves_identity() {
    // Declared gates pass: receipt with old identity verbatim.
    let r = check_migration(&gates()).expect("receipt");
    assert_eq!(r.old_version, "domainpack-1.0.0");
    assert_eq!(r.new_version, "domainpack-1.1.0");
    // Schema gate failed: refused.
    let mut g = gates();
    g.schema_compatible = false;
    assert_eq!(check_migration(&g), Err(MigrationError::SchemaIncompatible));
    // Policy migration undeclared: refused.
    let mut g = gates();
    g.policy_migration_declared = false;
    assert_eq!(
        check_migration(&g),
        Err(MigrationError::PolicyMigrationUndeclared)
    );
    // Receipt is immutable by construction: no mutation API; fields
    // preserve the transition exactly.
    let r2 = check_migration(&gates()).unwrap();
    assert_eq!(r, r2);
}

#[test]
fn exposure_is_monotonic() {
    let h = holdout(false, true, false);
    let after = record_access(&h, "campaign-7").expect("accessed");
    assert!(after.exposed);
    assert!(!after.fresh, "access makes the holdout non-fresh");
    assert_eq!(after.last_accessor.as_deref(), Some("campaign-7"));
    // Further access keeps exposure (monotonic; never reset to sealed).
    let again = record_access(&after, "campaign-8").expect("accessed again");
    assert!(again.exposed);
}

#[test]
fn challenger_evaluation_requires_fresh_or_qualified_holdout() {
    // Fresh, unexposed: admissible.
    assert!(admit_for_challenger(&holdout(false, true, false)).is_ok());
    // Exposed but separately qualified: admissible (fresh OR qualified).
    assert!(admit_for_challenger(&holdout(true, false, true)).is_ok());
    // Exposed without separate qualification: refused (stale).
    assert_eq!(
        admit_for_challenger(&holdout(true, false, false)),
        Err(HoldoutError::StaleHoldout)
    );
}

#[test]
fn family_binding_requires_lineage_qualification_and_allocation() {
    let ok = hephaestus::amendment::record::FamilyBinding {
        family_id: s("family-1"),
        selection_lineage: vec![s("campaign-3"), s("campaign-7")],
        method_qualification: s("method-registry:qualified-m-1"),
        error_allocation: 0.05,
    };
    assert!(check_family(&ok).is_ok());
    // Empty lineage refused.
    let mut b = ok.clone();
    b.selection_lineage.clear();
    assert!(check_family(&b).is_err());
    // Missing method qualification refused.
    let mut b = ok.clone();
    b.method_qualification = s("  ");
    assert!(check_family(&b).is_err());
    // Undeclared/out-of-range error allocation refused.
    let mut b = ok.clone();
    b.error_allocation = 0.0;
    assert!(check_family(&b).is_err());
    let mut b = ok;
    b.error_allocation = 1.0;
    assert!(check_family(&b).is_err());
}

#[test]
fn twin_run_byte_identical() {
    let a = check_migration(&gates()).unwrap();
    let b = check_migration(&gates()).unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
