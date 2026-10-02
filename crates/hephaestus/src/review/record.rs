//! Review-authority records (T-047, MASTER_SPEC §12: R-034/R-035/R-036).

use serde::{Deserialize, Serialize};

/// A reviewer's stance on a subject (R-034).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteStance {
    Approve,
    Reject,
    Abstain,
}

/// What a review vote IS: a judgment, never empirical validation
/// (R-034). Fixed at construction — there is no other basis a vote can
/// carry, and a vote record never becomes an evidence record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteBasis {
    Judgment,
}

/// One reviewer's judgment on a subject (R-034/AT-034). Deliberately
/// carries no numeric confidence, posterior, or score: a vote is an
/// opinion record, not a measurement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewVote {
    pub reviewer_id: String,
    pub stance: VoteStance,
    pub basis: VoteBasis,
}

/// The caller-supplied result of the authoritative deterministic check
/// (R-034). The review service performs no computation of its own — it
/// only refuses to advance without a passing receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeterministicOutcome {
    Passed { receipt_digest: String },
    Failed { reason: String },
    Absent,
}

/// Stable rejection reasons from the review service (typed-rejection
/// convention, R-061 lineage).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewRejection {
    /// The deterministic checker did not pass — votes never substitute.
    Deterministic { reason: String },
    /// Open blocking objections list their ids (R-036).
    OpenObjections { ids: Vec<String> },
    /// The requesting authority may not perform this operation (R-035).
    AuthorityDenied { authority: String },
    /// Unknown objection id.
    ObjectionNotFound { id: String },
}

/// What `advance` returns when the deterministic check passed and no
/// open blocking objection remains (R-036).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvanceReceipt {
    pub subject: String,
    pub receipt_digest: String,
}

/// A version-bound review record for one subject (subject digest pins
/// the reviewed version — R-034/AT-034's version-bound evidence).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewRecord {
    pub subject: String,
    pub subject_digest: String,
    pub votes: Vec<ReviewVote>,
    pub objections: Vec<Objection>,
    pub audit: Vec<AuditEvent>,
    pub evaluator_edit_proposals: Vec<EvaluatorEditProposal>,
}

/// Who is asking (R-035): generator, analyzer, and promotion authority
/// are separated; a candidate workspace inherits generator-side limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Authority {
    Generator,
    Analyzer,
    Promoter,
    CandidateWorkspace,
}

/// An audit event for a sensitive attempted operation (R-035/AT-035):
/// every evaluator-edit attempt is recorded against the record's
/// subject digest, whether denied or registered as a proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub action: String,
    pub authority: Authority,
    pub subject_digest: String,
    pub requested: Option<String>,
    pub outcome: String,
}

/// An analyzer-scoped evaluator-edit proposal (R-035). Registration
/// only — this service never mutates any evaluator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluatorEditProposal {
    pub requested_digest: String,
    pub requested_by: Authority,
}

impl ReviewRecord {
    /// Append an audit event bound to this record's subject digest.
    pub(crate) fn audit(
        &mut self,
        action: &str,
        authority: Authority,
        requested: Option<String>,
        outcome: &str,
    ) {
        self.audit.push(AuditEvent {
            action: action.to_string(),
            authority,
            subject_digest: self.subject_digest.clone(),
            requested,
            outcome: outcome.to_string(),
        });
    }
}

/// An objection's lifecycle state (R-036): resolution sets a
/// disposition; it never deletes the objection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Disposition {
    Open,
    Resolved { note: String, resolved_by: String },
}

/// A concrete reviewer objection retained with its disposition
/// (R-036/AT-036). `blocking` objections stop `advance` while open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Objection {
    pub id: String,
    pub statement: String,
    pub raised_by: String,
    pub blocking: bool,
    pub disposition: Disposition,
}
