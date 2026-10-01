# T-012 spec — mission intake & Intent-to-Mission compiler

Status: ready-for-agent

## Problem Statement

M2 cannot begin without a Mission service: today there is no way to turn a
broad authorized goal ("invent something useful", "reduce latency") into a
bounded, versioned, permission-scoped Mission. Without it, downstream engines
(discovery, genesis, experiment) have no intent record, no value frame, and no
authorization boundary to work inside (MASTER_SPEC §4, IMPLEMENTATION_PLAN
T-012, R-001/R-010/R-011/R-012).

## Solution

A pure `mission` module in the Rust runtime: a versioned `Mission` record plus
`compile(goal, profile)` and `revise(previous, change)` functions. Broad goals
compile without any seed hypothesis (R-001); reversible ambiguities become
explicit assumptions while authorization-grade ambiguities become
authorization requests instead of a Mission (R-011); every change produces a
new version with an impact report (R-012).

## User Stories

1. As a system owner, I want to submit a broad goal with my standing
   priorities and resource profile, so that I get a bounded Mission without
   supplying a hypothesis (R-001).
2. As a system owner, I want reversible defaults (terminology, search
   vocabulary, initial corpus, tentative subdomain) recorded as explicit
   assumptions, so that compilation proceeds without repeated prompts.
3. As a system owner, I want unapproved spending, external disclosure,
   irreversible operations, and materially ambiguous risk to come back as
   authorization requests rather than a silently-guessed Mission (R-011).
4. As a system owner, I want each inferred requirement to carry provenance and
   an override path, so that I can correct the compiler's guesses.
5. As a system owner, I want objective and success metrics separated with
   guardrails, so that "reduce latency" cannot compile into "destroy quality
   to go fast".
6. As a downstream engine, I want one of four explicit autonomy profiles
   (plan-only, local-research, sandbox-experiments, supervised-external), so
   that permitted tools and destinations are unambiguous.
7. As a downstream engine, I want actuation and public disclosure to never
   appear in any profile's permitted set, so that hazardous goals fail closed.
8. As a planner, I want goal/cost/dataset/quality changes to mint a new
   Mission version with a changed-sections impact report, so that I can
   invalidate dependent plans (R-012).
9. As a budget owner, I want the resource envelope in T-008 minor-unit money
   with explicit unknown prices, so that the BudgetLedger can enforce it.
10. As an auditor, I want a serde-stable Mission record with no hypothesis
    field, so that "no seed hypothesis" is structural, not a convention.

## Implementation Decisions

- New module `crates/hephaestus/src/mission/` with `record` (Mission,
  AutonomyProfile, Assumption, AuthorizationRequest, ImpactReport types) and
  `compiler` (`compile`, `revise`) submodules; registered in `lib.rs`.
- `compile` input: goal text + structured `Intake` (beneficiary, standing
  priorities, resource profile with T-008 money, requested profile,
  caller-supplied hypothesis as opaque provenance-tagged string if any).
- `compile` output: `Compiled::Mission` or `Compiled::NeedsAuthorization`
  (never both, never a guessed Mission for hazardous goals).
- `revise(previous, change)` output: `(Mission{version: n+1,
  supersedes: n}, ImpactReport{changed_sections})`; plan invalidation stays
  with the caller (scheduler owns dispatch).
- Sensitive-domain detection is keyword-and-profile based and documented as a
  tripwire, not a classifier: it over-triggers into authorization requests by
  design (fail-closed direction documented in module docs).
- No EventLedger coupling (Q12): Mission is a serde record; the caller
  persists. No dependency beyond shared money/serde types.
- No ADR (direct §4 implementation, no departure); no requirements.json
  change (R-001/R-010/R-011/R-012 already exist).

## Testing Decisions

- A good test asserts compiler behavior at the public seam (`compile`,
  `revise`), never internal helpers: given intake X, expect Mission Y or
  NeedsAuthorization Z with exact reason codes.
- Red-first at the `compile` seam for ticket 01, at ambiguity handling for
  02, at `revise` for 03 (TDD discipline per auto-answer policy).
- Prior art: `scheduler_deny.rs` / `sandbox_deny.rs` deny-matrix style for
  the authorization-request matrix; `budget_ledger.rs` for money-boundary
  tests; CLI twin-run determinism style for revise determinism (same change
  twice → byte-identical Mission).
- Determinism: `compile` and `revise` are pure — twin-run byte equality
  asserted for representative intakes.

## Out of Scope

Corpus ingestion (T-013), opportunity/mechanism/hypothesis engines (T-014+),
any UI/TUI/gateway, ledger persistence of Missions, learned or statistical
components of any kind, real external authorization flows (requests are
records; approval plumbing is T-004's HMAC format, already built).

## Further Notes

- GLOSSARY gains mission-compiler terms (decision row written FIRST per
  standing rule).
- New source files must be staged + added to `tools/runtime_allowlist.txt`
  with gate-seal regeneration at landing (ADR-024 standing procedure).
