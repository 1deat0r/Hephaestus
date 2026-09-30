# Spec — T-007: capability grants and the deterministic policy engine

Status: ready-for-agent
Goal source: derived:roadmap (IMPLEMENTATION_PLAN.md M1; untrusted derivation recorded in decisions.md)
Grill record: `.scratch/t007-policy-engine/grill.md` (12 questions, all self-answered)

## Problem Statement

The runtime can recheck a grant as JSON against a trust context, but it cannot
*issue* a scoped capability grant, and its authorization answer collapses most
binding failures into one aggregate error — so a caller learns *that*
dispatch was denied, not *which* binding broke. There is no single deterministic
decision function that takes the enumerated authorization facts and returns a
stable allow/deny with specific reasons. Without that, R-060's "records a
specific reason", R-099's standing-authority minting, and R-052's "the model
cannot become the authoritative decision-maker" have no runtime home.

## Solution

Two additions to the control plane, both pure logic over existing contract
types:

1. **Typed capability grants** — a `CapabilityGrant` binding operation,
   capability scope, destination, expiry, artifact identity, policy version,
   and approved cost, with `mint` (standing local authority), `validate`,
   `revoke`, and lossless round-trip to/from the `authorization_grant`
   contract.
2. **A deterministic policy engine** — `evaluate(request, state, trust) ->
   decision`: a pure, fail-closed function that returns stable, per-facet
   reason codes in a fixed order. It shares its facet predicates with the
   existing contract-layer grant check (extracted, so neither side forks the
   other's logic), takes its clock and trust from the injected context, and
   structurally cannot carry provider credentials or model inputs.

## User Stories

1. As a mission, I want to mint an approved scoped grant for one operation
   without repeated user prompts, so that standing local authority works as
   AT-099 requires.
2. As an operation, I want my grant to bind operation id, capability scope,
   destination, artifact identity, policy version, and expiry, so that
   changing any one of them after approval can no longer authorize me.
3. As an operator, I want `revoke()` to deny every subsequent evaluation with
   `GRANT_REVOKED`, so that withdrawal is immediate and explicit.
4. As a caller, I want `validate()` to reject a reversed or empty validity
   window at every public constructor (`mint`, `from_contract`), so that no
   grant violating its own contract or window can be built through the API —
   the struct's fields stay public for interop, so the engine re-validates
   the window at evaluation time (defense in depth, tested).
5. As a caller, I want grants convertible to and from the
   `authorization_grant` contract without loss, so that typed and contract
   layers interoperate.
6. As a caller, I want the engine to deny with a *specific* reason per broken
   facet, so that AT-060's "records a specific reason" is satisfied and
   debugging is possible.
7. As a caller, I want fail-closed behavior — missing facts produce reasons,
   never skips, so that absent information can only deny.
8. As a caller, I want identical inputs to produce byte-identical decisions,
   including reason order, so that R-052 determinism is checkable.
9. As a reviewer, I want the engine to read trust from a protected context
   and its clock from an injected source, so that R-095 holds and tests can
   freeze time.
10. As a reviewer, I want no credential field, provider handle, or model
    parameter anywhere in the engine's types or signature, so that provider
    credentials stay outside model-visible context by construction.
11. As a maintainer, I want the facet predicates shared with the existing
    contract-layer check, so that two implementations cannot drift.
12. As a maintainer, I want the existing contract-layer check's aggregate
    codes and its 18 deny tests byte-stable, so that nothing weakens.
13. As a test, I want a consistency check proving the engine and the
    contract-layer check agree on every shared facet, so that the refactor
    provably changed granularity, not outcomes.
14. As a test, I want a deny matrix parameterizing operation, capability,
    destination, artifact, policy, expiry, revocation, cost, trust, and
    mission-authorization mismatches, so that AT-060/AT-099 are actually
    exercised.
15. As a future sandbox (T-010), I want a fixed request shape carrying exactly
    the authorization facts, so that later rules can compose without
    redesigning the seam.

## Implementation Decisions

- **Module layout:** new control-plane module `policy` with a typed-grant
  file and an engine file; it reuses the existing security primitives (trust
  context, digest, contract-layer grant check) rather than reimplementing
  them. The security module stays the contract-layer check; `policy` is the
  runtime decision core (grill Q2).
- **Prefactor first:** facet predicates inline in the contract-layer grant
  check are extracted into shared pure functions; the check's public API,
  aggregate error codes, and behavior remain unchanged (grill Q3).
- **Grant binding fields:** operation id, capabilities (scope), destination,
  artifact sha256 (nullable per contract), policy version, issued/expires
  timestamps, state approved|revoked, max cost (currency + minor units),
  issuer id — mirroring the `authorization_grant` contract exactly; no schema
  changes.
- **Minting:** `mint` validates inputs (including a non-empty, ordered
  validity window) and yields an approved grant — the standing local
  authority of AT-099; minting is not recursively grant-gated (grill Q12).
  Revocation is an explicit state transition.
- **Engine contract:** `evaluate` is pure: fixed request/state/trust inputs →
  decision with `allowed: bool` and ordered reason codes; no I/O, no system
  clock (clock from the trust context's evaluated time), no randomness, no
  iteration-order sensitivity (grill Q6).
- **Reason codes:** stable strings, one per facet, emitted in fixed
  declaration order; fail-closed on every missing fact (grill Q4).
- **Credential exclusion:** request/state/decision are closed structs over
  enumerated authorization facts; a test pins their serialized key sets so a
  credential/provider slot cannot appear silently (grill Q5).
- **Consistency invariant:** on shared facets, engine denial ⇔ contract-layer
  check error, tested; divergence is a bug in the extraction.
- **Dependencies:** none added (grill Q8).
- **Glossary:** *capability grant*, *policy engine*, *reason code*,
  *standing local authority* — decision row written **before** the edit per
  the corrected `domain.md` rule (grill Q10).

## Testing Decisions

- Good tests assert external behavior at the public seam: mint/validate/
  revoke/round-trip outcomes, and evaluate(request, state, trust) decisions.
  No integration test reaches inside; in-module unit tests may cover the
  extracted predicates (spec Testing Decisions rule from T-006 applies here).
- **Seams:** `CapabilityGrant::{mint, validate, revoke, to_contract,
  from_contract}`, `PolicyEngine::evaluate`, shared predicates.
- **Prior art:** `tests/grant_deny.rs` (deny-first matrix, injected clock,
  `TrustContext` setup, `'static` artifact helpers) — the new deny matrix
  mirrors it; `tests/event_ledger.rs` (round-trip style).
- **Bound tests:** AT-060 (per-facet mismatch → specific reason; unchanged
  grant still allowed), AT-099 (mint without prompts; recheck denials),
  R-052 (identical inputs → byte-identical decision; frozen clock), R-060
  (binding), R-095 (trust only from the protected context — a grant absent
  from it denies even when internally consistent; see Further Notes on what
  this package can and cannot yet gate).
- **Consistency suite:** shared-facet agreement between engine and
  contract-layer check.
- **Suite:** `cargo fmt --check`, `clippy --workspace --all-targets -D
  warnings`, `cargo test --workspace`, `make ci` — evidence recorded as
  command + exit code in the state LOG.

## Out of Scope

- Sandbox, network, and secret-boundary rules (R-058/R-059) — enforcement
  needs T-010's sandbox runtime; the engine's fixed request shape is the
  composition seam for them.
- Budget reservations, spend reconciliation (T-008).
- Recording grants/decisions as events in the T-006 ledger — deferred
  follow-up (grill Q11).
- Model/provider integration of any kind — the engine is code-only
  (R-052); no adapter exists or is added.
- Any change to `schemas/contracts.schema.json`, `requirements.json`, or
  generated contract code. **Gate-file exception applied this run:**
  `tools/runtime_allowlist.txt` (+34 entries) and the regenerated
  `tools/gate_seal.sha256` — required because the user-authorized commit of
  the prior run's deliverables made them *tracked*, and the ADR-024 L4
  classifier governs tracked files (decision row 09:20; the classifier's own
  hint points at the allowlist, not a MANIFEST reseal).
- ADR: none — this executes the approved plan; no architectural departure.

## Further Notes

- Limitation to carry into the report: credential exclusion is a structural
  guarantee (no field, no parameter) plus a key-set test; this repo has no
  model context yet, so an end-to-end redaction test is impossible here and
  remains future work when a model boundary exists.
- Limitation: the consistency invariant covers shared facets only; sandbox/
  network rules have no counterpart yet by design.
- Limitation (composition contract): the engine takes the *resolved* grant;
  binding a record's `grant_ref` to the right grant record is the caller's
  job (the contract layer does that binding). The engine consults only
  `TrustContext` for trust — building a trusting context is the protected
  layer's job, outside this API; the package cannot yet gate that
  construction end-to-end because no model/runtime boundary exists in-repo.
- Deliberate tightening vs pre-extraction T-004 (decision row 09:45): the
  old inline cost check compared raw currency Values, so a record missing
  BOTH currencies could pass; the shared predicate requires both sides
  present. Strictly fail-closed; asserted by `malformed_cost_never_authorizes`
  and recorded rather than hidden.
- The contract-layer check keeps its aggregate code by design — code
  granularity differs, outcomes must not.
