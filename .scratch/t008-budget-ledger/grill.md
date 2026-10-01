# Grill — T-008: budget ledger

Self-interview (auto-workflow Phase 2, cycle under the user's perpetual
directive). All questions answered from codebase/spec evidence via the
auto-answer policy. Frontier emptied in one round.

Goal under grill: *Implement T-008: budget ledger with integer minor currency
units, transactional reservations, spend reconciliation, release, and explicit
unresolved external charges — concurrent reservations must never oversubscribe
the shared budget.*

## Evidence gathered first

- `IMPLEMENTATION_PLAN.md` T-008: "integer minor currency units, reservations,
  spend reconciliation, release, and unresolved external charges. Unknown
  prices are explicit. Concurrent reservations must not oversubscribe the
  shared budget." Depends on M0; T-009 depends on T-006+T-007+T-008.
- MASTER_SPEC §19 (line 373): "Budget accounting uses **reserved, spent,
  released, and unresolved** amounts in a defined currency. Concurrent
  operations reserve before execution. Provider usage is reconciled after
  completion." Line 408: "**Numeric unknowns are null with a reason, not
  zero.**" Line 410: `BudgetLedger` is a core interface.
- R-055/AT-055 (reserve transactionally before concurrent dispatch;
  "committed reservations plus spend do not exceed the authorized bound")
  and R-056/AT-056 (ambiguous external effects → "unknown or reconciled,
  never blindly duplicated") — both Contract M0, **Runtime M1**, enforcement
  "Scheduler and BudgetLedger" — the BudgetLedger half is this task.
- Contract `money` type exists (`currency` `[A-Z]{3}` + `minor_units: i64`),
  already used by grants/policy.
- Existing cost logic (`authorize_grant` / policy engine) checks a per-record
  cost against a grant cap — there is no shared reservoir anywhere yet.
- Scheduler (line 367) owns dispatch and reservations as a *service*; the
  ledger is the substrate T-009 will sit on.

## Round 1

❓ **Q1 — Scope cut**: what lands, what is deferred?

➡️ **In:** a single-currency in-process `BudgetLedger` over integer minor
units with four accounting states (reserved, spent, released, unresolved),
transactional `reserve` (all-or-nothing against `limit − spent − reserved −
unresolved`), `commit` (reservation → spend, remainder released), `release`,
`mark_unresolved` (explicit unknown amount + reason — null-with-reason),
`reconcile` (unresolved → resolved spend, exactly once), read-side
`available()/reserved()/spent()/unresolved()`. **Deferred with reasons:**
persistence/event replay (T-011 owns recovery), the scheduler/DAG that will
call reservations (T-009), FX conversion and price versions (MASTER_SPEC
records them when conversion is used — no conversion here, cross-currency is
an error), provider usage metering (T-010 receipts), UI. *source:
agent-default · plan + MASTER_SPEC:373/408.*

❓ **Q2 — Location**: where does the module live?

➡️ **`crates/hephaestus/src/budget/`** — `BudgetLedger` is named a core
interface next to `PolicyEngine` (MASTER_SPEC:410), and run 3 established the
pattern of one top-level module per core interface (`policy/`). Reuses the
generated `Money` type. *source: agent-default · MASTER_SPEC + run-3
precedent.*

❓ **Q3 — Concurrency model**: how can "never oversubscribe" hold?

➡️ **`&mut self` on every mutating call** — exclusive borrow makes the
check-and-debit a single atomic step in-process; no lock can be held across
a gap, so oversubscription is structurally impossible for one ledger.
Evidence: an integration test drives many threads through a `Mutex<…>`
wrapper hammering `reserve` against a small limit and asserts
`spent + reserved + unresolved <= limit` at the end (AT-055's
"many tasks, small shared budget"). Runner-up — internal `Mutex` — rejected:
shoves the same guarantee behind a lock the API then cannot express, and
`&mut self` already provides it. *source: agent-default · Rust ownership +
AT-055.*

❓ **Q4 — Arithmetic rules**: floats? overflow? exceeding reservations?

➡️ **Integer-only, checked arithmetic, no floats anywhere** (the plan says
integer minor units; a float would violate determinism and unit-exactness).
`checked_add`/`checked_sub`: overflow is an explicit error, never wrapping.
`commit(actual)` where `actual > reserved` is **denied**
(`RESERVATION_EXCEEDED`) — fail-closed; the caller reserves the delta rather
than letting spend silently blow past its reservation. Currency must match
the ledger's currency exactly (`CURRENCY_MISMATCH`) — no conversion (Q1).
Negative amounts denied. *source: agent-default · fail-closed + determinism.*

❓ **Q5 — Unknown prices**: how is "explicit" represented?

➡️ **`mark_unresolved(charge_id, amount: Option<Money>, reason: String)`** —
`None` + reason is the MASTER_SPEC:408 "null with a reason, not zero"
encoding; an unresolved charge holds capacity at its amount when known, and
holds *no* numeric amount (but a required reason) when unknown, counting as
an unresolved *entry* in `unresolved_count()` — never a silent 0 that would
understate exposure. `reconcile(charge_id, settled: Money)` resolves it
exactly once (`ALREADY_RECONCILED` on replay — AT-056's "never blindly
duplicated"). Unknown amount + no reason = refused at entry.
*source: agent-default · MASTER_SPEC:408 + AT-056.*

❓ **Q6 — TDD seams**: where does `tdd` drive?

➡️ Public entry points of the `budget` module: `new/limit/available/
reserved/spent/unresolved`, `reserve`, `commit`, `release`, `mark_unresolved`,
`reconcile`. Unit tests inside the module for arithmetic/invariant edges;
deny-first integration tests in `tests/budget_ledger.rs` (repo convention).
*source: agent-default · policy test-seams row.*

❓ **Q7 — Dependencies / test infra**?

➡️ **Zero new deps** (Money contract type + std only); thread tests use
`std::thread` + `std::sync::Mutex` (std). No temp dirs — the ledger is
in-memory (persistence deferred, Q1). *source: agent-default · Cargo.toml.*

❓ **Q8 — Determinism & clock**?

➡️ No clock, no randomness: state transitions happen in call order; read-side
totals are pure sums over typed entries. Same call sequence ⇒ same state
(R-052's deterministic-gates spirit applies to money math too). Tests freeze
inputs like the policy engine tests do. *source: agent-default.*

❓ **Q9 — Obligations & docs**: R-IDs, glossary, ADR?

➡️ **R-055/AT-055 and R-056/AT-056 cited in tests and commit material** (real
mapped obligations — Contract M0, Runtime M1). **GLOSSARY gains** *budget
ledger*, *reservation*, *unresolved charge* — decision row written **before**
the edit (corrected domain.md rule, run-4 discipline). **No ADR** (executes
the approved plan; not a departure). No `requirements.json`/schema/generated
changes (money contract already exists). *source: agent-default · OBLIGATIONS
+ domain.md.*

❓ **Q10 — Ticket shape**?

➡️ **Three tickets:** 01 core ledger (reserve/commit/release + invariant +
sequential AT-055 deny tests); 02 unresolved/reconcile limb (unknown prices,
exactly-once reconciliation, AT-056); 03 concurrency proof (multi-thread
oversubscription suite) + glossary terms + gate evidence. Edges: 1→2, 1→3.
Each slice is independently verifiable and context-sized; 03 doubles as the
run's evidence ticket. *source: agent-default · vertical-slice sizing +
dedup (STATE has no tickets for this spec).*

## Frontier status

Empty. No refusal-category item; nothing deferred to a later round.
