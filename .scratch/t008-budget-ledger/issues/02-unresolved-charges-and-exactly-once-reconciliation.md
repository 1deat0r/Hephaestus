# 02: Unresolved charges and exactly-once reconciliation

**What to build:** The external-charge limb: an ambiguous provider charge is
recorded as an unresolved entry with an explicit optional amount and a
mandatory reason (null-with-reason, never a silent zero), holds capacity
when its amount is known, and settles through `reconcile` exactly once —
refusing a replay — so AT-056's "unknown or reconciled, never blindly
duplicated" has a runtime home.

**Blocked by:** 01 (core budget ledger).

**Status:** done

- [ ] `mark_unresolved(id, amount: Option<Money>, reason)` records the entry;
      missing reason, duplicate id, wrong currency, or negative amount →
      typed refusal, nothing written (R-056 / AT-056)
- [ ] Unknown amount (`None`) registers in `unresolved` count without
      inventing a numeric value — never a zero (MASTER_SPEC:408)
- [ ] Known amounts reduce `available` while unresolved
- [ ] `reconcile(id, settled)` moves the entry to spend exactly once;
      second call → `ALREADY_RECONCILED`; wrong currency → typed refusal
- [ ] Read model reports `unresolved` (known total) and
      `unresolved_count()` (entries) separately
- [ ] Tests cite R-056/AT-056 in headers; deny-first coverage of every
      refusal path

## Comments

Closed 2026-10-02 — work landed earlier; verified green: budget_ledger concurrency tests green (M0_M1 receipts).
