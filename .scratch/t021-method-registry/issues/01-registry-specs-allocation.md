# 01: Method registry, specs, error allocation, zero-failure bound

**Status:** done

**What to build:** `methods` module: `MethodSpec` (name/version, estimand
with denominators + cost fields, assumptions, error allocation,
missingness policy enum, clipping policy, identity digest), immutable
registry (duplicates rejected, new version = new entry), `bonferroni_alpha`,
`zero_failure_bound` (planning-only label).

**Acceptance:**
- [x] Registry binds all fields; duplicates rejected (spec AC 1)
- [x] bonferroni splits + sums (spec AC 2)
- [x] zero_failure_bound(299, 0.05) <= 0.01 (spec AC 3)
- [x] Missingness enum strict defaults (spec AC 5)

## Comments
