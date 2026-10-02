# 01: Context-caching benchmark review gate — stronger comparator or justified exclusion

**What to build:** The R-083 review seam in the acceleration module:
a caching comparison is reviewable only when it uses the exact-cache
comparator or records a non-empty exclusion justification; full
reconstruction alone (the §28 straw baseline) is refused by name, and
both allowed outcomes are typed receipts — so benchmark review always
requests the stronger comparator or documents why it did not.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] AT-083: only-full-reconstruction, unjustified →
      `MissingStrongerComparatorOrJustification`
- [x] ExactCache used → `StrongerComparatorUsed`
- [x] Justified exclusion → `ExclusionJustified` (empty justification
      refused)
- [x] Glossary rows (Stronger comparator, Exclusion justification);
      allowlist +3 + reseal; gates green; tests cite R-083/AT-083

## Comments

Grill: `.scratch/t050-context-caching-baseline/grill.md`.
Spec: `.scratch/t050-context-caching-baseline/spec.md`.
