# T-064 spec - typed triggers, bounded cycles, continuous canary (M3 3.5c)

Status: ready-for-agent

## Problem Statement

Assessment clause 3.5c is the last NOT-RUN before an honest M3 claim:
R-115 demands recorded triggers, bounded cycles, and a
no-justified-change outcome; R-118 demands a monitor that stops a
breached rollout. None exist (grep: zero trigger/cycle/deadline
concepts); the t063 check is single-shot and the trigger concept is
absent entirely.

## Requirements trace

- R-115 (triggers recorded; bounded proposal or recorded
  no-justified-change; cycle caps/budget/deadline/stop rules),
  R-118 (monitor indicators; breach stops rollout; restore verified
  incumbent). Source: docs/SELF_IMPROVEMENT.md.
- Continuity: propose_from_observations is the ONLY proposal path;
  check_deployment_guardrails + rollback reused; ledger gains a
  `triggers` vec (`serde(default)` — pre-trigger JSON stays valid,
  entries stay strict).

## Design

- record.rs: `TriggerKind`, `TriggerRecord`, `CyclePolicy`,
  `CycleOutcome::{Proposed, NoJustifiedChange}`, `CycleError`,
  `MonitorPolicy`, `MonitorOutcome::{Completed, Stopped}`,
  `MonitorError`.
- mod.rs: `record_trigger(&mut ledger, …)` (persisted with the
  ledger), `run_improvement_cycle(...)`,
  `monitor_deployment(...)` (bounded check loop; breach → wired check →
  rollback with caller-proven `verified_incumbent_digest`).
- `champion_reuse`: injected-regression path driven by
  `monitor_deployment` (3-observation stream) — rollback receipt from
  the monitor.
- New `tests/trigger_canary.rs` (phased receipts).

## Acceptance Criteria

1. Triggers persist on the ledger; `record_trigger` refuses an empty
   subject (typed).
2. Cycle: all five typed refusals (missing stop rule, candidate cap,
   budget cap, deadline) + `Proposed` (via propose_from_observations)
   + `NoJustifiedChange` for empty/unseeded input.
3. Monitor: all-clear stream → Completed (bound respected on
   over-long streams); breach → Stopped with real RollbackReceipt
   naming the indicator; mismatched verified incumbent → early typed
   error.
4. champion_reuse still 3/3 with the monitor-driven regression path;
   new test green; existing suites green.
5. Gates green (`make ci` + doc-check); allowlist +4 + reseal.
6. Exit assessment: 3.5c re-dispositioned from receipts (PASS or
   NOT-RUN — no middle), M3 clause table re-verified fresh, claim
   flipped ONLY if every clause passes; limitations restated.

## Out of Scope

Wall-clock daemons, interpreting stop-rule text, model-driven
proposals, commit (rule 5).

## Further Notes

Grill: `.scratch/t064-trigger-and-canary-service/grill.md`.
