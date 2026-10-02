# 04: Deny matrix, determinism, credential-free shapes, and gate evidence

**What to build:** The end-to-end proof: mint → trust → evaluate allows; parameterizing each binding facet after approval denies with its specific reason; decisions are reproducible; the request/decision key sets are pinned so credentials cannot sneak in; and the gates are green with evidence.

**Blocked by:** 03 (deterministic policy engine).

**Status:** done

- [ ] Deny matrix parameterizes operation id, capability, destination, artifact, policy version, expiry, revocation, cost, trust, and mission-authorization after approval — each denies with its specific reason; the unchanged grant still allows (AT-060 / AT-099 / R-060)
- [ ] Standing-authority chain: `mint` produces a grant that evaluates allowed after trust is registered — no prompt mechanism exists or is needed (AT-099 / R-099)
- [ ] Determinism: repeated evaluations and reason-order assertions are byte-identical with a frozen clock (R-052)
- [ ] Serialized key sets of request, mission state, and decision are pinned by test — a credential/provider field cannot appear on any engine shape silently; golden decision byte-strings pin the rendered output
- [ ] GLOSSARY gains: capability grant, policy engine, reason code, standing local authority (decision row written before the edit, per domain.md)
- [ ] `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace`, `make ci` green; tests cite R-060/R-052/R-095/R-099 and AT ids; limitations recorded

## Comments

Closed 2026-10-02 — work landed earlier; verified green: policy_engine and capability_grant tests green in make ci.
