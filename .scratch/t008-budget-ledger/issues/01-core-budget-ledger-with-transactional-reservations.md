# 01: Core budget ledger with transactional reservations

**What to build:** A single-currency ledger over integer minor units whose
`reserve` is all-or-nothing against the shared bound, whose `commit` turns a
reservation into exactly its spend, and whose `release` returns capacity —
so `available + reserved + spent` never leaves the authorized limit, which
is what a scheduler needs to dispatch against (AT-055's sequential half).

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] `BudgetLedger::new(limit: Money)` fixes the currency; read model
      exposes `limit/available/reserved/spent` as pure sums
- [ ] `reserve(id, amount)` is transactional: over-limit, duplicate id,
      wrong currency, negative, or overflowing amounts are refused with
      typed errors and mutate nothing (R-055 / AT-055)
- [ ] `commit(id, actual)` converts the reservation to spend exactly;
      remainder returns to capacity; `actual > reserved` →
      `RESERVATION_EXCEEDED`; unknown id → `NOT_FOUND`
- [ ] `release(id)` returns capacity; unknown id → `NOT_FOUND`
- [ ] Invariant `available + reserved + spend == limit` holds after every
      successful call and after every refusal (asserted across the deny
      matrix)
- [ ] Integer-only checked arithmetic — no floats; overflow/negative denied
- [ ] In-module unit tests for arithmetic edges; deny-first integration
      tests citing R-055/AT-055; zero new dependencies
