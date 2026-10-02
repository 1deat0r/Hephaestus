# 01: Grounding gate for evidence kinds — repetition never promotes

**What to build:** The R-007 verification gate: a declared
Observation without a grounded source span is refused with a named
error carrying the (seen and rejected) ingestion count; model
judgments stand unchanged however often they re-appear; and the four
R-007 kinds prove pairwise-distinct through the schema enum — so the
AT-007 negative (repeated unsupported model statement) can never
become empirical evidence.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] AT-007: ungrounded declared-Observation (repeated) →
      `UngroundedObservation { ingestions }`; same input declared
      ModelJudgment → Ok
- [x] Grounded Observation → Ok
- [x] Four kinds pairwise-unequal across serde round-trip
- [x] Tests cite R-007/AT-007; allowlist +4 + reseal; gates green

## Comments

Grill: `.scratch/t057-evidence-kind-distinctness/grill.md`.
Spec: `.scratch/t057-evidence-kind-distinctness/spec.md`.
