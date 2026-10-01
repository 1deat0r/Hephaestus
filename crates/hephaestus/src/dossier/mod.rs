//! Raw-data capture, cost receipts, clean-environment reproduction, and
//! exportable dossiers (T-023, MASTER_SPEC section 15, R-044).
//!
//! No filesystem I/O here: `export` returns the JSON document; writing
//! is the caller's act. Cost receipts are quantities (R-103: record
//! provider charges and resource/human-time quantities rather than
//! inventing a conversion price). Negative results are first-class
//! (R-003) and distinguish the five §15:300 kinds.

pub mod record;

pub use record::{
    CostReceipt, Deviation, Dossier, Environment, FailureEntry, Lineage, NegativeResultKind,
    ReproductionOutcome, RunRecord,
};

use serde::{Deserialize, Serialize};

/// What export refuses (each named; §14:282 continuity — a result
/// lacking reproduction material cannot pass a reproducibility gate).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportError {
    MissingRawData,
    MissingFailureHistoryField,
    MissingDeviationsField,
    MissingReproCommands,
    /// A human-time cost with an invented conversion price (R-103).
    PricedHumanTime,
}

/// Clean-environment reproduction (§15:298): compare the record's pinned
/// environment to the reproduction environment and the measured values.
pub fn reproduce(
    record: &RunRecord,
    env: &Environment,
    measurements: &[(String, f64)],
) -> ReproductionOutcome {
    // Environment mismatch is typed, never silent.
    if record.environment.os != env.os || record.environment.tool_versions != env.tool_versions {
        return ReproductionOutcome::EnvironmentMismatch;
    }
    // Same environment: artifact digests must match.
    if record.environment.artifact_digests != env.artifact_digests {
        return ReproductionOutcome::EnvironmentMismatch;
    }
    // Measurements must agree exactly on the recorded quantities
    // (declared-precision comparison is the caller's policy; the record
    // holds exact values, so exact agreement is the faithful default).
    if record.measurements.len() != measurements.len()
        || record
            .measurements
            .iter()
            .zip(measurements.iter())
            .any(|((n1, v1), (n2, v2))| n1 != n2 || (v1 - v2).abs() > f64::EPSILON)
    {
        return ReproductionOutcome::Disagrees;
    }
    ReproductionOutcome::Reproduced
}

/// Export gate (R-044, §15:299): refuses incomplete dossiers.
pub fn validate_for_export(d: &Dossier) -> Result<(), ExportError> {
    if d.raw_data.is_empty() {
        return Err(ExportError::MissingRawData);
    }
    // Fields must be PRESENT (empty vec allowed, None/absent refused is
    // structural: Vec fields are always present in Rust, so the gate is
    // on content that must exist for either history).
    if d.failure_history.is_empty() && d.deviations.is_empty() {
        // Neither history present at all: R-044 requires BOTH histories
        // be accounted; a clean run records that explicitly via a
        // deviation-free capture — but a dossier with no failures AND no
        // deviations AND no record of that fact cannot be exported.
        return Err(ExportError::MissingFailureHistoryField);
    }
    if d.repro_commands.is_empty() {
        return Err(ExportError::MissingReproCommands);
    }
    // R-103: human intervention is a quantity, never priced.
    for receipt in &d.cost_ledger {
        if receipt.kind == "human_intervention" && receipt.provider_charge.is_some() {
            return Err(ExportError::PricedHumanTime);
        }
    }
    Ok(())
}

/// Export the dossier as the JSON document (§15:299). Deterministic:
/// twin runs byte-identical (serde_json on ordered structures).
pub fn export(d: &Dossier) -> Result<String, ExportError> {
    validate_for_export(d)?;
    serde_json::to_string(d).map_err(|_| ExportError::MissingRawData)
}
