# Grill — T-007: capability grants and the deterministic policy engine

Self-interview (auto-workflow Phase 2). Every question the `grilling` skill
would put to the human, answered from codebase evidence + goal via the
auto-answer policy. Frontier emptied in one round; nothing deferred.

Goal under grill: *Implement T-007: capability grants and the deterministic
policy engine — grants bind operation, scope, destination, expiration, artifact
identity where relevant, and approved cost; provider credentials stay outside
model-visible context.*

## Evidence gathered before answering

- `crates/hephaestus/src/security/grant.rs` (T-004): JSON-level
  `authorize_grant(GrantCheck, TrustContext) -> Vec<String>` already rechecks
  trust, mission/policy/artifact/operation/destination/capability bindings,
  cost cap, and the time window — but reports **one aggregate
  `GRANT_BINDING_MISMATCH`** for most binding failures.
- `crates/hephaestus/tests/grant_deny.rs`: 18 deny-first tests (T-004) pin
  that behavior; they must not weaken.
- `schemas/contracts.schema.json`: `authorization_grant` already defines the
  wire contract (operation_id, capabilities, destination, artifact_sha256,
  policy_version, issued_at, expires_at, state approved|revoked, max_cost,
  issuer_id).
- `docs/OBLIGATIONS.md`: R-058/059/060/099 are **Contract M0, Runtime M1** —
  this task is their runtime home.
- AT-060: "Every mismatch denies dispatch/effect and **records a specific
  reason**; valid unchanged grants still work." AT-099: "standing local
  authority can **mint** valid scoped grants without repeated user prompts."
  R-052/AT-052: exact policy gates deterministic — "the model cannot become
  the authoritative decision-maker."

## Round 1

❓ **Q1 — Scope cut**: Does T-007 also cover sandbox/network/secret rules
(R-058/059), budget reservations (T-008), and grant persistence?

➡️ **No — the engine covers the authorization decision facets only**: mission
authorization state, grant binding (operation, capabilities/scope,
destination, artifact identity, policy version), expiry, revocation, cost
within approved cap, trust, all with an injected clock. **Deferred with
reasons:** sandbox/network/secret-boundary enforcement needs T-010's
sandbox/attestation runtime (the engine's request type is the seam it will
later feed); budget *reservations* are T-008 (the engine checks the per-record
cost against the grant's approved cap, as `authorize_grant` already does);
dossier/adoption authorization is later milestones. Recorded as deliberate
omissions, not oversights.
*source: agent-default · policy "keep the tracer bullet that proves the
riskiest assumption".*

❓ **Q2 — Language and location**: Rust where?

➡️ **New top-level module `crates/hephaestus/src/policy/`** (`grant.rs` typed
grant, `engine.rs` evaluation), reusing `security::{grant, trust, digest}`.
Rationale: `PolicyEngine` is named in MASTER_SPEC §410 as a *core interface*
alongside `BudgetLedger`/`SandboxProvider`, and OBLIGATIONS names it as
enforcement service for R-004/006/058/059/060/099 — it deserves its own
module rather than a ninth file under `security/` (which is the T-004
contract-check layer). Runner-up — `security/policy.rs` — rejected to keep
"contract-layer checks" and "runtime decision core" as separate surfaces.
*source: agent-default · MASTER_SPEC/OBLIGATIONS evidence.*

❓ **Q3 — Relationship to the existing `authorize_grant`**: duplicate its
logic, call it, or share it?

➡️ **Share through extracted facet predicates; do not fork, do not change the
old API.** Extract the pure predicates already inline in `grant.rs`
(mission-ref match, policy chain, artifact binding, operation binding,
destination binding, capability subset, cost-within-cap, time window) into
small functions used by **both** `authorize_grant` (whose aggregate
`GRANT_BINDING_MISMATCH` output and 18 tests stay byte-stable) and the new
engine (which reports the *specific* facet AT-060 demands). This is the
prefactor ticket-to-tickets asks for: make the change easy first. Runner-up —
engine calls `authorize_grant` and maps its aggregate code to generic reasons
— rejected: it cannot satisfy "records a specific reason".
*source: agent-default · single-source-of-truth; no weakening of T-004 tests.*

❓ **Q4 — Reason codes**: shape and order?

➡️ **Stable string codes, one per facet** (`MISSION_AUTH_NOT_APPROVED`,
`GRANT_OPERATION_MISMATCH`, `GRANT_CAPABILITY_NOT_GRANTED`,
`GRANT_DESTINATION_MISMATCH`, `GRANT_ARTIFACT_MISMATCH`,
`GRANT_POLICY_MISMATCH`, `GRANT_REVOKED`, `GRANT_EXPIRED`,
`GRANT_INVALID_INTERVAL`, `GRANT_COST_EXCEEDED`, `GRANT_NOT_TRUSTED`,
`RECORD_BUDGET_MISSING`…), returned in a **fixed declaration order** so two
runs over equal inputs yield byte-identical decisions. Deny is fail-closed:
any missing fact is a reason, never a skip. Engine reasons are a superset of
the aggregate; a consistency test asserts that wherever the engine denies,
`authorize_grant` also errors (and vice-versa on the shared facets).
*source: agent-default · AT-060 "specific reason" + R-052 determinism.*

❓ **Q5 — Provider credentials outside model-visible context**: how is that
kept true in-repo, where no model runtime exists yet?

➡️ **API-shape guarantee + tests, honestly labeled.** `PolicyRequest`,
`PolicyState`, and `PolicyDecision` are fixed structs whose fields are the
enumerated authorization facts — there is no credential field, no provider
handle, and `evaluate` is a pure function taking no I/O or model parameters
(R-052: the model cannot be the authoritative decision-maker because it is not
in the type signature). A test pins the serialized key set of
`PolicyRequest`/`PolicyDecision` so a credential slot cannot be added
silently. Limitation for the report: this repo has no model context yet, so
the guarantee is structural here, not yet an end-to-end redaction test.
*source: agent-default · R-052/AT-052 + plan line.*

❓ **Q6 — Clock and trust sources**: where does `now` come from?

➡️ **Injected via the existing pattern:** trust facts from `TrustContext`
(R-095: never candidate-supplied), clock from `TrustContext::evaluated_at`
(external clock, set by the caller — the same convention `authorize_grant`
already uses). The engine reads no system clock itself → determinism is
testable by fixing the context.
*source: agent-default · codebase convention (security/grant.rs).*

❓ **Q7 — TDD seams**: where does `tdd` drive?

➡️ **Public entry points only:** `CapabilityGrant::{mint, validate, revoke,
to_contract, from_contract}` and `PolicyEngine::evaluate`, plus the extracted
facet predicates as the internal seam shared with `authorize_grant`. Unit
tests inside the module for predicates; integration tests in `tests/` for
mint→evaluate and the deny matrix (deny-first, matching the repo's
`*_deny.rs` convention).
*source: agent-default · policy "test seams" row + existing tests layout.*

❓ **Q8 — Test infrastructure / dependencies**: anything new needed?

➡️ **Zero new dependencies, no temp dirs.** Everything is pure logic over
existing types (`serde_json`, `time` already in the crate). The only prior
art needing files is T-006's ledger — not touched here.
*source: agent-default · Cargo.toml evidence.*

❓ **Q9 — M0 prerequisite and gates**: same as T-006?

➡️ **Yes** — Phase 6 runs fmt/clippy/`cargo test --workspace`/`make ci`; a red
M0 gate blocks the ticket rather than being worked around.
*source: agent-default · plan dependency line.*

❓ **Q10 — Documentation obligations**: ADR? GLOSSARY? requirements?

➡️ **No ADR** (executes the approved plan — not an architectural departure).
**No `requirements.json`/traceability edits** (no obligation added; those are
generated/manifest-gated). **GLOSSARY gains terms** — *capability grant*,
*policy engine*, *reason code*, *standing local authority* — and per the
**corrected rule in `docs/agents/domain.md`** (editable with a decision row)
the decision row is written **before** the edit, this time in the right order.
*source: agent-default · domain.md rule as corrected in run 2.*

❓ **Q11 — Record decisions in the T-006 event ledger** (grants minted,
decisions made)?

➡️ **No — deferred as a follow-up.** It couples T-007 to T-006's event API and
is not required by the plan text; the tracer bullets stay independent.
Recorded in the spec's Out of Scope so the omission is deliberate.
*source: agent-default · scope cut.*

❓ **Q12 — Grant issuance authority**: who mints, and is that itself policy
checked?

➡️ **`CapabilityGrant::mint` is the "standing local authority" (AT-099)**: a
caller-supplied issuer id + validated inputs produce an approved grant; minting
is not itself gated by another grant (bootstrap has to start somewhere) — the
*use* of the grant is what the engine gates. Revocation is explicit
`revoke()`, and a revoked grant denies with `GRANT_REVOKED`. This mirrors how
`issuer_id` exists in the contract without a recursive issuer chain.
*source: agent-default · contract evidence + AT-099 wording.*

## Frontier status

Empty. No refusal-category item (no secrets, money, legal, auth grants, or
external publication involved); no question deferred to a later round.
