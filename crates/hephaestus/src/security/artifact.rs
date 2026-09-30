//! Artifact identity (T-004).
//!
//! An [`ArtifactDigest`] proves *which bytes* an artifact is (identity and
//! integrity over captured bytes). It is deliberately a bare identity type:
//! possessing or producing a digest never authorizes anything. Authorization
//! requires a keyed approval (see [`super::approval`]) plus an externally
//! supplied trust context (see [`super::trust`]) — never a plain content
//! hash, per IMPLEMENTATION_PLAN T-004.

use std::fmt;

/// A SHA-256 artifact digest: exactly 64 lowercase hex characters, matching
/// the contract schema's `^[a-f0-9]{64}$` profile.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArtifactDigest(String);

/// Why a value is not a valid artifact digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidDigest(pub String);

impl fmt::Display for InvalidDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid artifact digest: {}", self.0)
    }
}

impl std::error::Error for InvalidDigest {}

impl ArtifactDigest {
    /// Parse a digest from its hex spelling (strict: lowercase, length 64).
    pub fn parse(value: &str) -> Result<Self, InvalidDigest> {
        let ok = value.len() == 64
            && value
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
        if ok {
            Ok(Self(value.to_string()))
        } else {
            Err(InvalidDigest(value.to_string()))
        }
    }

    /// Digest of actually captured bytes. The caller must have verified the
    /// bytes came from the declared source; a digest of model-supplied
    /// "claimed bytes" proves nothing.
    pub fn of_bytes(bytes: &[u8]) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let hex: String = hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        Self(hex)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ArtifactDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
