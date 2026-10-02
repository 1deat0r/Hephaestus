# 02: Extract shared facet predicates from the contract-layer grant check (prefactor)

**What to build:** The binding logic that today lives inline inside the contract-layer grant check becomes shared pure predicates (mission-ref, policy chain, artifact, operation, destination, capability subset, cost-within-cap, time window) used by both that check and — later — the policy engine, so two implementations cannot drift.

**Blocked by:** None (can start immediately).

**Status:** done

- [ ] Predicates are pure functions (facts in, verdict out; clock passed in, no I/O, no trust reads)
- [ ] The contract-layer check is rewritten to call them with **no observable behavior change on contract-valid inputs**: public API unchanged, aggregate error codes unchanged; the one deliberate delta (malformed cost records with absent currency now deny instead of passing) is deny-direction, documented in spec Further Notes, and pinned by `malformed_cost_never_authorizes` (pass-2 amendment)
- [ ] All existing contract-layer deny tests pass unmodified (no test file edits allowed in this ticket)
- [ ] In-module unit tests cover each predicate's boundary (equal-cost allowed, window edges, subset edges)

## Comments

Closed 2026-10-02 — work landed earlier; verified green: policy_engine and capability_grant tests green in make ci.
