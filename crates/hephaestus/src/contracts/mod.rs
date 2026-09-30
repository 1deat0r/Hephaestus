//! Typed contract records for the Hephaestus runtime (T-002).
//!
//! `generated.rs` is produced from `schemas/contracts.schema.json` by
//! `tools/generate_contracts_rs.py`; never edit it by hand. Structural
//! deserialization (required fields, enums, const strings, unknown-field
//! rejection) happens in serde; `validate()` adds format, pattern, bound and
//! uniqueness checks. [`ContractRecord::from_value`] gates on the current
//! schema version and rejects anything unsupported.

pub mod generated;

pub use generated::{ContractRecord, Money, RecordRef, SCHEMA_VERSION, SchemaVersion};

use std::fmt;

/// A single contract violation found by generated `validate()` functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractViolation {
    /// Dotted path to the offending field, e.g. `analysis.target_n`.
    pub path: String,
    /// Human-readable reason, e.g. `pattern: "^[a-f0-9]{64}$" failed`.
    pub message: String,
}

impl fmt::Display for ContractViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

/// Errors produced when parsing or validating a contract record.
#[derive(Debug)]
pub enum ContractError {
    /// The JSON value was not a record of a known kind.
    UnknownKind(String),
    /// `schema_version` was present but not the supported version.
    /// Unsupported versions are rejected, never guessed at (T-002).
    UnsupportedSchemaVersion(String),
    /// A required envelope field was missing or malformed.
    MalformedRecord(String),
    /// serde failed to deserialize the record structure.
    Json(serde_json::Error),
    /// Structural parse succeeded but contract validation failed.
    Invalid(Vec<ContractViolation>),
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractError::UnknownKind(k) => write!(f, "unknown record kind: {k}"),
            ContractError::UnsupportedSchemaVersion(v) => {
                write!(
                    f,
                    "unsupported schema_version: {v} (supported: {SCHEMA_VERSION})"
                )
            }
            ContractError::MalformedRecord(m) => write!(f, "malformed record: {m}"),
            ContractError::Json(e) => write!(f, "record JSON invalid: {e}"),
            ContractError::Invalid(violations) => {
                write!(
                    f,
                    "contract validation failed with {} violation(s)",
                    violations.len()
                )
            }
        }
    }
}

impl std::error::Error for ContractError {}

impl ContractError {
    /// All violations if and only if the error is [`ContractError::Invalid`].
    pub fn violations(&self) -> &[ContractViolation] {
        match self {
            ContractError::Invalid(v) => v,
            _ => &[],
        }
    }
}

/// Parse every record of a bundle (`{"records": [...]}` or a bare array).
pub fn parse_records(value: &serde_json::Value) -> Result<Vec<ContractRecord>, ContractError> {
    let array = value
        .get("records")
        .or(Some(value))
        .filter(|v| v.is_array())
        .and_then(|v| v.as_array())
        .ok_or_else(|| ContractError::MalformedRecord("bundle has no records array".to_string()))?;
    array
        .iter()
        .cloned()
        .map(ContractRecord::from_value)
        .collect()
}

/// Parse a bundle from a JSON string.
pub fn parse_bundle_json(text: &str) -> Result<Vec<ContractRecord>, ContractError> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(ContractError::Json)?;
    parse_records(&value)
}
