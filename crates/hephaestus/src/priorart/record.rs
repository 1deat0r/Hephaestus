//! Prior-art records (T-018): claims, conclusions, charts, reports.

use serde::{Deserialize, Serialize};

/// The closed set of allowed prior-art conclusions (MASTER_SPEC
/// §10:203). None of these means guaranteed global novelty; no score
/// field exists anywhere on the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Conclusion {
    /// Clear rediscovery found in stage 1.
    Known,
    /// Claim-level overlap with assessable differences.
    NearMatch,
    /// Nothing found inside the searched scope — explicitly scoped, never
    /// a global-novelty claim.
    NoMatchWithinSearchScope,
    /// Prior art contradicts the claim.
    Conflicting,
    /// Search could not decide (missing access, ambiguous evidence).
    Unresolved,
}

/// One decomposable part of a candidate (§10:200): component, mechanism,
/// use context, operating regime, claimed result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtomicClaim {
    pub id: String,
    pub component: String,
    pub mechanism: String,
    pub use_context: String,
    pub regime: String,
    pub claimed_result: String,
}

impl AtomicClaim {
    /// The mechanism-specific tokens used for redaction (§10:205).
    pub fn mechanism_tokens(&self) -> Vec<&str> {
        self.mechanism
            .split_whitespace()
            .filter(|t| t.len() > 3)
            .collect()
    }
}

/// A passage reference verified against captured bytes (R-107): source
/// name + byte range + the captured text itself. A mutable locator alone
/// is never sufficient.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassageRef {
    pub source: String,
    pub start: usize,
    pub end: usize,
    pub captured_text: String,
}

/// One claim-chart row: the claim's value vs the prior-art value, with a
/// verified passage reference and a similarity/difference account
/// (§10:206, R-112).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartRow {
    pub field: String, // component|mechanism|use_context|regime|claimed_result
    pub candidate_value: String,
    pub prior_art_value: String,
    pub passage: PassageRef,
    pub similar: bool,
    pub account: String,
}

/// A verified claim chart for one claim against one prior-art item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimChart {
    pub claim_id: String,
    pub rows: Vec<ChartRow>,
    pub differences_assessed: bool,
}

/// Disclosure gate (§10:205): redacted search is the default path;
/// unredacted external submission requires explicit approval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DisclosureGate {
    pub full_disclosure_approved: bool,
}

/// The search plan: scopes covered, query families, redaction state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchPlan {
    pub scopes: Vec<String>,
    pub query_families: Vec<String>,
    pub gate: DisclosureGate,
}

/// Report metadata (R-029): dates, scopes, query families, near-matches,
/// missing access.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportMeta {
    pub search_date: String,
    pub scopes_covered: Vec<String>,
    pub query_families: Vec<String>,
    pub near_matches: Vec<String>,
    pub missing_access: Vec<String>,
}

/// The full report: per-claim conclusions, charts, metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriorArtReport {
    pub conclusions: Vec<(String, Conclusion)>,
    pub charts: Vec<ClaimChart>,
    pub meta: ReportMeta,
}

/// Rediscovery label (R-112): a validated solution displays its
/// rediscovery status; a validated candidate additionally requires a
/// verified chart with assessed differences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RediscoveryLabel {
    ValidatedSolution,
    ValidatedCandidate,
}
