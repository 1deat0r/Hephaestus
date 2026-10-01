//! Domain contract and adapter verifier gates (T-046,
//! R-061/R-062/R-063).
//!
//! A THIN named enforcement surface over proven machinery:
//! `semantic::validate_bundle` stays the validator of record (R-061) —
//! this module maps its findings onto stable typed rejection codes;
//! the provider-leak scan guards core/adapter boundaries (R-062); and
//! version/reference checks fail closed with stable reason codes
//! (R-063). Pure functions, no I/O.

use crate::semantic;
use serde::{Deserialize, Serialize};

/// A stable, machine-comparable rejection. Codes never change meaning
/// across versions; `detail` carries the specifics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedRejection {
    pub code: RejectionCode,
    pub detail: String,
}

/// Stable rejection codes (R-063: typed failure categories).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RejectionCode {
    /// Schema-valid but semantically contradictory records (R-061).
    SemanticContradiction,
    /// A provider SDK type leaked into a core symbol (R-062).
    ProviderLeak,
    /// Unknown schema version (R-063).
    UnknownSchemaVersion,
    /// A reference that does not resolve (R-063).
    MissingReference,
    /// A capability the adapter does not declare (R-062 adapter swap).
    UnsupportedCapability,
}

impl TypedRejection {
    pub fn code_name(&self) -> &'static str {
        match self.code {
            RejectionCode::SemanticContradiction => "SEMANTIC_CONTRADICTION",
            RejectionCode::ProviderLeak => "PROVIDER_LEAK",
            RejectionCode::UnknownSchemaVersion => "UNKNOWN_SCHEMA_VERSION",
            RejectionCode::MissingReference => "MISSING_REFERENCE",
            RejectionCode::UnsupportedCapability => "UNSUPPORTED_CAPABILITY",
        }
    }
}

/// Schema versions this verifier knows. Anything else is rejected with
/// `UNKNOWN_SCHEMA_VERSION` (fail-closed, R-063).
pub const KNOWN_SCHEMA_VERSIONS: &[&str] = &["1.0", "1.1", "1.2", "1.3"];

/// R-061/AT-061: run the semantic validator of record and map its
/// findings onto stable codes. Missing references inside the bundle
/// surface as `MISSING_REFERENCE`; everything else contradictory
/// (duplicates, kind mismatches, guardrail violations) as
/// `SEMANTIC_CONTRADICTION`.
pub fn verify_cross_record(bundle: &serde_json::Value) -> Result<(), Vec<TypedRejection>> {
    let findings = semantic::validate_bundle(bundle);
    if findings.is_empty() {
        return Ok(());
    }
    let rejections = findings
        .into_iter()
        .map(|f| {
            if f.starts_with("MISSING_REFERENCE") {
                TypedRejection {
                    code: RejectionCode::MissingReference,
                    detail: f,
                }
            } else {
                TypedRejection {
                    code: RejectionCode::SemanticContradiction,
                    detail: f,
                }
            }
        })
        .collect();
    Err(rejections)
}

/// Provider SDK markers that must never appear as core symbol names
/// (R-062: provider-specific types stay outside core interfaces).
pub const PROVIDER_MARKERS: &[&str] = &[
    "openai",
    "anthropic",
    "openrouter",
    "reqwest",
    "hyper::client",
    "sdk_",
];

/// R-062/AT-062: fail with `PROVIDER_LEAK` when a core symbol name
/// embeds a provider SDK marker.
pub fn verify_no_provider_leak(core_symbols: &[String]) -> Result<(), Vec<TypedRejection>> {
    let mut leaks = Vec::new();
    for sym in core_symbols {
        let lower = sym.to_lowercase();
        if let Some(marker) = PROVIDER_MARKERS.iter().find(|m| lower.contains(*m)) {
            leaks.push(TypedRejection {
                code: RejectionCode::ProviderLeak,
                detail: format!("core symbol '{sym}' embeds provider marker '{marker}'"),
            });
        }
    }
    if leaks.is_empty() { Ok(()) } else { Err(leaks) }
}

/// R-062 adapter-swap surface: an adapter declares the capabilities it
/// supports; requesting anything else fails explicitly (never a silent
/// fallback).
pub fn verify_supported_capability(
    declared: &[String],
    requested: &str,
) -> Result<(), TypedRejection> {
    if declared.iter().any(|c| c == requested) {
        Ok(())
    } else {
        Err(TypedRejection {
            code: RejectionCode::UnsupportedCapability,
            detail: format!(
                "capability '{requested}' not declared by this adapter (declared: {})",
                declared.join(", ")
            ),
        })
    }
}

/// R-063/AT-063: unknown schema version or unresolvable reference ->
/// stable codes; everything resolves -> Ok.
pub fn verify_version_and_refs(
    schema_version: &str,
    known_refs: &[String],
    requested_refs: &[String],
) -> Result<(), Vec<TypedRejection>> {
    let mut rejections = Vec::new();
    if !KNOWN_SCHEMA_VERSIONS.contains(&schema_version) {
        rejections.push(TypedRejection {
            code: RejectionCode::UnknownSchemaVersion,
            detail: format!("schema version '{schema_version}' is not registered"),
        });
    }
    for r in requested_refs {
        if !known_refs.contains(r) {
            rejections.push(TypedRejection {
                code: RejectionCode::MissingReference,
                detail: format!("reference '{r}' does not resolve"),
            });
        }
    }
    if rejections.is_empty() {
        Ok(())
    } else {
        Err(rejections)
    }
}
