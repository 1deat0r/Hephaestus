# T-063 spec - crash reconciliation + wired canary (M3 clause 3.5b)

Status: ready-for-agent

## Problem Statement

The exit assessment's 3.5b: interrupted-promotion reconciliation and a
wired guardrail-stop path are NOT-RUN. Underneath, `from_json`'s silent
empty-on-corrupt violates R-116/R-118 directly, and a promotion decided
but unrecorded leaves nothing to reconcile.

## Requirements trace

- R-118 (crash recovery reconstructs champion + reconciles interrupted
  deployments; guardrail violation stops rollout), R-116 (restart loses
  nothing), R-119 (demonstration continuation). Source:
  docs/SELF_IMPROVEMENT.md.
- Continuity: promote()/rollback()/ledger conventions; append stays the
  commit point; persist is temp+rename atomic (dossier precedent keeps
  file-writing policy at the owner module — selfimprove owns ledger
  persistence).

## Design

- `ImprovementLedger::from_json(&str) -> Result<Self, RecoverError>`
  (fail-closed; 3 callers updated), `persist(path)` atomic,
  `recover(dir) -> Result<(Self, Vec<RecoveryAction>), RecoverError>`.
- WAL: `begin_deployment` (pending.json = the entry) →
  `commit_deployment` (dedup-aware append + persist + clear pending);
  recovery classifies: corrupt main → CorruptLedger; stale pending →
  Replayable (well-formed) / Discarded-loudly (malformed); stale temp →
  noted.
- `check_deployment_guardrails(&Deployment, &GuardrailIndicators) ->
  Result<(), CanaryViolation>`; demo regression path routed through it
  (champion_reuse tweak).

## Acceptance Criteria

1. `cargo test --test crash_recovery` (new) green: corrupt main → Err
   (never empty); atomic persist notes stale tmp; both WAL windows
   reconcile exactly-once (no duplicate entries); canary breach names
   the indicator and drives rollback.
2. `from_json` Result callers updated; `champion_reuse` 3/3 still green
   with the canary wiring; existing suites untouched-green.
3. Gates green (`make ci` + `make doc-check`); allowlist +4 + reseal.
4. Exit assessment 3.5b updated: crash half PASS (fresh receipts);
   live-daemon half stays NOT-RUN (wording honest).

## Out of Scope

Continuous polling daemon, trigger-record typing (next assessment
gap), commit (rule 5).

## Further Notes

Grill: `.scratch/t063-crash-reconciliation-demo/grill.md`.
