//! Manifest approval format (T-004).
//!
//! Approvals are keyed HMAC-SHA256 authentications over canonical manifest
//! bytes — deterministic for a given key and payload. A plain content hash
//! is never an approval: verification requires the secret key, which is
//! supplied from outside the system (environment/HSM), never from a
//! candidate record or model output (IMPLEMENTATION_PLAN T-004, ADR-010).

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use super::digest::canonical_bytes;

type HmacSha256 = Hmac<Sha256>;

/// Secret key for manifest approvals. Constructed only from externally
/// provisioned bytes; this type intentionally has no deserialization.
#[derive(Clone)]
pub struct ApprovalKey(Vec<u8>);

/// Why an approval operation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalError {
    /// Key material was too short or empty.
    WeakKey(usize),
    /// Tag was not 64 lowercase hex characters.
    MalformedTag(String),
    /// HMAC verification failed: wrong key, tampered payload, or a value
    /// that was never an approval (e.g. a bare content hash).
    VerificationFailed,
    /// Payload could not be canonicalized (non-finite number, bad key).
    CanonicalizationFailed,
}

impl std::fmt::Display for ApprovalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalError::WeakKey(n) => {
                write!(f, "approval key too short: {n} bytes (need >= 32)")
            }
            ApprovalError::MalformedTag(t) => {
                write!(f, "malformed approval tag: {t}")
            }
            ApprovalError::VerificationFailed => write!(f, "approval verification failed"),
            ApprovalError::CanonicalizationFailed => {
                write!(f, "manifest is not canonicalizable")
            }
        }
    }
}

impl std::error::Error for ApprovalError {}

impl ApprovalKey {
    /// Provision key material from outside the record system. Minimum 32
    /// bytes; empty or short material fails closed.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ApprovalError> {
        if bytes.len() < 32 {
            return Err(ApprovalError::WeakKey(bytes.len()));
        }
        Ok(Self(bytes.to_vec()))
    }
}

fn tag_hex(key: &ApprovalKey, payload: &[u8]) -> Result<String, ApprovalError> {
    let mut mac =
        HmacSha256::new_from_slice(&key.0).map_err(|_| ApprovalError::WeakKey(key.0.len()))?;
    mac.update(payload);
    Ok(mac
        .finalize()
        .into_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// Deterministic approval over raw payload bytes.
pub fn approve(key: &ApprovalKey, payload: &[u8]) -> Result<String, ApprovalError> {
    tag_hex(key, payload)
}

/// Deterministic approval over a manifest's canonical JSON encoding.
pub fn approve_manifest(
    key: &ApprovalKey,
    manifest: &serde_json::Value,
) -> Result<String, ApprovalError> {
    let bytes = canonical_bytes(manifest).map_err(|_| ApprovalError::CanonicalizationFailed)?;
    approve(key, &bytes)
}

/// Verify an approval tag against raw payload bytes. Uses the MAC's
/// constant-time verification; any malformed tag fails closed.
pub fn verify(key: &ApprovalKey, payload: &[u8], tag: &str) -> Result<(), ApprovalError> {
    if tag.len() != 64 || !tag.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        return Err(ApprovalError::MalformedTag(tag.to_string()));
    }
    let tag_bytes =
        hex_to_bytes(tag).ok_or_else(|| ApprovalError::MalformedTag(tag.to_string()))?;
    let mut mac =
        HmacSha256::new_from_slice(&key.0).map_err(|_| ApprovalError::WeakKey(key.0.len()))?;
    mac.update(payload);
    mac.verify_slice(&tag_bytes)
        .map_err(|_| ApprovalError::VerificationFailed)
}

/// Verify an approval against a manifest's canonical JSON encoding.
pub fn verify_manifest(
    key: &ApprovalKey,
    manifest: &serde_json::Value,
    tag: &str,
) -> Result<(), ApprovalError> {
    let bytes = canonical_bytes(manifest).map_err(|_| ApprovalError::CanonicalizationFailed)?;
    verify(key, &bytes, tag)
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}
