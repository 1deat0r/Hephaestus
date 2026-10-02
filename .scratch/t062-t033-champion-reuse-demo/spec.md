# T-062 spec - T-033 champion-reuse demonstration

Status: ready-for-agent

## Problem Statement

The exit assessment's only NOT-RUN M3-core clause (3.5): R-119's
demonstration — a later mission uses the improved persisted champion,
restart preserves the state, an injected regression triggers rollback,
and disqualified challengers stay undeployed. selfimprove has
promote/rollback/ledger but **no proposal step, no champion lookup, and
no consumer** (grep-verified: from_json round-trip is the only
restart evidence today).

## Requirements trace

- R-115 (bounded proposal from observations), R-116 (durable learning,
  restart), R-117 (candidate shape — enforced by existing promote),
  R-118 (deployment/rollback receipts), R-119 (the demonstration
  itself). Source: docs/SELF_IMPROVEMENT.md + MASTER_SPEC §24.
- Continuity: promote()/rollback()/LedgerEntry conventions untouched;
  M2 slice (apply_all bound) and missionrun chain reused as the
  mission body; content-addressed challenger payloads verified against
  candidate digests (same discipline as promote's verified artifacts).

## Design

Extend `selfimprove`:

- `propose_from_observations(...) -> Result<ImprovementCandidate,
  ProposalError>`: bounded deterministic constructor; refuses empty
  observation ids (`UnseededProposal`) and self-budgeting
  (`budget_from == challenger_id` → `SelfBudgeting`) — R-115 checks
  at the proposal edge, mirroring evaluate_candidate's later gates.
- `active_champion(&ImprovementLedger, target) -> Option<Champion>`:
  ordered walk; `deployed` → challenger; later `rolled_back` → the
  deployment's incumbent; no entries → None. Uses existing outcome
  string convention; `rolled_back` documented.

Demo test `tests/champion_reuse.rs` (phased receipts): mission A
(non-seeded opportunity from the fixture trace, default bound 8) →
propose (bound-16 challenger, observation ids) → fixture Assessment
{Supported, Passing, external evaluator} → promote → persist ledger to
temp file → mission B uses championed bound 16 (digest-verified payload
map) → restart from file → identical resolution → injected regression →
rollback receipt → mission C/D use incumbent 8 → four disqualified
challengers (Unsupported, confounded, over-budget, permission-expanding)
each refused by name.

## Acceptance Criteria

1. `cargo test --test champion_reuse` green: all phases above with
   receipts (bound_used/source fields in each mission receipt).
2. `propose_from_observations` refusals typed (unseeded, self-budgeting);
   `active_champion` covers deployed / rolled-back / empty.
3. Existing suites untouched-green (selfimprove, e2e_m2, e2e_m3, cli);
   gates `make ci` + `make doc-check` green; allowlist +2 + reseal.
4. No live canary/drift service claimed — exit-assessment limitation
   line stays (injected regression is the in-bounds demonstration).

## Out of Scope

Live trigger/canary services (typed records remain), model-driven
proposal, commit (rule 5).

## Further Notes

Grill: `.scratch/t062-t033-champion-reuse-demo/grill.md`.
