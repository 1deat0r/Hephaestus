# T-050 spec - R-083: compare proposed context caching against a credible baseline

Status: ready-for-agent

## Problem Statement

R-083 (M3): a benchmark review of proposed context caching must
compare against a credible existing baseline — running only full
reconstruction while a stronger exact-cache implementation exists is
refused unless the exclusion is justified (AT-083 negative → required
outcome: review requests the stronger comparator or justifies
exclusion). §28: "A full-rebuild straw baseline alone is inadequate."
No caching/comparator concept exists in src (zero `cach` hits in
acceleration).

## Requirements trace

- R-083/AT-083 (OBLIGATIONS; enforcement: Protected context oracle and
  evaluator; contract M0, runtime M3). Source: MASTER_SPEC §28.
- Continuity: acceleration's matched-ablation discipline (envelopes,
  recorded measurements) is the comparison home; MechanismKind (six M5
  mechanisms) untouched.

## Design

Extend `acceleration`:

- `CachingComparator { FullReconstruction, ExactCache }`.
- `CachingComparison { comparators_used: Vec<CachingComparator>,
  exclusion_justification: Option<String> }`.
- `review_caching_comparison(&CachingComparison)`:
  - ExactCache among the used comparators ->
    `Ok(CachingReview::StrongerComparatorUsed)`;
  - else non-empty exclusion justification ->
    `Ok(CachingReview::ExclusionJustified { justification })`;
  - else -> `Err(CachingReviewError::
    MissingStrongerComparatorOrJustification)` (typed-rejection
    convention) — the negative case.

## Acceptance Criteria

1. **AT-083 negative**: comparison using only full reconstruction
   with no justification → `MissingStrongerComparatorOrJustification`.
2. **Required outcome (a)**: ExactCache used → `StrongerComparatorUsed`.
3. **Required outcome (b)**: ExactCache excluded with a justification
   → `ExclusionJustified` carrying it; empty-string justification
   still refuses.
4. Glossary rows: Stronger comparator, Exclusion justification.
5. Gates green (`make ci` + `make doc-check`), allowlist +3 + reseal,
   tests cite R-083/AT-083.

## Out of Scope

- Wiring into the protected context oracle/evaluator (composition —
  callers invoke the review seam).
- Implementing an actual cache (the obligation is comparison rigor).
- MechanismKind / AblationPair / PromotionVerdict changes.
- Committing or pushing (rule 5).

## Further Notes

Grill: `.scratch/t050-context-caching-baseline/grill.md`.
