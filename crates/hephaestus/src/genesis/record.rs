//! Genesis mechanism records (T-015): the mechanism contract that T-016's
//! hypothesis compiler builds on.
//!
//! A `MechanismRecord` states entities, relevant variables, causal or
//! computational relationships, prerequisites, expected effects, operating
//! regime, failure modes, and a minimal realization (MASTER_SPEC §8:158).
//! Placeholder language — "use AI," "add a graph," "make it adaptive" — is
//! refused at construction (R-022/AT-022): it names no mechanism.

use serde::{Deserialize, Serialize};

/// Why mechanism construction refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MechanismError {
    /// The realization names no mechanism (placeholder language).
    PlaceholderLanguage(String),
    /// Realization or required fields empty: nothing is stated.
    Empty,
}

impl std::fmt::Display for MechanismError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlaceholderLanguage(p) => write!(
                f,
                "'{p}' names no mechanism: a mechanism record must state entities, \
                 variables, relationships, and a minimal realization (MASTER_SPEC §8)"
            ),
            Self::Empty => write!(
                f,
                "mechanism record requires a realization and relationships"
            ),
        }
    }
}

impl std::error::Error for MechanismError {}

/// Provenance: which operator produced the record, from which opportunity,
/// at which operator version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub operator: String,
    pub operator_version: u32,
    pub source_opportunity: String,
}

/// Advisory plausibility flag (R-024): may prioritize or flag, NEVER
/// eliminates. Qualitative with a reason — no invented numbers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlausibilityFlag {
    /// Advisory qualifier, e.g. `implausible-but-valid`.
    pub qualifier: String,
    pub reason: String,
}

/// The mechanism record (MASTER_SPEC §8:158): every field present; the
/// constructor refuses placeholder language and empty realizations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechanismRecord {
    /// One-line statement of the mechanism (the realization's core claim).
    pub statement: String,
    /// Relevant variables.
    pub variables: Vec<String>,
    /// Causal or computational relationships between variables.
    pub relationships: Vec<String>,
    /// Prerequisites that must hold for the mechanism to operate.
    pub prerequisites: Vec<String>,
    /// Expected effects when operating.
    pub expected_effects: Vec<String>,
    /// Operating regime: where the mechanism is expected to hold.
    pub operating_regime: String,
    /// Failure modes.
    pub failure_modes: Vec<String>,
    /// Minimal realization.
    pub realization: String,
    /// Provenance: operator + version + source opportunity.
    pub provenance: Provenance,
    /// Advisory plausibility flag: present means flagged, never filtered
    /// (R-024/AT-024).
    pub plausibility: Option<PlausibilityFlag>,
}

/// Placeholder phrases that name no mechanism (MASTER_SPEC:158 negative
/// examples). Matched case-insensitively as substrings.
const PLACEHOLDER_PHRASES: [&str; 3] = ["use ai", "add a graph", "make it adaptive"];

impl MechanismRecord {
    /// Construct with the full §8:158 field set. Refuses placeholder
    /// language and empty statements (fail closed on the mechanism
    /// contract).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        statement: String,
        variables: Vec<String>,
        relationships: Vec<String>,
        prerequisites: Vec<String>,
        expected_effects: Vec<String>,
        operating_regime: String,
        failure_modes: Vec<String>,
        realization: String,
    ) -> Result<Self, MechanismError> {
        let lowered = statement.to_lowercase();
        for phrase in PLACEHOLDER_PHRASES {
            if lowered.contains(phrase) {
                return Err(MechanismError::PlaceholderLanguage(statement));
            }
        }
        if statement.trim().is_empty() || relationships.is_empty() || realization.trim().is_empty()
        {
            return Err(MechanismError::Empty);
        }
        Ok(Self {
            statement,
            variables,
            relationships,
            prerequisites,
            expected_effects,
            operating_regime,
            failure_modes,
            realization,
            // Provenance is stamped by the registry after construction.
            provenance: Provenance {
                operator: String::new(),
                operator_version: 0,
                source_opportunity: String::new(),
            },
            plausibility: None,
        })
    }
}
