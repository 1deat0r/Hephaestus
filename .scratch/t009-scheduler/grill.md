# Grill — T-009: typed task DAG and bounded worker scheduler

Self-interview (Phase 2, perpetual-directive era). Frontier emptied in one
round; answers from plan + MASTER_SPEC §19 + OBLIGATIONS evidence.

Goal: *typed task DAG and bounded worker scheduler — validate acyclicity,
dependency outputs, read/write sets, retry semantics, timeouts, resource
limits, cancellation; batch trivial deterministic tasks.*

## Evidence

- Plan T-009 (depends on T-006/T-007/T-008 — all complete); MASTER_SPEC:367
  "bounded, fair, dependency-aware queue… per-provider concurrency, memory,
  CPU, GPU, network, experiment exclusivity… critical path without
  starving"; :369 operation lifecycle + "recorded before the effect
  dispatched" (durability = T-011); :371 no blind retry of non-idempotent
  effects; :373 reserve-before-execute (substrate = the T-008 BudgetLedger);
  :375 cancellation prevents new reservations, may leave
  unknown/awaiting-reconciliation; :111 scheduler owns dispatch,
  reservations, retries, recovery; R-055/056/057 = the Scheduler-enforced
  trio (R-067/068 are M4 UI — out).
- Ruled out of reach: real workers (T-010), replay/durability (T-011),
  provider fairness knobs beyond priority classes (needs real providers).

## Round 1

❓ **Q1 Scope**: in = typed `TaskDag` + validation, executor **trait**
(capability contract — AGENTS.md), bounded scheduler (ready-set, resource
classes, exclusivity, priority classes), budget-coupled dispatch
(reserve→commit/release via BudgetLedger), bounded retries, timeout
*outcomes*, cancellation, trivial-task batching. Out: durability/events
(T-011), real executors (T-010), wall-clock (see Q6), UI, backoff timing,
fairness beyond classes. *source: plan text + division of labor with
T-010/T-011.*

❓ **Q2 Location**: `crates/hephaestus/src/scheduler/` (MASTER_SPEC:439
names `crates/scheduler` for "DAG, reservations, retries, recovery") —
same one-module-per-core-interface pattern. *source: MASTER_SPEC:439.*

❓ **Q3 DAG validation rules**: (a) acyclic (cycle → error naming the
members); (b) dependencies reference existing nodes; (c) **dependency
outputs**: every declared input of a task must be an initial input or an
output produced by a transitive dependency; (d) **read/write sets**: two
tasks that write the same key and can run concurrently (no dependency path
either way) → conflict error (serialization must be explicit); (e)
declarations well-formed (attempt bounds ≥1, timeout >0, cost ≥0). All
fail-closed: a DAG that fails validation never enters the scheduler.
*source: plan sentence + deterministic resolution of "dependency outputs".*

❓ **Q4 Executor seam**: `trait TaskExecutor { fn run(&self, task) ->
TaskOutcome; fn run_batch(&self, tasks) -> Vec<TaskOutcome> }` — a
capability contract behind which T-010's sandboxed worker will plug;
scheduler only sees outcomes (`Succeeded{cost?}`, `Failed{reason}`,
`TimedOut`, `Cancelled`). Mock executor lives in tests (policy: mock only
true externals — executor IS the external). *source: AGENTS.md capability-
contract rule + AT-055's mock-executor precedent in the plan (T-010).*

❓ **Q5 Budget coupling**: a task with declared `cost` is **reserved before
dispatch** (AT-055), `commit`d on success (actual ≤ declared, else excess
released... actual>declared → commit refused by T-008 → scheduler
re-reserves delta? No — scheduler commits actual only if ≤ reserved, else
releases the reservation and commits... T-008 forbids commit>reserved:
policy: scheduler commits the reserved amount and records nothing beyond —
no: honest policy = **executor's reported actual cost wins**: if actual ≤
reserved → commit(actual); if actual > reserved → commit(reserved) is
wrong... choose: release-then-mark_unresolved for the delta? Too clever.
**Decision: T-009 tasks carry a fixed declared cost; actual must equal
declared or the outcome is `Failed{COST_MISMATCH}`** (deterministic M1
executors price exactly); excess/shortfall reconciliation is T-010/T-011
territory. Failure/cancel → `release`. *source: T-008 API + determinism.*

❓ **Q6 Timeouts without a clock**: timeouts are **validated** on the spec
(>0, bounded) and **enforced by the executor** (it owns real time, returns
`TimedOut`); the scheduler treats `TimedOut` per retry policy. Clock never
enters the scheduler → tests stay deterministic (same discipline as
policy/budget). *source: determinism precedent + division of labor.*

❓ **Q7 Retry semantics**: per-task `RetryPolicy { max_attempts ≥1 }`;
retries happen only for `Failed`/`TimedOut`; **non-idempotent tasks are
never retried automatically** (`retryable: false` on the task → single
attempt, per MASTER_SPEC:371 no-blind-retry); attempt counter visible in
the outcome. No backoff (no clock) — immediate retry, documented.
*source: MASTER_SPEC:371 + plan "retry semantics".*

❓ **Q8 Cancellation semantics**: `cancel(task_id)` → (1) pending tasks
drop to `Cancelled` **without new reservations** (:375); (2) running tasks
get `executor.signal_cancel` (trait method) and resolve to `Cancelled` or
their real outcome; (3) budget: reservations on cancelled work are
`release`d; an executor-reported ambiguous effect maps to BudgetLedger
`mark_unresolved` (the awaiting-reconciliation state :375 promises — this
is where R-056 races meet the ledger). *source: MASTER_SPEC:375 +
T-008's unresolved limb.*

❓ **Q9 Batching**: tasks marked `trivial: bool` (deterministic by
definition of trivial) that are simultaneously ready are grouped into one
`run_batch` call; batch outcomes map 1:1. Non-trivial tasks dispatch
individually. Evidence: a mock executor counts calls — 5 trivial ready
tasks → 1 `run_batch`, never 5 `run`. *source: plan sentence.*

❓ **Q10 Fairness/priority**: priority classes on tasks (`critical`,
`normal`, `verification`, `exploration` — MASTER_SPEC:367's four hungry
classes); scheduler drains highest class first among ready tasks; bound =
total in-flight ≤ `max_in_flight` plus resource-unit capacity and
exclusive-task mutual exclusion (an exclusive task runs alone). No
starvation claim beyond class order (documented limitation).
*source: MASTER_SPEC:367.*

❓ **Q11 Seam/tests**: public seam = `TaskDag::{new,add,validate}` +
`scheduler::{enqueue, step/run, cancel, status}` + `TaskExecutor` trait.
Deny-first integration tests (`tests/scheduler_deny.rs` + `tests/scheduler_run.rs`),
unit tests for topo/ready-set internals; zero new deps (std only;
BudgetLedger reused). *source: repo convention + policy seams row.*

❓ **Q12 Docs**: GLOSSARY +3 (task DAG, scheduler, trivial batch) with
decision row FIRST; R-055/R-056 cited (R-057 recovery deferred T-011 —
stated); no ADR (plan execution); no schema/requirements edits.
*source: domain.md rule + OBLIGATIONS.*

## Frontier status

Empty. No refusal-category item; nothing deferred without a stated owner.
