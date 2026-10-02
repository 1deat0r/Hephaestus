# 02: Revision lineage + inherited-support reassessment (R-027)

**What to build:** `revise(hypothesis, changes) -> Hypothesis`: mints
version+1 with `supersedes`; detects substantive changes (mechanism,
boundary condition, primary endpoint, falsifier) and forces
`inherited_support: NeedsReassessment` on them — a changed mechanism after
favorable results can never silently inherit confirmatory support (AT-027
negative). Unchanged fields inherit normally. Twin-run byte-identical.

**Blocked by:** 01 (Hypothesis record + compiler).

**Status:** done

- [x] `revise` mints version+1 with supersedes lineage (R-027)
- [x] Substantive change (mechanism/boundary/endpoint/falsifier) → `NeedsReassessment` forced (AT-027)
- [x] Unchanged fields inherit support normally
- [x] Twin-run byte-identical revision

## Comments

## Comments
Done 2026-10-01T06:49Z: 4 new tests, 12/12. TDD deviation: revise() landed with ticket 01 seam; ticket 02 tests are regression coverage (AT-027 assertions green first run).
