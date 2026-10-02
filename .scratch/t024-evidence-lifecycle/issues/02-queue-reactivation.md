# 02: Portfolio queue + reactivation

**Status:** done

**What to build:** `queue` (budget-gated: BudgetUnavailable refusal),
`reactivate` (new version referencing original failure + changed-condition
evidence, original retained), re-evaluation dispatch respecting the
budget+permission gate.

**Acceptance:**
- [x] Budget refusal (spec AC 5)
- [x] Reactivation new version + original intact (spec AC 5)
- [x] Twin-run byte-identical (spec AC 7)

## Comments
