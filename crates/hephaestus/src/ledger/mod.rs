//! Append-only event ledger, rebuildable projections, and content-addressed
//! artifact storage (T-006).
//!
//! Three submodules, one public surface: [`EventLedger`] is the authoritative
//! history, [`Timeline`] is a derived view, [`ArtifactStore`] holds bytes.
//!
//! Durability claim: events and staged bytes survive process death via
//! `flush` + `sync_data` on their files. This is not a claim about power loss
//! across every filesystem.

mod artifact_store;
mod event_ledger;
mod timeline;

pub use artifact_store::{ArtifactStore, StagedArtifact};
pub use event_ledger::{EventLedger, Recovery};
pub use timeline::{Timeline, TimelineEntry};

use crate::contracts::ContractViolation;

/// Errors produced by ledger, projection, and store operations.
#[derive(Debug)]
pub enum LedgerError {
    /// Filesystem failure.
    Io(std::io::Error),
    /// JSON (de)serialization failure — reported distinctly from IO.
    Json {
        /// The serde failure detail.
        reason: String,
    },
    /// The event failed contract validation; nothing was written.
    InvalidEvent(Vec<ContractViolation>),
    /// A caller-supplied digest string is not a valid sha256 digest.
    BadDigest {
        /// The rejected value.
        value: String,
    },
    /// Line `line` (1-based) does not chain to its predecessor.
    ChainBroken {
        /// The 1-based line number whose link failed to verify.
        line: usize,
    },
    /// Line `line` (1-based) is present but unparseable and not trailing.
    Corrupt {
        /// The 1-based line number that failed to parse.
        line: usize,
        /// Parse failure detail.
        reason: String,
    },
    /// Bytes read back from the store did not hash to the requested digest.
    DigestMismatch {
        /// The digest that was asked for.
        expected: String,
        /// The digest the bytes actually hash to.
        actual: String,
    },
    /// The event's `sequence` is not the mission's next expected value.
    SequenceMismatch {
        /// The mission whose ordering was violated.
        mission: String,
        /// The sequence value the ledger expected.
        expected: i64,
        /// The sequence value the caller supplied.
        found: i64,
        /// The 1-based line number the event occupies or would occupy.
        line: usize,
    },
}

impl std::fmt::Display for LedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LedgerError::Io(e) => write!(f, "ledger io: {e}"),
            LedgerError::Json { reason } => write!(f, "ledger json: {reason}"),
            LedgerError::BadDigest { value } => write!(f, "invalid digest: {value}"),
            LedgerError::ChainBroken { line } => {
                write!(f, "ledger chain broken at line {line}")
            }
            LedgerError::Corrupt { line, reason } => {
                write!(f, "ledger corrupt at line {line}: {reason}")
            }
            LedgerError::DigestMismatch { expected, actual } => {
                write!(f, "digest mismatch: expected {expected}, actual {actual}")
            }
            LedgerError::SequenceMismatch {
                mission,
                expected,
                found,
                line,
            } => {
                write!(
                    f,
                    "sequence mismatch at line {line} for mission {mission}: \
                     expected {expected}, found {found}"
                )
            }
            LedgerError::InvalidEvent(v) => {
                write!(f, "invalid event ({} violations): ", v.len())?;
                for (i, violation) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{}: {}", violation.path, violation.message)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for LedgerError {}

impl From<std::io::Error> for LedgerError {
    fn from(e: std::io::Error) -> Self {
        LedgerError::Io(e)
    }
}

/// Wrap a `serde_json` failure in the ledger's JSON variant.
pub(crate) fn json_error(e: serde_json::Error) -> LedgerError {
    LedgerError::Json {
        reason: e.to_string(),
    }
}
