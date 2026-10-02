//! Review and Verification service (T-047, MASTER_SPEC §12,
//! R-034/R-035/R-036).
//!
//! Records review votes as judgments that can never substitute
//! empirical validation (R-034), separates generator/analyzer/promotion
//! authority over evaluator edits with an audit trail (R-035), and
//! retains every objection with its disposition so open blocking
//! objections stop advancement (R-036). The deterministic checker stays
//! authoritative: [`advance`] only consults a caller-supplied
//! [`DeterministicOutcome`] and open blocking objections — vote tallies
//! are structurally never read.

pub mod record;

pub use record::{
    AdvanceReceipt, AuditEvent, Authority, DeterministicOutcome, Disposition,
    EvaluatorEditProposal, Objection, ReviewRecord, ReviewRejection, VoteBasis, VoteStance,
};

/// Open a version-bound review record for one subject digest.
pub fn new_record(subject: &str, subject_digest: &str) -> ReviewRecord {
    ReviewRecord {
        subject: subject.to_string(),
        subject_digest: subject_digest.to_string(),
        votes: Vec::new(),
        objections: Vec::new(),
        audit: Vec::new(),
        evaluator_edit_proposals: Vec::new(),
    }
}

/// Record a reviewer's vote as a judgment (R-034): the basis is fixed
/// to [`VoteBasis::Judgment`] at construction.
pub fn record_vote(record: &mut ReviewRecord, reviewer_id: &str, stance: VoteStance) {
    record.votes.push(record::ReviewVote {
        reviewer_id: reviewer_id.to_string(),
        stance,
        basis: VoteBasis::Judgment,
    });
}

/// Advance a subject when the authoritative deterministic check passed
/// and no open blocking objection remains (R-034, R-036).
///
/// Vote tallies are never consulted — unanimous approval cannot bypass
/// a failed or absent deterministic check (AT-034).
pub fn advance(
    record: &ReviewRecord,
    outcome: DeterministicOutcome,
) -> Result<AdvanceReceipt, ReviewRejection> {
    match outcome {
        DeterministicOutcome::Failed { reason } => Err(ReviewRejection::Deterministic { reason }),
        DeterministicOutcome::Absent => Err(ReviewRejection::Deterministic {
            reason: "no deterministic check".to_string(),
        }),
        DeterministicOutcome::Passed { receipt_digest } => {
            let open: Vec<String> = record
                .objections
                .iter()
                .filter(|o| o.blocking && matches!(o.disposition, Disposition::Open))
                .map(|o| o.id.clone())
                .collect();
            if !open.is_empty() {
                Err(ReviewRejection::OpenObjections { ids: open })
            } else {
                Ok(AdvanceReceipt {
                    subject: record.subject.clone(),
                    receipt_digest,
                })
            }
        }
    }
}

/// Attempt an evaluator edit on behalf of `authority` (R-035).
///
/// EVERY attempt appends an [`AuditEvent`] bound to the record's
/// subject digest. Generator, Promoter, and CandidateWorkspace are
/// denied (AT-035: candidate-workspace write denied + audit recorded);
/// Analyzer only registers an [`EvaluatorEditProposal`] — this service
/// has no evaluator-mutation surface at all, so analyzer scope cannot
/// weaken the separation either.
pub fn request_evaluator_edit(
    record: &mut ReviewRecord,
    authority: Authority,
    requested_digest: &str,
) -> Result<(), ReviewRejection> {
    match authority {
        Authority::Analyzer => {
            record.audit(
                "evaluator_edit_request",
                authority,
                Some(requested_digest.to_string()),
                "registered",
            );
            record
                .evaluator_edit_proposals
                .push(record::EvaluatorEditProposal {
                    requested_digest: requested_digest.to_string(),
                    requested_by: authority,
                });
            Ok(())
        }
        Authority::Generator | Authority::Promoter | Authority::CandidateWorkspace => {
            record.audit(
                "evaluator_edit_request",
                authority,
                Some(requested_digest.to_string()),
                "denied",
            );
            Err(ReviewRejection::AuthorityDenied {
                authority: match authority {
                    Authority::Generator => "generator",
                    Authority::Promoter => "promoter",
                    Authority::CandidateWorkspace => "candidate_workspace",
                    Authority::Analyzer => unreachable!(),
                }
                .to_string(),
            })
        }
    }
}

/// Raise an objection against the subject (R-036); returns its id.
/// The objection is retained with disposition `Open`.
pub fn raise_objection(
    record: &mut ReviewRecord,
    statement: &str,
    raised_by: &str,
    blocking: bool,
) -> String {
    let id = format!("obj-{}", record.objections.len() + 1);
    record.objections.push(record::Objection {
        id: id.clone(),
        statement: statement.to_string(),
        raised_by: raised_by.to_string(),
        blocking,
        disposition: Disposition::Open,
    });
    id
}

/// Resolve an objection by setting its disposition (R-036): the entry
/// itself is never removed — both resolved and open objections stay
/// visible on the record.
pub fn resolve_objection(
    record: &mut ReviewRecord,
    id: &str,
    note: &str,
    resolved_by: &str,
) -> Result<(), ReviewRejection> {
    let found = record
        .objections
        .iter_mut()
        .find(|o| o.id == id)
        .ok_or_else(|| ReviewRejection::ObjectionNotFound { id: id.to_string() })?;
    found.disposition = Disposition::Resolved {
        note: note.to_string(),
        resolved_by: resolved_by.to_string(),
    };
    Ok(())
}
