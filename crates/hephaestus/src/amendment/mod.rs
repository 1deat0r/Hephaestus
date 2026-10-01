//! Amendment row T-032/T-033 (T-035, IMPLEMENTATION_PLAN:138).
//!
//! New-domain validity/migration gates and fresh or separately
//! qualified holdouts for challenger evaluation: explicit transitions
//! with immutable historical identities (R-114), monotonic sealed-data
//! exposure (R-101), and typed experiment families with declared error
//! allocation (R-100).

pub mod record;

pub use record::{
    FamilyBinding, Holdout, HoldoutError, MigrationError, MigrationGates, MigrationReceipt,
};

/// Check an explicit version transition (R-114): both declared gates
/// must pass; the receipt preserves the old identity verbatim and is
/// immutable by construction (no mutation API exists).
pub fn check_migration(gates: &MigrationGates) -> Result<MigrationReceipt, MigrationError> {
    if !gates.schema_compatible {
        return Err(MigrationError::SchemaIncompatible);
    }
    if !gates.policy_migration_declared {
        return Err(MigrationError::PolicyMigrationUndeclared);
    }
    Ok(MigrationReceipt {
        old_version: gates.old_version.clone(),
        new_version: gates.new_version.clone(),
        schema_compatible: gates.schema_compatible,
        policy_migration_declared: gates.policy_migration_declared,
    })
}

/// Record sealed-data access (R-101): exposure is monotonic — an
/// exposed holdout can never return to sealed. Returns the updated
/// holdout state.
pub fn record_access(holdout: &Holdout, accessor: &str) -> Result<Holdout, HoldoutError> {
    Ok(Holdout {
        exposed: true,
        fresh: false,
        separately_qualified: holdout.separately_qualified,
        holdout_id: holdout.holdout_id.clone(),
        last_accessor: Some(accessor.to_string()),
    })
}

/// Admit a holdout for challenger evaluation: only FRESH holdouts or
/// separately qualified ones are admissible; an exposed holdout without
/// separate qualification is refused (no evaluation on data the
/// selection process already saw — R-100/R-101).
pub fn admit_for_challenger(holdout: &Holdout) -> Result<(), HoldoutError> {
    if holdout.exposed && !holdout.separately_qualified {
        return Err(HoldoutError::StaleHoldout);
    }
    Ok(())
}

/// Check a family binding (R-100): lineage present, method qualified,
/// error allocation declared before evaluation (positive, finite).
pub fn check_family(binding: &FamilyBinding) -> Result<(), String> {
    if binding.selection_lineage.is_empty() {
        return Err("empty selection lineage".to_string());
    }
    if binding.method_qualification.trim().is_empty() {
        return Err("method qualification missing".to_string());
    }
    if !(binding.error_allocation > 0.0 && binding.error_allocation < 1.0) {
        return Err("error allocation must be declared in (0, 1)".to_string());
    }
    Ok(())
}
