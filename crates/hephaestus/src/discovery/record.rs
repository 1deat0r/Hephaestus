//! Knowledge records for discovery (T-014): trace records, opportunities,
//! rejected candidates.
//!
//! A `TraceRecord` is structured authorized trace data whose evidence binds
//! to corpus span coordinates. Operators emit `Opportunity` records (all
//! MASTER_SPEC §7:150 fields, explicit unknowns) or `RejectedCandidate`
//! records with reasons — nothing is silently dropped.

use crate::knowledge::Span;
use serde::{Deserialize, Serialize};

/// Numeric or duration value carried by a trace record. Units are part of
/// the value: anomaly analysis refuses to compare across units (R-020).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TraceVal {
    DurationMs(u64),
    Count(u64),
    Ratio(f64),
}

impl TraceVal {
    /// Unit identity used for comparability gating.
    pub fn unit(&self) -> &'static str {
        match self {
            Self::DurationMs(_) => "ms",
            Self::Count(_) => "count",
            Self::Ratio(_) => "ratio",
        }
    }

    /// Magnitude as f64 (lossy only for display; comparisons are exact
    /// for the integer variants).
    pub fn magnitude(&self) -> f64 {
        match self {
            Self::DurationMs(v) => *v as f64,
            Self::Count(v) => *v as f64,
            Self::Ratio(v) => *v,
        }
    }
}

/// Structured authorized trace record. `evidence` binds the record to a
/// span in the corpus; records without evidence can never ground an
/// emitted opportunity (no fabricated citations).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceRecord {
    pub id: String,
    /// Record kind: `duration`, `counter`, `metric`, `event`,
    /// `assumption`, `capability`, `objective`, `coupling`.
    pub kind: String,
    pub name: String,
    pub value: TraceVal,
    pub evidence: Option<Span>,
}

/// Pressure-point type (IMPLEMENTATION_PLAN:52).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PressureKind {
    Bottleneck,
    ConflictingObjectives,
    FailurePattern,
    Anomaly,
    Assumption,
    ChangedCapability,
}

/// Grounding verdict, reported independently of narrative polish (R-021).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Validity {
    /// Span-bound evidence attached and verified against the corpus.
    EvidenceBacked,
    /// An alternative-explanation challenge was recorded against the
    /// candidate and stands unrefuted.
    Challenged,
    /// Conjecture only: no grounding evidence (MASTER_SPEC:150 — may
    /// exist but never promotes on polish).
    Speculative,
}

/// Value estimate: qualitative bucket with justification — no invented
/// numbers (spec: fabricated estimates are out of scope).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueEstimate {
    Unclear,
    Modest,
    Significant,
}

/// The operator output (MASTER_SPEC §7:150): every field present, unknowns
/// explicit. `validity` is orthogonal to `narrative_polish` — a complete,
/// well-written narrative never lifts validity (R-021/AT-021).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Opportunity {
    pub kind: PressureKind,
    pub problem_statement: String,
    pub beneficiary: String,
    pub context: String,
    /// Span-bound evidence references; empty only when the opportunity is
    /// explicitly speculative (never for emitted grounded ones).
    pub evidence_references: Vec<Span>,
    pub suspected_bottleneck: Option<String>,
    /// Explicit causal uncertainty: stated as text with the uncertainty
    /// named — never a fabricated numeric confidence.
    pub causal_uncertainty: String,
    pub value_estimate: ValueEstimate,
    /// Feasibility notes: what makes the limitation addressable.
    pub feasibility_envelope: String,
    /// Initial prior-art query plan (T-018 consumes this).
    pub prior_art_query_plan: Vec<String>,
    pub unanswered_questions: Vec<String>,
    /// Later audit hook for rediscovered known solutions (T-018 labels;
    /// T-014 never claims novelty — R-009 boundary).
    pub rediscovery_hint: Option<String>,
    pub validity: Validity,
    /// Narrative completeness 0..=N (count of filled §7:150 fields).
    /// Deliberately separate from `validity` (R-021).
    pub narrative_polish: usize,
    /// Set when a candidate was created from conjecture with no grounding
    /// evidence (MASTER_SPEC:150). Speculative opportunities are emitted
    /// only through `Opportunity::speculative`, never by operators that
    /// require evidence.
    pub speculative: bool,
}

impl Opportunity {
    /// Stable discriminant name for ordering and reason codes.
    pub fn kind_name(&self) -> &'static str {
        self.kind.kind_name()
    }

    /// A grounded opportunity: evidence present, validity evidence-backed.
    pub fn grounded(kind: PressureKind, evidence: Vec<Span>) -> Self {
        Self {
            kind,
            problem_statement: String::new(),
            beneficiary: String::new(),
            context: String::new(),
            evidence_references: evidence,
            suspected_bottleneck: None,
            causal_uncertainty: String::new(),
            value_estimate: ValueEstimate::Unclear,
            feasibility_envelope: String::new(),
            prior_art_query_plan: Vec::new(),
            unanswered_questions: Vec::new(),
            rediscovery_hint: None,
            validity: Validity::EvidenceBacked,
            narrative_polish: 0,
            speculative: false,
        }
    }

    /// A conjecture-only candidate: exists, flagged speculative, never
    /// evidence-backed until grounding is attached (MASTER_SPEC:150).
    pub fn speculative(kind: PressureKind) -> Self {
        let mut o = Self::grounded(kind, Vec::new());
        o.validity = Validity::Speculative;
        o.speculative = true;
        o
    }
}

/// Why a candidate was rejected. Rejections are retained for the
/// discarded-opportunity audit (MASTER_SPEC:152), never dropped.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectedCandidate {
    pub kind: PressureKind,
    /// Machine-stable reason code plus human-readable detail.
    pub reason: String,
    pub detail: String,
}

/// Operator outcome: emitted opportunities plus retained rejections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisOutcome {
    pub opportunities: Vec<Opportunity>,
    pub rejected: Vec<RejectedCandidate>,
}

/// Narrative-completeness count: how many §7:150 narrative fields are
/// filled. Deliberately SEPARATE from validity (R-021/AT-021): a high
/// count never lifts the grounding verdict, and a low count never lowers
/// evidence-backed status.
pub fn narrative_polish(opp: &Opportunity) -> usize {
    [
        !opp.problem_statement.is_empty(),
        !opp.beneficiary.is_empty(),
        !opp.context.is_empty(),
        opp.suspected_bottleneck.is_some(),
        !opp.causal_uncertainty.is_empty(),
        !opp.feasibility_envelope.is_empty(),
        !opp.prior_art_query_plan.is_empty(),
        !opp.unanswered_questions.is_empty(),
        opp.value_estimate != ValueEstimate::Unclear,
    ]
    .iter()
    .filter(|b| **b)
    .count()
}

/// Grounding verdict derived from the evidence actually attached (R-021):
/// evidence-backed only when span-bound references exist; otherwise the
/// candidate is speculative — no matter how complete its narrative.
pub fn assess_validity(opp: &Opportunity) -> Validity {
    if !opp.evidence_references.is_empty() {
        Validity::EvidenceBacked
    } else {
        Validity::Speculative
    }
}
