//! Schema-version migration registry (T-002).
//!
//! `migrate_to_current` refuses to guess. Only versions with an explicitly
//! registered step are transformed; anything else — including schema 1.0 —
//! fails closed with a reason (see [`MigrationError::NotRegistered`]).

use serde_json::{Value, json};

use crate::contracts::{ContractError, ContractRecord, SCHEMA_VERSION};

/// Why a record could not be migrated into the current schema.
#[derive(Debug)]
pub enum MigrationError {
    /// `schema_version` is not a version this registry knows at all
    /// (e.g. `0.9`, `1.3`, `2.0`, or a non-string/missing value is
    /// [`MigrationError::Malformed`]).
    UnsupportedVersion(String),
    /// A known older version has no honest path to the current schema.
    NotRegistered {
        from: String,
        kind: String,
        reason: &'static str,
    },
    /// The record is missing the fields needed to even dispatch a migration.
    Malformed(String),
    /// Migration ran but the resulting record failed contract validation.
    Contract(ContractError),
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::UnsupportedVersion(v) => {
                write!(f, "unsupported schema_version: {v}")
            }
            MigrationError::NotRegistered { from, kind, reason } => {
                write!(
                    f,
                    "no registered migration {from} -> current for kind {kind}: {reason}"
                )
            }
            MigrationError::Malformed(m) => write!(f, "cannot migrate malformed record: {m}"),
            MigrationError::Contract(e) => write!(f, "migrated record failed validation: {e}"),
        }
    }
}

impl std::error::Error for MigrationError {}

impl From<ContractError> for MigrationError {
    fn from(value: ContractError) -> Self {
        MigrationError::Contract(value)
    }
}

/// A single registered schema-version step: `(from) -> current` transform.
type Step = fn(Value) -> Result<Value, MigrationError>;

/// 1.1 -> 1.2 is a pure retag: the shared record definitions are deep-equal
/// between the archived 1.1 schema and the current one (verified when the
/// registry was written), so the only change is `schema_version`.
const STEP_1_1_TO_1_2: Step = |mut value| {
    value["schema_version"] = json!(SCHEMA_VERSION);
    Ok(value)
};

/// Why schema 1.0 has no registered migration, cited by the error path.
const REASON_1_0: &str = "1.0 -> 1.1 added required non-nullable bindings \
(evidence_snapshot_sha256, guardrail_ids minItems 1, operation_id) that no 1.0 \
record honestly contains; synthesizing them would fabricate evidence. ADR-013 \
requires explicit migration receipts, which need the runtime receipt \
infrastructure (T-011+). Fail closed instead of guessing.";

/// Versions that have at least one registered step.
const REGISTERED_FROM: &[&str] = &["1.1"];

fn step_for(from: &str) -> Option<Step> {
    match from {
        "1.1" => Some(STEP_1_1_TO_1_2),
        _ => None,
    }
}

/// Migrate a single record value of any registered schema version into the
/// current contract, then parse and validate it as a [`ContractRecord`].
///
/// * Unknown versions are rejected ([`MigrationError::UnsupportedVersion`]).
/// * Known versions without an honest registered step are rejected with a
///   reason ([`MigrationError::NotRegistered`]) — never silently reinterpreted.
/// * A success is always a fully validated current-schema record.
pub fn migrate_to_current(mut value: Value) -> Result<ContractRecord, MigrationError> {
    let mut hops = 0usize;
    loop {
        let version = value
            .get("schema_version")
            .and_then(Value::as_str)
            .ok_or_else(|| MigrationError::Malformed("missing schema_version".to_string()))?
            .to_string();
        if version == SCHEMA_VERSION {
            break;
        }
        let kind = value
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("<missing kind>")
            .to_string();
        if version == "1.0" {
            return Err(MigrationError::NotRegistered {
                from: version,
                kind,
                reason: REASON_1_0,
            });
        }
        let Some(step) = step_for(&version) else {
            return Err(MigrationError::UnsupportedVersion(version));
        };
        debug_assert!(REGISTERED_FROM.contains(&version.as_str()));
        value = step(value)?;
        hops += 1;
        if hops > 8 {
            return Err(MigrationError::Malformed(
                "migration registry did not converge".to_string(),
            ));
        }
    }
    ContractRecord::from_value(value).map_err(MigrationError::Contract)
}

/// Migrate every record of a bundle and return the parsed current records.
pub fn migrate_bundle(value: &Value) -> Result<Vec<ContractRecord>, MigrationError> {
    let records = value
        .get("records")
        .and_then(Value::as_array)
        .ok_or_else(|| MigrationError::Malformed("bundle has no records array".to_string()))?;
    records.iter().cloned().map(migrate_to_current).collect()
}
