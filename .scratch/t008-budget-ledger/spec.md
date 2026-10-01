# Spec — T-008: budget ledger

Status: ready-for-agent
Goal source: derived:roadmap (M1 sequence under the user's perpetual-operation directive)
Grill record: `.scratch/t008-budget-ledger/grill.md` (10 questions, all self-answered)

## Problem Statement

The control plane can price-check a single request against a grant cap, but
it has no shared reservoir: nothing tracks how much of an authorized budget
is reserved, spent, released, or stuck as an unresolved external charge, so
concurrent operations could collectively exceed the bound and an ambiguous
provider charge would have nowhere to live except a silent zero. R-055's
"committed reservations plus spend do not exceed the authorized bound" and
R-056's "unknown or reconciled, never blindly duplicated" have no runtime
home, and T-009's scheduler has no substrate to reserve against.

## Solution

An in-process `BudgetLedger` over integer minor currency units, single
currency per ledger, with four accounting states straight from MASTER_SPEC
§19 — **reserved, spent, released, unresolved**:

1. **Transactional reservation** — `reserve` is check-then-debit in one
   `&mut self` step against `limit − spent − reserved − unresolved`; it
   either commits fully or changes nothing, so oversubscription is
   structurally impossible in-process.
2. **Spend reconciliation** — `commit` turns a reservation into spend
   exactly (remainder released; actual > reserved denied), `release`
   returns capacity, `mark_unresolved` records an external charge with an
   explicit optional amount + mandatory reason (null-with-reason, never a
   silent zero), and `reconcile` settles it exactly once.
3. **Read side** — `available()/reserved()/spent()/unresolved()` are pure
   sums; no clock, no floats, checked arithmetic throughout.

## User Stories

1. As a scheduler, I want `reserve` to be all-or-nothing, so that a losing
   racer leaves the ledger byte-identical to before it tried.
2. As an operator, I want `available + reserved + spent + unresolved == limit`
   at every quiescent point, so that the bound is checkable, not aspirational.
3. As an operator, I want `commit` to convert exactly the reserved amount,
   so that spend can never exceed what was authorized for it.
4. As a caller, I want `commit(actual > reserved)` denied with
   `RESERVATION_EXCEEDED`, so that growth requires a fresh reservation
   instead of silently blowing past the bound.
5. As a caller, I want `release` to return a reservation's capacity intact,
   so that abandoned work does not leak budget.
6. As a caller, I want a currency mismatch (or a negative or overflowing
   amount) refused with a typed error, so that money math is unit-exact and
   deterministic — floats appear nowhere.
7. As an auditor, I want an external charge of unknown price recorded as
   `None` with a mandatory reason, so that unknowns are null-with-reason and
   never a zero that understates exposure (MASTER_SPEC:408).
8. As an auditor, I want an unknown-price charge to still register as an
   unresolved *entry* in the read model, so that its presence is visible even
   without an amount.
9. As an auditor, I want `reconcile` to settle an unresolved charge exactly
   once and refuse a second time with `ALREADY_RECONCILED`, so that AT-056's
   "never blindly duplicated" holds.
10. As a caller, I want `mark_unresolved` without a reason refused, so that
    every unknown carries its explanation.
11. As a test, I want many threads hammering `reserve` through a `Mutex`
    against a small limit, so that AT-055's "many tasks, small shared budget"
    proves the invariant under real contention.
12. As a reviewer, I want R-055/R-056 (and their AT ids) named in the test
    files, so that traceability claims are checkable.
13. As a maintainer, I want zero new dependencies, so that the offline build
    and every gate keep working unchanged.
14. As a reviewer, I want the four accounting states to match MASTER_SPEC
    §19's vocabulary exactly, so that spec, glossary, and code speak one
    language.
15. As a future T-009 scheduler, I want reservations keyed by a caller-chosen
    opaque id with duplicate ids refused, so that operations map 1:1 onto
    reservations without the ledger knowing what an operation is.

## Implementation Decisions

- **Location:** new `budget` module in the control-plane crate; reuses the
  generated `Money` contract type (currency `[A-Z]{3}` + `minor_units i64`).
- **Single currency:** the ledger is created with one currency; any
  mismatched amount is `CURRENCY_MISMATCH` (no conversion — grill Q1).
- **Invariant:** `spent + reserved + unresolved_known + limit-remainder`
  discipline enforced internally: `available = limit − spent − reserved −
  unresolved_known_amounts`, computed with checked arithmetic; every
  mutator re-checks the invariant before committing its write (refusals
  mutate nothing).
- **Concurrency:** `&mut self` mutators; no internal lock (grill Q3). The
  threads test wraps the ledger in `std::sync::Mutex` the way a future
  scheduler will.
- **Arithmetic:** integer-only, `checked_add`/`checked_sub`; overflow,
  negative, and cross-currency inputs are typed errors; `commit(actual >
  reserved)` → `RESERVATION_EXCEEDED` (caller re-reserves the delta).
- **Unknowns:** `mark_unresolved(id, Option<Money>, reason)`; `None` amount
  still counts as an unresolved entry (count, not fake value); reason
  required; `reconcile(id, settled)` → `ALREADY_RECONCILED` on second call,
  `CURRENCY_MISMATCH` on wrong currency.
- **Duplicate reservation ids** refused (`DUPLICATE_RESERVATION`); unknown
  ids on commit/release/reconcile → `NOT_FOUND`.
- **No clock, no randomness:** transitions follow call order; read models
  are pure sums (determinism, grill Q8).
- **Dependencies:** none (grill Q7).
- **Glossary:** *budget ledger*, *reservation*, *unresolved charge* —
  decision row written **before** the edit (run-4 discipline).
- **No ADR** — executes the approved plan; R-055/R-056 are the mapped
  obligations (grill Q9).

## Testing Decisions

- Good tests assert external behavior at the public seam: totals after call
  sequences, typed refusals, and the invariant under contention. In-module
  unit tests may cover arithmetic edges (checked-overflow, sum helpers).
- **Seams:** `BudgetLedger::{new, limit, available, reserved, spent,
  unresolved, reserve, commit, release, mark_unresolved, reconcile}`.
- **Prior art:** `tests/grant_deny.rs` (deny-first matrix, typed errors),
  `tests/policy_engine.rs` (determinism/golden discipline), `tests/event_ledger.rs`
  (reopen/durability style — not needed here: no persistence).
- **Bound tests:** AT-055 (sequential oversubscription denial + concurrent
  thread hammer with final invariant assert), AT-056 (unknown price entry,
  exactly-once reconcile, replay refusal), plus currency/overflow/negative
  refusals and duplicate/unknown id handling; tests cite R-055/R-056/AT ids
  in headers.
- **Suite:** `cargo fmt --check`, `clippy --workspace --all-targets -D
  warnings`, `cargo test --workspace`, `make doc-check`, `make ci` — each
  evidence line (command + rc) recorded in the state LOG.

## Out of Scope

- Persistence and event replay of budget state (T-011 owns recovery);
  the T-006 event ledger integration stays a follow-up (same rationale as
  T-007's grill Q11).
- The scheduler/DAG that will call reservations (T-009) and provider usage
  metering/receipts (T-010/T-011).
- FX conversion, price versions, exchange-rate provenance (MASTER_SPEC
  records them *when conversion is used* — no conversion here).
- Per-grant cost caps (already covered by the policy engine) — this ledger
  is the shared reservoir those caps coexist with, not a replacement.
- Any change to `schemas/contracts.schema.json`, `requirements.json`,
  generated code, or gate files.
- ADR: none — plan execution, not an architectural departure.

## Further Notes

- Limitation: the "never oversubscribe" guarantee is in-process
  (single writer behind `&mut self` or one `Mutex`); MASTER_SPEC:379's local
  MVP design (one authoritative writer, embedded DB) matches this —
  multi-process coordination is the later adapter's problem.
- Limitation: durability is out of scope by T-011's design; a crash loses
  budget state, which is why no ticket claims recovery behavior.
- Determinism note: totals depend only on the call sequence, so the thread
  test asserts the *invariant*, not a specific interleaving.
