//! Prototype records (T-020, MASTER_SPEC §15:296-297).

use serde::{Deserialize, Serialize};

/// The protected context (R-095): evaluator + baseline identities and the
/// registry of verified artifact digests. Trust flows from here, never
/// from worker claims.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProtectedContext {
    /// Digest of the protected evaluator implementation.
    pub evaluator_digest: String,
    /// Digest of the protected baseline artifact.
    pub baseline_digest: String,
    /// Artifact digests the protected side has verified.
    pub verified_artifacts: Vec<String>,
}

impl ProtectedContext {
    pub fn verifies(&self, digest: &str) -> bool {
        self.verified_artifacts.iter().any(|d| d == digest)
    }
}

/// A build receipt (R-094): artifact digest plus recorded command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildReceipt {
    pub artifact_digest: String,
    pub command: String,
}

/// A test receipt (R-094): tested artifact digest plus outcome record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestReceipt {
    pub artifact_digest: String,
    pub passed: bool,
    pub record: String,
}

/// A worker-proposed change: the ONLY thing a worker may propose - a
/// candidate artifact swap. Claims are assertions, never trust (R-095).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrototypeChange {
    /// Which artifact the change replaces (candidate-scoped path).
    pub target_artifact: String,
    pub new_artifact_digest: String,
    /// Claim IDs this change is supposed to affect.
    pub claim_ids: Vec<String>,
    pub build: BuildReceipt,
    pub test: TestReceipt,
}

/// Why authorization refused, retained (R-024 pattern: structured
/// reason, never a silent drop).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rejection {
    /// The change targets the protected evaluator.
    ProtectedEvaluatorTarget,
    /// The change targets the protected baseline.
    ProtectedBaselineTarget,
    /// The receipt's digest is not in the protected registry (R-095).
    UnverifiedArtifactDigest,
    /// Build or test receipt missing/failed.
    ReceiptFailure,
}

/// Authorization outcome (R-094): a receipt binding the accepted change
/// to verified bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationReceipt {
    pub target_artifact: String,
    pub artifact_digest: String,
    pub claim_ids: Vec<String>,
    /// Digest of the evaluator this change was authorized AGAINST - the
    /// change is void if the evaluator moves.
    pub against_evaluator_digest: String,
}

/// A component in an assembly (§15:297).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub version: String,
    /// Interface signatures this component provides.
    pub provides: Vec<String>,
    /// Interface signatures this component requires.
    pub requires: Vec<String>,
    /// Declared resource budget (arbitrary comparable unit).
    pub resource_budget: u64,
    /// Named global invariants this component upholds.
    pub invariants: Vec<String>,
}

/// One assembly check result, with the failing component named.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssemblyFinding {
    /// Which component/check produced this finding.
    pub component: String,
    pub check: String,
    pub ok: bool,
    pub detail: String,
}

/// The full assembly report (§15:297).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssemblyReport {
    pub findings: Vec<AssemblyFinding>,
    pub total_resource_budget: u64,
    /// Cost-shift accounting: per-component budget share of the total
    /// (an optimization shifting cost to another stage must be visible).
    pub cost_shift_rows: Vec<(String, u64)>,
    pub passes: bool,
}
