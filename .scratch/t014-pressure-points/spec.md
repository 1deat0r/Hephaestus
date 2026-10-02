# T-014 spec — pressure-point operators over the corpus

Status: ready-for-agent

## Problem Statement

M2 discovery must originate grounded opportunities from pressure points, not
supplied ideas (R-019). T-012 compiled missions carry no hypothesis; T-013
built the evidence substrate (Corpus, spans, edges). Nothing yet turns
authorized traces + corpus evidence into structured `Opportunity` records.
Today there is no bottleneck extraction, conflicting-objectives mining,
failure-pattern analysis, anomaly detection, assumption mining, or
changed-capability detection (IMPLEMENTATION_PLAN:52, MASTER_SPEC §7).

## Solution

A pure `discovery` module (no I/O, no ledger coupling — mission-compiler
precedent). Input: `&Corpus` (evidence substrate) + `&[TraceRecord]`
(structured authorized trace records whose evidence binds to corpus spans).
Output: `AnalysisOutcome { opportunities: Vec<Opportunity>, rejected:
Vec<RejectedCandidate> }`. Six operators, one per pressure-point type from
IMPLEMENTATION_PLAN:52. Every emitted opportunity carries ≥1 evidence
reference (span-bound) or explicit uncertainty; conjecture-only candidates
are constructible but flagged `speculative` (MASTER_SPEC:150).

## User Stories

1. As a discovery caller, I want bottleneck extraction from authorized
   trace records, so that a hidden performance bottleneck becomes a cited
   opportunity (R-019 negative case: AT-019).
2. As a discovery caller, I want anomaly detection that demands
   prediction + observation in a shared unit under comparable conditions,
   so that a fake anomaly from incompatible units is rejected before any
   promotion (R-020, AT-020; MASTER_SPEC:144).
3. As a readiness caller, I want opportunity validity (evidence-backed /
   speculative / challenged) reported independently of narrative polish, so
   that polished-but-unevidenced candidates never score as grounded needs
   (R-021, AT-021).
4. As a genesis caller, I want conflicting-objectives and failure-pattern
   operators, so that trade-offs require evidence of coupling in the
   relevant regime (MASTER_SPEC:144) and recurring failures surface with
   their pattern evidence.
5. As a genesis caller, I want assumption mining and changed-capability
   detection, so that unstated assumptions and newly-verified capabilities
   (with the concrete constraint they change — MASTER_SPEC:144) become
   explicit opportunities.
6. As an auditor, I want rejected candidates retained with reasons (never
   silently dropped), so that the discarded-opportunity audit (§7:152) has
   its substrate.

## Out of Scope

- Mechanism operators (T-015), hypothesis compilation (T-016), search
  orchestration / yield metrics / diversity reserve (T-017), prior-art
  investigation and novelty language (T-018).
- Fabricated numeric value estimates: `value_estimate` is a qualitative
  bucket with justification; no invented numbers.
- Traces from network or live system capture: `TraceRecord` is data given
  by the caller; capture is a later concern.

## Acceptance Criteria

1. Hidden-bottleneck trace (AT-019 negative case) → bottleneck opportunity
   citing the slow span; twin runs byte-identical.
2. Anomaly with mismatched units (AT-020 negative case) → `rejected` with
   reason, zero emitted opportunities.
3. Polished candidate with no evidence → `speculative` validity, low
   validity independent of narrative completeness (R-021).
4. Trade-off opportunity requires coupling evidence in-regime; bare
   counter-movement without coupling evidence is rejected.
5. Changed-capability opportunity requires verified availability + the
   concrete prior constraint it changes (MASTER_SPEC:144).
6. Every emitted opportunity carries ≥1 evidence reference or explicit
   uncertainty; rejection reasons are recorded for every rejected candidate.
7. `cargo test --workspace` green; clippy `-D warnings` clean; fmt clean;
   `make ci` EXIT=0 (allowlist + seal updated for new files).

## Success Risks

- Operators drifting into heuristic soup: each operator gets a named
  contract (what it accepts, what it rejects, why) tested at the seam.
- Vocabulary drift: GLOSSARY rows written FIRST (`Pressure point`,
  `Opportunity`), domain.md vocabulary rule followed.
