# T-020 spec - prototype worker authorization + assembly verification

Status: ready-for-agent

## Problem Statement

Workers must be able to change candidate artifacts while protected
evaluators and baselines stay untouchable, and passing local tests must not
be sufficient for composition. MASTER_SPEC §15 requires interface, version,
invariant, resource-aggregation, interaction, and end-to-end guardrail
verification at assembly; R-094/095 require receipts verified against the
protected context, never worker claims.

## Requirements trace

- R-064: bounded local system, isolated workers (change-level contract
  here; process isolation is T-010's).
- R-094: build/test receipts with verified artifact bytes.
- R-095: trust from protected context, not candidate-supplied assertions.
- §15:296: workers return artifacts + receipts; cannot silently alter
  protected evaluators or the baseline.
- §15:297: assembly verifies interfaces, versions, invariants, resource
  aggregation, interaction effects, end-to-end guardrails.

## Acceptance Criteria

1. `authorize` accepts a candidate-artifact change backed by a verified
   receipt (digest in protected registry).
2. `authorize` rejects evaluator/baseline-targeted changes with a named
   rejection; worker claims of "unchanged" without protected digests are
   rejected.
3. Receipt with unverified digest rejected (R-095).
4. `assemble` verifies pairwise interfaces + versions; missing interface
   named.
5. Resource aggregation: sum of budgets within mission limit; cost-shift
   row accounts total system cost (§15:297).
6. Global invariants + end-to-end guardrails checked; failures name the
   component.
7. Twin-run byte-identical reports.

## Risks

- Over-engineering: no type system, no sandbox here - contract + digest
  verification only.
- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
