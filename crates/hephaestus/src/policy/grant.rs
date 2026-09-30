//! Typed capability grants (T-007).
//!
//! A grant binds one operation, its capability scope, destination, artifact
//! identity, policy version, validity window, and approved cost.
//! [`CapabilityGrant::mint`] is
//! the standing local authority (AT-099): it validates inputs against the
//! `authorization_grant` contract and a non-empty window, and yields an
//! approved grant. Grants convert losslessly to and from the contract so the
//! typed and JSON layers interoperate.

use time::OffsetDateTime;

use crate::contracts::generated::{
    AuthorizationGrant, AuthorizationGrantKind, AuthorizationGrantState, MissionDataOrigin, Money,
    Provenance, RecordRef, SchemaVersion,
};

/// Everything `mint` needs to build an approved grant.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantSpec {
    /// Contract id.
    pub id: String,
    /// Mission the grant must bind to.
    pub mission_ref: RecordRef,
    /// Operation identity the grant covers.
    pub operation_id: String,
    /// Capability scope the grant allows.
    pub capabilities: Vec<String>,
    /// Execution destination (None = the contract's null destination).
    pub destination: Option<String>,
    /// Approval/intervention artifact identity, if bound.
    pub artifact_sha256: Option<String>,
    /// Policy version the grant must chain to.
    pub policy_version: String,
    /// RFC 3339 start of the validity window (inclusive).
    pub issued_at: String,
    /// RFC 3339 end of the validity window (exclusive).
    pub expires_at: String,
    /// Approved cost ceiling.
    pub max_cost: Money,
    /// Standing local authority that issued the grant.
    pub issuer_id: String,
    /// Contract envelope: creation time (RFC 3339).
    pub created_at: String,
    /// Contract envelope: data origin.
    pub data_origin: MissionDataOrigin,
    /// Contract envelope: provenance.
    pub provenance: Provenance,
    /// Contract envelope: record version.
    pub record_version: i64,
}

/// A validated, contract-conformant capability grant.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityGrant {
    /// Contract id.
    pub id: String,
    /// Contract envelope: record version.
    pub record_version: i64,
    /// Contract envelope: creation time (RFC 3339).
    pub created_at: String,
    /// Contract envelope: data origin.
    pub data_origin: MissionDataOrigin,
    /// Contract envelope: provenance.
    pub provenance: Provenance,
    /// Mission the grant binds to.
    pub mission_ref: RecordRef,
    /// Operation identity the grant covers.
    pub operation_id: String,
    /// Capability scope the grant allows.
    pub capabilities: Vec<String>,
    /// Execution destination (None = contract null).
    pub destination: Option<String>,
    /// Approval/intervention artifact identity, if bound.
    pub artifact_sha256: Option<String>,
    /// Policy version the grant chains to.
    pub policy_version: String,
    /// RFC 3339 window start (inclusive).
    pub issued_at: String,
    /// RFC 3339 window end (exclusive).
    pub expires_at: String,
    /// Approved or revoked.
    pub state: AuthorizationGrantState,
    /// Approved cost ceiling.
    pub max_cost: Money,
    /// Issuing standing local authority.
    pub issuer_id: String,
}

/// Why a grant was refused.
#[derive(Debug, PartialEq)]
pub enum GrantError {
    /// The validity window is empty, reversed, or unparseable.
    InvalidWindow,
    /// The grant violates the `authorization_grant` contract.
    Contract(Vec<crate::contracts::ContractViolation>),
}

impl std::fmt::Display for GrantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrantError::InvalidWindow => write!(f, "grant validity window is empty or reversed"),
            GrantError::Contract(violations) => {
                write!(
                    f,
                    "grant violates contract ({} violations): ",
                    violations.len()
                )?;
                for (i, v) in violations.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{}: {}", v.path, v.message)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for GrantError {}

impl CapabilityGrant {
    /// The standing local authority: validate `spec` and mint an approved
    /// grant. Never returns success for a grant that violates the
    /// `authorization_grant` contract or whose validity window is empty —
    /// whether it will *authorize* still depends on trust, policy, and clock
    /// facts the engine checks at evaluation time.
    pub fn mint(spec: GrantSpec) -> Result<Self, GrantError> {
        let grant = CapabilityGrant {
            id: spec.id,
            record_version: spec.record_version,
            created_at: spec.created_at,
            data_origin: spec.data_origin,
            provenance: spec.provenance,
            mission_ref: spec.mission_ref,
            operation_id: spec.operation_id,
            capabilities: spec.capabilities,
            destination: spec.destination,
            artifact_sha256: spec.artifact_sha256,
            policy_version: spec.policy_version,
            issued_at: spec.issued_at,
            expires_at: spec.expires_at,
            state: AuthorizationGrantState::Approved,
            max_cost: spec.max_cost,
            issuer_id: spec.issuer_id,
        };
        grant.validate()?;
        Ok(grant)
    }

    /// Re-check contract conformance and window order on the current values.
    pub fn validate(&self) -> Result<(), GrantError> {
        let violations = self.to_contract().validate();
        if !violations.is_empty() {
            return Err(GrantError::Contract(violations));
        }
        let issued = OffsetDateTime::parse(
            &self.issued_at,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|_| GrantError::InvalidWindow)?;
        let expires = OffsetDateTime::parse(
            &self.expires_at,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|_| GrantError::InvalidWindow)?;
        if issued >= expires {
            return Err(GrantError::InvalidWindow);
        }
        Ok(())
    }

    /// Revoke the grant (idempotent; the state is what the engine checks).
    pub fn revoke(&mut self) {
        self.state = AuthorizationGrantState::Revoked;
    }

    /// Convert to the `authorization_grant` contract record.
    pub fn to_contract(&self) -> AuthorizationGrant {
        AuthorizationGrant {
            id: self.id.clone(),
            schema_version: SchemaVersion::V1_2,
            record_version: self.record_version,
            created_at: self.created_at.clone(),
            data_origin: self.data_origin,
            provenance: self.provenance.clone(),
            kind: AuthorizationGrantKind::AuthorizationGrant,
            mission_ref: self.mission_ref.clone(),
            operation_id: self.operation_id.clone(),
            capabilities: self.capabilities.clone(),
            destination: self.destination.clone(),
            artifact_sha256: self.artifact_sha256.clone(),
            policy_version: self.policy_version.clone(),
            issued_at: self.issued_at.clone(),
            expires_at: self.expires_at.clone(),
            state: self.state,
            max_cost: self.max_cost.clone(),
            issuer_id: self.issuer_id.clone(),
        }
    }

    /// Rebuild from the contract record; re-validates like `mint`.
    pub fn from_contract(grant: &AuthorizationGrant) -> Result<Self, GrantError> {
        let typed = CapabilityGrant {
            id: grant.id.clone(),
            record_version: grant.record_version,
            created_at: grant.created_at.clone(),
            data_origin: grant.data_origin,
            provenance: grant.provenance.clone(),
            mission_ref: grant.mission_ref.clone(),
            operation_id: grant.operation_id.clone(),
            capabilities: grant.capabilities.clone(),
            destination: grant.destination.clone(),
            artifact_sha256: grant.artifact_sha256.clone(),
            policy_version: grant.policy_version.clone(),
            issued_at: grant.issued_at.clone(),
            expires_at: grant.expires_at.clone(),
            state: grant.state,
            max_cost: grant.max_cost.clone(),
            issuer_id: grant.issuer_id.clone(),
        };
        typed.validate()?;
        Ok(typed)
    }
}
