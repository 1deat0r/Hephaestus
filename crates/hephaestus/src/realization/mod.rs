//! External-act authorization gate (T-048, MASTER_SPEC §15,
//! R-045/AT-045).
//!
//! Adoption, publication, and manufacturing of a finished candidate
//! each require SEPARATE authorization. A positive prototype result is
//! evidence, never authorization (AT-045 negative: a finished positive
//! prototype cannot request automatic public deployment on its own
//! strength). The module records authorizations handed to it by the
//! standing authority and enforces act-separateness and subject-digest
//! binding; it grants nothing itself.

use serde::{Deserialize, Serialize};

/// The three external actions that each need their own authorization
/// (R-045).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalAct {
    Adoption,
    Publication,
    Manufacturing,
}

/// Why the caller says the action should proceed. The ONLY modeled
/// justification is a positive prototype result — and it is never
/// sufficient on its own (R-045).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Justification {
    PositivePrototypeResult { result_digest: String },
}

/// A per-act authorization recorded from the standing authority
/// (R-045). One record covers exactly one act for one subject digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActAuthorization {
    pub act: ExternalAct,
    pub subject: String,
    pub subject_digest: String,
    /// The standing authority that granted it — never a model output.
    pub granted_by: String,
    /// Version-bound authority receipt (composes with R-060 approvals).
    pub authority_receipt: String,
}

/// Why an external-act request was refused (typed-rejection
/// convention).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActError {
    /// The supplied positive result does not authorize this act.
    ResultNeverAuthorizes {
        act: ExternalAct,
        justification_digest: String,
    },
    /// Authorizations exist for this subject, but not for this act.
    MissingSeparateAuthorization { act: ExternalAct },
    /// The authorization is for a different subject digest.
    AuthorizationMismatch { act: ExternalAct },
}

/// Version-bound runtime evidence that an act was authorized: the act
/// and the authorization receipt it rode on (R-045 positive case).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActReceipt {
    pub act: ExternalAct,
    pub authorization_digest: String,
}

/// The gate: recorded per-act authorizations plus the request seam.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RealizationGate {
    authorizations: Vec<ActAuthorization>,
}

/// Record an authorization handed over by the standing authority (the
/// module itself grants nothing).
pub fn record_authorization(gate: &mut RealizationGate, authorization: ActAuthorization) {
    gate.authorizations.push(authorization);
}

impl RealizationGate {
    pub fn new() -> Self {
        Self::default()
    }

    /// Request an external action (R-045). A positive result alone is
    /// refused by name; authorizations are per-act and pinned to the
    /// subject digest.
    pub fn request(
        &self,
        act: ExternalAct,
        subject: &str,
        subject_digest: &str,
        justification: &Justification,
    ) -> Result<ActReceipt, ActError> {
        if let Some(auth) = self
            .authorizations
            .iter()
            .find(|a| a.act == act && a.subject == subject)
        {
            if auth.subject_digest != subject_digest {
                return Err(ActError::AuthorizationMismatch { act });
            }
            return Ok(ActReceipt {
                act,
                authorization_digest: auth.authority_receipt.clone(),
            });
        }
        // No authorization for this act on this subject.
        let subject_has_any = self.authorizations.iter().any(|a| a.subject == subject);
        if subject_has_any {
            // Other acts are authorized — but never THIS one.
            Err(ActError::MissingSeparateAuthorization { act })
        } else {
            match justification {
                Justification::PositivePrototypeResult { result_digest } => {
                    Err(ActError::ResultNeverAuthorizes {
                        act,
                        justification_digest: result_digest.clone(),
                    })
                }
            }
        }
    }
}
