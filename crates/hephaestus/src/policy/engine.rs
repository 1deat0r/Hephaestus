//! The deterministic policy engine (T-007).
//!
//! [`PolicyEngine::evaluate`] is a pure function: enumerated authorization
//! facts in, an allow/deny decision with stable per-facet reasons out. It
//! reads trust and its clock from the caller's [`TrustContext`] (R-095),
//! shares its facet predicates with the contract-layer check, and — by
//! construction — accepts no credential, provider, or model input (R-052):
//! nothing here can become an authoritative decision-maker except this code.

use serde::Serialize;

use crate::contracts::generated::Money;
use crate::security::grant::{
    DestinationBinding, artifact_bound, capabilities_subset, destination_allowed,
    destination_bound, interval_ordered, minor_cost_within, mission_state_approved,
    operation_bound, policy_chain_bound, refs_bound, window_valid,
};
use crate::security::trust::TrustContext;

use super::CapabilityGrant;

/// Why a decision denied — one stable code per facet, fixed vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReasonCode {
    /// Mission authorization is not `approved` (or missing).
    MissionAuthNotApproved,
    /// No grant was supplied for this operation.
    GrantMissing,
    /// The protected context does not trust this exact grant record.
    GrantNotTrusted,
    /// The grant binds a different mission or mission version.
    MissionRefMismatch,
    /// The grant has been revoked.
    GrantRevoked,
    /// Grant policy, mission policy, and context policy are not one chain.
    GrantPolicyMismatch,
    /// The approval's artifact identity does not match the request.
    GrantArtifactMismatch,
    /// The grant authorizes a different operation.
    GrantOperationMismatch,
    /// The grant's destination does not match the request's.
    GrantDestinationMismatch,
    /// The mission's allowed destinations do not include the destination.
    GrantDestinationNotAllowed,
    /// Required capabilities exceed the grant or the mission allow-list.
    GrantCapabilityNotGranted,
    /// The request carries no budget to check against the approved cost.
    RequestBudgetMissing,
    /// Requested cost exceeds the grant's approved cost cap (or currency differs).
    GrantCostExceeded,
    /// The grant's validity window is empty, reversed, or unparseable.
    GrantInvalidInterval,
    /// Outside the validity window, or the injected clock is unknown.
    GrantExpiredOrTimeUnknown,
}

/// What the operation wants: the enumerated request facts, nothing else.
/// Fixed shape by design — no credential field, no provider handle, no model
/// input can live here (R-052); a key-set test pins this.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PolicyRequest {
    /// Operation identity the grant must cover.
    pub operation_id: String,
    /// Capabilities the operation requires.
    pub required_capabilities: Vec<String>,
    /// Execution destination (None = unbound, as in the contract).
    pub destination: Option<String>,
    /// Approval/intervention artifact identity the operation presents.
    pub artifact_sha256: Option<String>,
    /// Cost the operation needs now (None = fail closed).
    pub budget: Option<Money>,
}

/// The mission's authorization facts as the engine sees them.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MissionState {
    /// Mission id.
    pub id: String,
    /// Mission record version (for `mission_ref` binding).
    pub version: i64,
    /// Authorization state: `Some("approved")` required.
    pub authorization_state: Option<String>,
    /// Mission policy version the grant must chain to.
    pub policy_version: Option<String>,
    /// Mission-wide capability allow-list (None = allow nothing).
    pub allowed_capabilities: Option<Vec<String>>,
    /// Mission-wide destination allow-list.
    pub allowed_destinations: Option<Vec<String>>,
}

/// The engine's answer: allow/deny plus the reasons, in fixed order.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PolicyDecision {
    /// True only when no reason denied.
    pub allowed: bool,
    /// Denial reasons in fixed declaration order; empty when allowed.
    pub reasons: Vec<ReasonCode>,
}

/// The deterministic policy engine. State-free: every fact is an argument.
#[derive(Debug, Clone, Copy, Default)]
pub struct PolicyEngine;

impl PolicyEngine {
    /// Evaluate one operation against its mission, optional grant, and the
    /// protected trust/clock context. Pure, fail-closed, deterministic:
    /// identical inputs produce identical decisions, including reason order.
    pub fn evaluate(
        request: &PolicyRequest,
        mission: &MissionState,
        grant: Option<&CapabilityGrant>,
        ctx: &TrustContext,
    ) -> PolicyDecision {
        let mut reasons: Vec<ReasonCode> = Vec::new();

        if !mission_state_approved(mission.authorization_state.as_deref()) {
            reasons.push(ReasonCode::MissionAuthNotApproved);
        }

        let Some(grant) = grant else {
            reasons.push(ReasonCode::GrantMissing);
            return PolicyDecision {
                allowed: false,
                reasons,
            };
        };

        // R-095: trust comes from the protected context, verified against the
        // grant's exact contract bytes — never from the caller's assertion.
        let grant_trusted = serde_json::to_value(grant.to_contract())
            .ok()
            .map(|value| ctx.trusted_grant(&grant.id, grant.record_version, &value))
            .unwrap_or(false);
        if !grant_trusted {
            reasons.push(ReasonCode::GrantNotTrusted);
        }

        if !refs_bound(
            Some((grant.mission_ref.id.as_str(), grant.mission_ref.version)),
            (mission.id.as_str(), mission.version),
        ) {
            reasons.push(ReasonCode::MissionRefMismatch);
        }
        if grant.state != crate::contracts::generated::AuthorizationGrantState::Approved {
            reasons.push(ReasonCode::GrantRevoked);
        }
        if !policy_chain_bound(
            Some(grant.policy_version.as_str()),
            mission.policy_version.as_deref(),
            ctx.current_policy(mission.id.as_str()),
        ) {
            reasons.push(ReasonCode::GrantPolicyMismatch);
        }
        if !artifact_bound(
            grant.artifact_sha256.as_deref(),
            request.artifact_sha256.as_deref(),
        ) {
            reasons.push(ReasonCode::GrantArtifactMismatch);
        }
        if !operation_bound(
            Some(grant.operation_id.as_str()),
            request.operation_id.as_str(),
        ) {
            reasons.push(ReasonCode::GrantOperationMismatch);
        }
        let grant_destination = match grant.destination.as_deref() {
            None => DestinationBinding::Unbound,
            Some(value) => DestinationBinding::Set(Some(value)),
        };
        if !destination_bound(grant_destination, request.destination.as_deref()) {
            reasons.push(ReasonCode::GrantDestinationMismatch);
        }
        if let Some(wanted) = request.destination.as_deref() {
            let allowed: Vec<&str> = mission
                .allowed_destinations
                .as_ref()
                .map(|list| list.iter().map(String::as_str).collect())
                .unwrap_or_default();
            if !destination_allowed(&allowed, wanted) {
                reasons.push(ReasonCode::GrantDestinationNotAllowed);
            }
        }

        let required: Vec<&str> = request
            .required_capabilities
            .iter()
            .map(String::as_str)
            .collect();
        let grant_caps: Vec<&str> = grant.capabilities.iter().map(String::as_str).collect();
        let mission_caps: Option<Vec<&str>> = mission
            .allowed_capabilities
            .as_ref()
            .map(|list| list.iter().map(String::as_str).collect());
        if !capabilities_subset(&required, Some(grant_caps.as_slice()))
            || !capabilities_subset(&required, mission_caps.as_deref())
        {
            reasons.push(ReasonCode::GrantCapabilityNotGranted);
        }

        match request.budget.as_ref() {
            None => reasons.push(ReasonCode::RequestBudgetMissing),
            Some(budget) => {
                let cap = Some((grant.max_cost.currency.as_str(), grant.max_cost.minor_units));
                let used = Some((budget.currency.as_str(), budget.minor_units));
                if !minor_cost_within(cap, used) {
                    reasons.push(ReasonCode::GrantCostExceeded);
                }
            }
        }

        if !interval_ordered(
            Some(grant.issued_at.as_str()),
            Some(grant.expires_at.as_str()),
        ) {
            reasons.push(ReasonCode::GrantInvalidInterval);
        } else if !window_valid(
            Some(grant.issued_at.as_str()),
            Some(grant.expires_at.as_str()),
            ctx.evaluated_at(),
        ) {
            reasons.push(ReasonCode::GrantExpiredOrTimeUnknown);
        }

        PolicyDecision {
            allowed: reasons.is_empty(),
            reasons,
        }
    }
}
