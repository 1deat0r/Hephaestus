# T-064 grill - typed triggers + bounded cycle + continuous canary (M3 3.5c)

Goal: assessment clause 3.5c — the SOLE remaining M3 blocker. R-115:
triggers are recorded (mission completion / batch / drift / repeated
failure / owner), each trigger yields a bounded proposal OR a recorded
no-justified-change, each cycle has budget/caps/deadline/stop rule.
R-118: monitor predeclared indicators; a breach STOPS the rollout and
restores the verified incumbent.

## Facts (probes)

- Zero trigger/cycle/deadline/candidate-cap concepts in src (grep).
- learning.rs/gate already cover durable learning + stale reuse (R-116
  half) — untouched.
- t063 gave: `check_deployment_guardrails` (single check, wired into
  champion_reuse) and `rollback` with a caller-proven incumbent digest.
- ImprovementLedger is entries-only; JSON is strict since t063.

## Q1 - Trigger records?
**A:** `TriggerKind` (MissionCompletion | BatchInterval | DriftMeasured
| RepeatedFailure | OwnerRequest) + `TriggerRecord { kind, subject,
detail, cycle_index }`; `record_trigger(&mut ledger, ...) -> &TriggerRecord`
appends to the ledger (a NEW `triggers` vec with `#[serde(default)]` —
pre-trigger ledgers stay valid; entries stay strict). Durability comes
from the same persist path. (agent-default)

## Q2 - Bounded cycle?
**A:** `CyclePolicy { max_candidates, reserved_budget, deadline_cycle,
stop_rules }` (stop_rules REQUIRED non-empty — declared, not
interpreted); `run_improvement_cycle(&TriggerRecord, &[ProposalInput],
&CyclePolicy) -> Result<CycleOutcome, CycleError>`:
- MissingStopRule / CandidateCapExceeded / BudgetCapExceeded /
  DeadlinePassed (cycle_index > deadline) — typed refusals;
- no candidates or none seeded → `NoJustifiedChange { reason }`
  (the recorded outcome R-115 demands);
- else → `Proposed { candidate }` via the existing
  `propose_from_observations` (no second proposal path). (agent-default)

## Q3 - Continuous (bounded) canary monitor?
**A:** `MonitorPolicy { max_checks, stop_rules, verified_incumbent_digest }`
→ `monitor_deployment(&Deployment, &MonitorPolicy, &[GuardrailIndicators])
-> Result<MonitorOutcome, MonitorError>`: checks observations in order,
bounded by max_checks (stream longer → `Completed { checks }` — the bound
IS the stop rule); first breach → the wired
`check_deployment_guardrails` → real `rollback` using the policy's
verified incumbent digest (mismatch → early `MonitorError::
UnverifiedIncumbent` — the proof is caller-supplied, not vacuous) →
`Stopped { checks, rollback: RollbackReceipt }`. No wall-clock in the
module (repo purity rule): the loop is bounded by declared policy, not
seconds — the runtime runs it per deployment. (agent-default)

## Q4 - Demo wiring?
**A:** champion_reuse's injected-regression step upgrades from the
single check to `monitor_deployment` over a 3-observation stream
(clear, clear, breach) → its existing rollback then comes FROM the
monitor's Stopped receipt (reason carries the violation). The single
check fn stays (the monitor calls it). (agent-default)

## Q5 - Assessment 3.5c disposition + M3 claim?
**A:** After receipts: 3.5c → PASS if (1) triggers persist, (2) cycle
refuses unbounded cases + records no-justified-change, (3) monitor
stops a rollout with a real rollback receipt. The "continuous daemon"
wording in the doc gets REPLACED by the honest scope: a bounded
per-deployment monitor loop (what R-118's text requires) + recorded
triggers (R-115) — an out-of-process forever-daemon is not in the
obligations. THEN re-verify every M3 clause fresh in the same doc; if
all pass, M3 disposition flips to CLAIMED with the limitation lines
re-stated (label unchanged; M4 still not claimed). If any receipt is
weak, keep NOT-RUN — no claim for effort's sake. (agent-default)

## Q6 - Scope?
**A:** one new test file `tests/trigger_canary.rs` + selfimprove src +
champion_reuse wiring edit; no wall-clock/daemons; no commit (rule 5).
(agent-default)
