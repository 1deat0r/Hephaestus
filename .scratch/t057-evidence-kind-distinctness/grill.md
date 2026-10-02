# T-057 grill - R-007: evidence kinds distinct (observation vs assumption vs model judgment vs derivation)

Goal: R-007/AT-007 (Verification service; contract M0, runtime M2,
MASTER_SPEC §3) — negative: "Ingest an unsupported model statement
repeatedly." Required: "No repetition promotes it to empirical
evidence."

Existing surfaces (facts):
- The six-value `contracts::generated::EvidenceEvidenceType` enum
  exists (schema-authored) with the Evidence record carrying it — but
  NOTHING in crates/python cites R-007, exercises the four kinds, or
  gates promotion (run-24 re-audit).
- knowledge owns ingestion + spans (`ingest_bytes`, `link_edge`,
  `verify_span`) — the natural home for a grounding-based
  verification gate: an OBSERVATION is grounded in a source span; a
  model judgment is not.
- Corpus EdgeTypes (Supports/Contradicts/…) are edge semantics, not
  evidence kinds — no collision.

## Q1 - What is the mechanism that makes the negative honest?
**A:** Promotion to Observation REQUIRES a grounded span; repetition
count is never a substitute. `attest_evidence_class(declared,
&StatementIngestion)` in knowledge: declared Observation without a
grounded span → `ClassifyError::UngroundedObservation { ingestions }`
(the count rides in the error, proving repetition was seen and
rejected as a basis); declared non-Observation → Ok (the kind stands,
repetitions or not). Plus a distinctness test: the four kinds
(observation/assumption/model_judgment/derivation) round-trip
through the GENERATED enum distinctly — different kinds on identical
text are different records, never conflated. (agent-default)

## Q2 - Why reuse the generated enum instead of a new one?
**A:** Single source of truth — the contract defines the kinds; a
second enum would drift from the schema (and the schema is the frozen
envelope, out of bounds). The gate imports
`contracts::generated::EvidenceEvidenceType` directly.
(agent-default)

## Q3 - Where do tests live?
**A:** New `tests/evidence_class.rs` (clear seam name; allowlist +1
→ +4 with scratch): deny-first (ungrounded observation refused with
ingestion count, all-four round-trip distinctness, grounded
observation accepted). Citations R-007/AT-007 — the negative/required
map exactly to the gate, so AT-007 IS mirrored honestly here.
(agent-default)

## Q4 - Scope?
**A:** No schema/generator edits; no changes to ingest_bytes/edges;
`StatementIngestion` is the gate's own input type (text, repetition
count, optional grounded span — composed by callers from Corpus
data). No commit (rule 5). (agent-default)
