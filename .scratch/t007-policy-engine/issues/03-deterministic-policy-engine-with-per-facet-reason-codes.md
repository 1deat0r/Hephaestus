# 03: Deterministic policy engine with per-facet reason codes

**What to build:** A pure `evaluate` that takes the enumerated authorization facts, mission state, and protected trust/clock context and returns allow-or-deny with a stable, specific reason code per broken facet — fail-closed, byte-identical on identical inputs, and structurally incapable of carrying provider credentials or model input.

**Blocked by:** 01 (typed capability grants), 02 (facet predicate extraction).

**Status:** done

- [ ] Denies with a specific reason per facet: mission authorization, operation, capabilities, destination, artifact, policy chain, revocation, expiry/interval, cost, trust, missing budget (AT-060 / R-060)
- [ ] Fail-closed: every missing or unparseable fact yields a reason, never a skip; the positive control (exact, unexpired, approved, trusted grant) evaluates allowed
- [ ] Reasons emitted in fixed declaration order; identical inputs (frozen clock, frozen trust) produce byte-identical decisions (R-052)
- [ ] Trust read only from the protected trust context: a grant with no trust recorded there — even one that is internally consistent and approved — denies with `GRANT_NOT_TRUSTED` (R-095; the engine never consults grant-internal assertions; constructing a trusting context is the protected layer's job, outside this API); clock read only from the context's evaluated time (no system clock)
- [ ] Request/state/decision structs contain only enumerated authorization facts — no credential field, no provider handle, no model parameter
- [ ] Consistency invariant tested: on every shared facet, engine denial ⇔ contract-layer check error
- [ ] Reuses the ticket-02 predicates; does not fork their logic

## Comments

Closed 2026-10-02 — work landed earlier; verified green: policy_engine and capability_grant tests green in make ci.
