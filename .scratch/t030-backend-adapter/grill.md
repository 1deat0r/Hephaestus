# T-030 grill - execution backend adapter contract (Tachyon or alternative)

Goal: inspect and integrate the ACTUAL execution protocol of a backend
(Tachyon or another), using adapter contract tests — cancellation, durable
receipts, policy boundaries, budgets, replay. Do NOT port or rewrite an
existing repository merely to meet a conceptual naming plan
(IMPLEMENTATION_PLAN:102, R-066, R-051).

## Q1 - The constraint reality (AGENTS.md)?
**A:** "Tachyon is optional until its actual interface passes contract
tests." No Tachyon implementation exists in this repo, and no backend
binary is available. Attempting to invent a "Tachyon protocol" would
fabricate an interface — FORBIDDEN. The honest deliverable: the adapter
CONTRACT (the typed boundary any backend must pass) + a LocalProcess
adapter over the EXISTING local execution path (which is real), with
contract tests over the LOCAL adapter. Tachyon integration stays blocked
pending a real protocol to inspect. (agent-default)

## Q2 - Module placement + seam?
**A:** New `crates/hephaestus/src/backend/` module. Seam:
`ExecutionBackend` trait (dispatch, cancel, receipt, replay) +
`check_contract(backend) -> ContractReport` — the SAME contract tests any
future Tachyon adapter must pass (R-066: inspect + contract-test BEFORE
claiming compatibility). (agent-default)

## Q3 - What does the contract check (roadmap list)?
**A:** (a) cancellation: a dispatched task can be cancelled and reports
Cancelled; (b) durable receipts: receipts survive "restart" (re-read from
persisted ledger); (c) policy boundaries: a dispatch outside the declared
read/write sets is refused; (d) budgets: dispatch refused when budget
exhausted; (e) replay: determinism scoped to recorded responses + declared
environments (R-051) — replay of the same receipt yields the same recorded
outputs, nothing more. (agent-default)

## Q4 - What is the LocalProcess adapter over?
**A:** Deterministic, in-process task execution (a function map, no
process spawning — sandbox execution is T-010's module). It is REAL
execution of pure functions with recorded outputs — enough to exercise
every contract clause honestly. (agent-default)

## Q5 - Durable receipts (R-057 continuity)?
**A:** Receipt = task id, output digest, recorded output bytes, budget
spent, completed_at cursor. Stored in an append-only receipt ledger;
`replay(receipt_id)` re-derives the digest from recorded bytes — mismatch
refused. (agent-default)

## Q6 - Policy boundaries?
**A:** Task declares read/write sets (T-009 continuity); the backend
refuses dispatch when the declared sets are empty or the task id was
already completed with the same digest (no duplicate side effects).
(agent-default)

## Q7 - Budgets?
**A:** Backend carries a budget counter; dispatch decrements; refusal at
zero (named BudgetExhausted). (agent-default)

## Q8 - Tachyon placeholder policy?
**A:** An explicit `tachyon_integration: Blocked` note in the module docs +
report: no protocol inspected, no compatibility claim (R-066). (agent-default)

## Q9 - Vocabulary?
**A:** GLOSSARY rows FIRST: Execution backend, Dispatch receipt, Contract
report. Decision row before edit. (agent-default)

## Q10 - TDD seams?
**A:** Red-first per ticket: trait+local adapter (01), contract checks (02).
(agent-default)
