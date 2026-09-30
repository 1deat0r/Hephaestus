//! External trust context (T-004, ADR-013).
//!
//! Mirrors `reference.qualification.ValidationContext`: trust facts come
//! from a protected verifier — authenticated grants, receipts, verified
//! artifact bytes, current versions/hashes/policies, an external clock.
//! This type deliberately implements no `Deserialize` and exposes no
//! constructor other than [`TrustContext::empty`] plus explicit builder
//! calls: a candidate record or model output can never carry its own trust
//! into a check. Default trust is empty; missing facts fail closed.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use time::OffsetDateTime;

use super::digest::record_digest;

/// Immutable `(id, record_version)` identity of a record.
pub type RecordKey = (String, i64);

/// Externally supplied trust facts. Construct only via `empty()` and the
/// `trust_*`/clock setters — never parse this from a bundle.
#[derive(Debug, Clone, Default)]
pub struct TrustContext {
    trusted_receipts: BTreeSet<RecordKey>,
    trusted_qualifications: BTreeSet<RecordKey>,
    trusted_grants: BTreeSet<RecordKey>,
    trusted_holdout_accesses: BTreeSet<RecordKey>,
    trusted_families: BTreeSet<RecordKey>,
    trusted_improvements: BTreeSet<RecordKey>,
    champion_by_scope: BTreeMap<(String, String), String>,
    authenticated_record_hashes: BTreeMap<RecordKey, String>,
    verified_artifacts: BTreeSet<String>,
    current_versions: BTreeMap<String, i64>,
    current_record_hashes: BTreeMap<RecordKey, String>,
    current_policies: BTreeMap<String, String>,
    evaluated_at: Option<OffsetDateTime>,
    synthetic_fixture_mode: bool,
}

impl TrustContext {
    /// A context with zero trust — everything denies until explicitly fed.
    pub fn empty() -> Self {
        Self::default()
    }

    fn key(id: &str, version: i64) -> RecordKey {
        (id.to_string(), version)
    }

    pub fn trust_receipt(&mut self, id: &str, version: i64) {
        self.trusted_receipts.insert(Self::key(id, version));
    }

    pub fn trust_qualification(&mut self, id: &str, version: i64) {
        self.trusted_qualifications.insert(Self::key(id, version));
    }

    pub fn trust_grant(&mut self, id: &str, version: i64) {
        self.trusted_grants.insert(Self::key(id, version));
    }

    pub fn trust_holdout_access(&mut self, id: &str, version: i64) {
        self.trusted_holdout_accesses.insert(Self::key(id, version));
    }

    pub fn trust_family(&mut self, id: &str, version: i64) {
        self.trusted_families.insert(Self::key(id, version));
    }

    pub fn trust_improvement(&mut self, id: &str, version: i64) {
        self.trusted_improvements.insert(Self::key(id, version));
    }

    /// Record the externally authenticated hash for a trusted identity.
    pub fn authenticate_record_hash(&mut self, id: &str, version: i64, sha256: String) {
        self.authenticated_record_hashes
            .insert(Self::key(id, version), sha256);
    }

    /// Mark captured artifact bytes as verified by the protected verifier.
    pub fn verify_artifact(&mut self, sha256: String) {
        self.verified_artifacts.insert(sha256);
    }

    pub fn set_current_version(&mut self, id: &str, version: i64) {
        self.current_versions.insert(id.to_string(), version);
    }

    pub fn set_current_record_hash(&mut self, id: &str, version: i64, sha256: String) {
        self.current_record_hashes
            .insert(Self::key(id, version), sha256);
    }

    pub fn set_current_policy(&mut self, mission_id: &str, policy_version: String) {
        self.current_policies
            .insert(mission_id.to_string(), policy_version);
    }

    pub fn set_champion(&mut self, mission_id: &str, target: &str, candidate_sha256: String) {
        self.champion_by_scope.insert(
            (mission_id.to_string(), target.to_string()),
            candidate_sha256,
        );
    }

    /// External clock fact (ADR-013): evaluations without a clock deny
    /// time-bounded claims instead of guessing.
    pub fn set_evaluated_at(&mut self, at: OffsetDateTime) {
        self.evaluated_at = Some(at);
    }

    /// Test-fixture escape hatch; never set in production trust flows.
    pub fn set_synthetic_fixture_mode(&mut self, on: bool) {
        self.synthetic_fixture_mode = on;
    }

    pub fn evaluated_at(&self) -> Option<OffsetDateTime> {
        self.evaluated_at
    }

    pub fn synthetic_fixture_mode(&self) -> bool {
        self.synthetic_fixture_mode
    }

    pub fn current_policy(&self, mission_id: &str) -> Option<&str> {
        self.current_policies.get(mission_id).map(String::as_str)
    }

    pub fn current_version(&self, id: &str) -> Option<i64> {
        self.current_versions.get(id).copied()
    }

    pub fn has_verified_artifact(&self, sha256: &str) -> bool {
        self.verified_artifacts.contains(sha256)
    }

    pub fn champion(&self, mission_id: &str, target: &str) -> Option<&str> {
        self.champion_by_scope
            .get(&(mission_id.to_string(), target.to_string()))
            .map(String::as_str)
    }

    fn trusted(
        allowed: &BTreeSet<RecordKey>,
        authenticated: &BTreeMap<RecordKey, String>,
        id: &str,
        version: i64,
        record: &Value,
    ) -> bool {
        let key = Self::key(id, version);
        if !allowed.contains(&key) {
            return false;
        }
        match record_digest(record) {
            Ok(digest) => authenticated.get(&key) == Some(&digest),
            Err(_) => false,
        }
    }

    /// Trust requires BOTH membership in the right allow-set AND an
    /// externally authenticated hash matching the record's current bytes.
    /// Changed content after authentication fails closed (R-098).
    pub fn trusted_receipt(&self, id: &str, version: i64, record: &Value) -> bool {
        Self::trusted(
            &self.trusted_receipts,
            &self.authenticated_record_hashes,
            id,
            version,
            record,
        )
    }

    pub fn trusted_grant(&self, id: &str, version: i64, record: &Value) -> bool {
        Self::trusted(
            &self.trusted_grants,
            &self.authenticated_record_hashes,
            id,
            version,
            record,
        )
    }

    pub fn trusted_qualification(&self, id: &str, version: i64, record: &Value) -> bool {
        Self::trusted(
            &self.trusted_qualifications,
            &self.authenticated_record_hashes,
            id,
            version,
            record,
        )
    }

    pub fn trusted_family(&self, id: &str, version: i64, record: &Value) -> bool {
        Self::trusted(
            &self.trusted_families,
            &self.authenticated_record_hashes,
            id,
            version,
            record,
        )
    }

    pub fn trusted_holdout_access(&self, id: &str, version: i64, record: &Value) -> bool {
        Self::trusted(
            &self.trusted_holdout_accesses,
            &self.authenticated_record_hashes,
            id,
            version,
            record,
        )
    }

    pub fn trusted_improvement(&self, id: &str, version: i64, record: &Value) -> bool {
        Self::trusted(
            &self.trusted_improvements,
            &self.authenticated_record_hashes,
            id,
            version,
            record,
        )
    }

    /// `None` when the record's current bytes differ from (or lack) an
    /// external hash — callers treat that as stale/unknown (R-098).
    pub fn record_hash_matches(&self, id: &str, version: i64, record: &Value) -> Option<bool> {
        let expected = self
            .authenticated_record_hashes
            .get(&Self::key(id, version))
            .or_else(|| self.current_record_hashes.get(&Self::key(id, version)))?;
        Some(record_digest(record).ok().as_ref() == Some(expected))
    }
}
