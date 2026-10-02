# T-022 spec - independent evaluator + typed result interpreter

Status: ready-for-agent

## Problem Statement

Experiment plans (T-019) and the method registry (T-021) exist, but nothing
turns measured outcomes into typed, semantically honest results. MASTER_SPEC
§14 requires three separate result dimensions (execution validity /
scientific conclusion / engineering target), threshold interpretation rules
(above/contradict/inconclusive), noninferiority via lower bound vs -m, no
"proven true" representation, and identification of the claim and conditions
each conclusion applies to. Guardrails are checked independently.

## Requirements trace

- R-026 / AT-026: estimand, units, comparator, threshold provenance
  recorded on results.
- R-100: method must be registry-qualified (integration with T-021).
- §14:280: three dimensions, preserved separately; claim + conditions
  identified.
- §14:281: interval vs theta semantics; noninferiority lower bound vs -m.
- §14:282: no proven-true from finite tests.

## Acceptance Criteria

1. TypedResult carries all three dimensions + claim/conditions + provenance
   (R-026, §14:280).
2. Interval above theta -> Supported; below -> Contradicted; overlap ->
   Inconclusive; missing interval -> NotAssessed (§14:281).
3. Invalid/incomplete execution -> both conclusions NotAssessed.
4. Noninferiority: lower bound > -m -> met; else not met (§14:281).
5. No proven-true variant exists (§14:282).
6. Guardrails evaluated independently; failure recorded without flipping
   science dimension.
7. Evaluator digest + registry-qualified method required (R-095, R-100).
8. Twin-run byte-identical.

## Risks

- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
- Over-engineering: interval arrives computed; this module interprets, it
  does not fit distributions.
