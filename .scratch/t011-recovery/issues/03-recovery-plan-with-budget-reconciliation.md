# 03: Recovery plan with budget reconciliation

**What to build:** `recover(view, budget_state) -> RecoveryPlan` implementing
AT-057's rule table with MASTER_SPEC:371's no-blind-retry discipline:
requeue only what is permitted, resolve ambiguous work into the budget's
unresolved limb exactly once, never auto-repair corruption.

**Blocked by:** 02 (replay view from the verified chain).

**Status:** done

- [x] Rule table: Planned ⇒ requeue; Ambiguous+retryable+attempts<max ⇒
      requeue; Ambiguous otherwise ⇒ release held reservation + record
      `unresolved` **exactly once** (idempotent guard on an existing
      unresolved/reconciled event); exhausted ambiguity ⇒ unresolved as
      well (a possibly-completed effect is reconciled, never relabelled a
      settled failure — MASTER_SPEC:371; corrected from "terminal Failed");
      `cancel_requested` ⇒ cancelled (no requeue); Corrupt ⇒ surfaced only
- [x] Plan lists ids for requeue/unresolved/terminal/corrupt in ledger
      order; plan construction is pure (budget mutation happens only in an
      explicit `apply` step, tested separately)
- [x] Apply step: releases reservations it can, calls `mark_unresolved`
      with reasons, never double-applies on a second `apply` run
- [x] Tests cite R-057/AT-057 (+ AT-056 for the exactly-once arm) and
      cover every rule-table row

## Comments

2026-10-01T00:21:34Z — Done; verified by operations_deny (19 tests) + replay unit tests + green gates (279 workspace, doc-check 0, make ci 0, manifest 208/127).

Closed 2026-10-02 — work landed earlier; verified green: operations_deny replay, recovery, and fault tests green (M0_M1 receipts).
