//! T-007 ticket 01 — typed capability grants (deny-first, seam: `hephaestus::policy::grant`).
//!
//! Obligations cited in tests: R-099/AT-099 (standing authority mints scoped
//! grants), R-060/AT-060 (bindings fixed at approval).

use hephaestus::contracts::generated::{
    AuthorizationGrantState, MissionDataOrigin, Money, Provenance, RecordRef, TrustOrigin,
};
use hephaestus::policy::{CapabilityGrant, GrantError, GrantSpec};

fn spec(id: &str) -> GrantSpec {
    GrantSpec {
        id: id.to_string(),
        mission_ref: RecordRef {
            id: "MIS-1".to_string(),
            version: 1,
        },
        operation_id: "OP-1".to_string(),
        capabilities: vec!["run_sandboxed".to_string()],
        destination: Some("local".to_string()),
        artifact_sha256: Some("a".repeat(64)),
        policy_version: "PV-1".to_string(),
        issued_at: "2026-09-30T00:00:00+00:00".to_string(),
        expires_at: "2026-10-01T00:00:00+00:00".to_string(),
        max_cost: Money {
            currency: "USD".to_string(),
            minor_units: 500,
        },
        issuer_id: "AUTH-LOCAL".to_string(),
        created_at: "2026-09-30T00:00:00+00:00".to_string(),
        data_origin: MissionDataOrigin::SyntheticFixture,
        provenance: Provenance {
            actor_id: "AUTH-LOCAL".to_string(),
            method: "standing_local_authority".to_string(),
            input_refs: vec![],
            artifact_hashes: vec![],
            trust_origin: TrustOrigin::Owner,
        },
        record_version: 1,
    }
}

#[test]
fn mint_binds_every_facet_and_is_approved() {
    // AT-099 / R-099: standing local authority mints a fully bound grant.
    let grant = CapabilityGrant::mint(spec("GRANT-1")).expect("mint");
    assert_eq!(grant.id, "GRANT-1");
    assert_eq!(grant.operation_id, "OP-1");
    assert_eq!(grant.capabilities, vec!["run_sandboxed"]);
    assert_eq!(grant.destination.as_deref(), Some("local"));
    assert_eq!(
        grant.artifact_sha256.as_deref(),
        Some("a".repeat(64).as_str())
    );
    assert_eq!(grant.policy_version, "PV-1");
    assert_eq!(grant.issued_at, "2026-09-30T00:00:00+00:00");
    assert_eq!(grant.expires_at, "2026-10-01T00:00:00+00:00");
    assert_eq!(grant.state, AuthorizationGrantState::Approved);
    assert_eq!(grant.max_cost.minor_units, 500);
    assert_eq!(grant.issuer_id, "AUTH-LOCAL");
}

#[test]
fn mint_rejects_empty_or_reversed_window() {
    // A grant that can never authorize is never constructible.
    let mut reversed = spec("GRANT-REV");
    reversed.issued_at = "2026-10-01T00:00:00+00:00".to_string();
    reversed.expires_at = "2026-09-30T00:00:00+00:00".to_string();
    assert_eq!(
        CapabilityGrant::mint(reversed),
        Err(GrantError::InvalidWindow)
    );
    let mut empty = spec("GRANT-EQ");
    empty.expires_at = empty.issued_at.clone();
    assert_eq!(CapabilityGrant::mint(empty), Err(GrantError::InvalidWindow));
}

#[test]
fn mint_rejects_contract_violations() {
    // Malformed binding fields are refused at mint, not discovered later.
    let mut bad_id = spec("bad id!");
    assert!(matches!(
        CapabilityGrant::mint(bad_id.clone()),
        Err(GrantError::Contract(_))
    ));
    bad_id.capabilities.clear();
    assert!(matches!(
        CapabilityGrant::mint(bad_id),
        Err(GrantError::Contract(_))
    ));
    let mut bad_cost = spec("GRANT-COST");
    bad_cost.max_cost.currency = "usd".to_string();
    assert!(matches!(
        CapabilityGrant::mint(bad_cost),
        Err(GrantError::Contract(_))
    ));
}

#[test]
fn revoke_transitions_state_and_surfaces_in_the_contract() {
    let mut grant = CapabilityGrant::mint(spec("GRANT-REVOKE")).expect("mint");
    assert_eq!(grant.state, AuthorizationGrantState::Approved);
    grant.revoke();
    assert_eq!(grant.state, AuthorizationGrantState::Revoked);
    // Revoked grants stay contract-conformant: revocation is a state, not
    // an invalid record.
    let contract = grant.to_contract();
    assert!(contract.validate().is_empty());
    assert_eq!(
        serde_json::to_value(&contract).expect("serialize")["state"],
        "revoked"
    );
}

#[test]
fn contract_round_trip_is_lossless() {
    // Typed <-> contract interop: the layers must not lose or invent facts.
    let minted = CapabilityGrant::mint(spec("GRANT-RT")).expect("mint");
    let contract = minted.to_contract();
    assert!(
        contract.validate().is_empty(),
        "contract form must validate"
    );
    let json = serde_json::to_value(&contract).expect("serialize");
    let parsed: hephaestus::contracts::generated::AuthorizationGrant =
        serde_json::from_value(json).expect("deserialize");
    let back = CapabilityGrant::from_contract(&parsed).expect("from contract");
    assert_eq!(back, minted);
}

#[test]
fn from_contract_also_rejects_invalid_grants() {
    // T-01 AC: validation runs at EVERY public constructor, not just mint.
    let minted = CapabilityGrant::mint(spec("GRANT-FC")).expect("mint");
    let mut contract = minted.to_contract();
    contract.issued_at = "2026-10-02T00:00:00+00:00".to_string(); // reversed window
    contract.expires_at = "2026-10-01T00:00:00+00:00".to_string();
    assert_eq!(
        CapabilityGrant::from_contract(&contract),
        Err(GrantError::InvalidWindow)
    );

    let mut contract = minted.to_contract();
    contract.operation_id = String::new(); // contract minLength violation
    assert!(matches!(
        CapabilityGrant::from_contract(&contract),
        Err(GrantError::Contract(_))
    ));
}
