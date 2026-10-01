# Spec — T-009: typed task DAG and bounded worker scheduler

Status: ready-for-agent
Goal source: derived:roadmap (perpetual-directive era; T-006/T-007/T-008 complete)
Grill record: `.scratch/t009-scheduler/grill.md` (12 questions, all self-answered)

## Problem Statement

The control plane can persist history (T-006), authorize operations
(T-007), and hold budget (T-008) — but nothing decides *what runs next*: no
typed DAG validates that a plan is schedulable before execution starts, and
no queue enforces the bounds MASTER_SPEC §19 demands (concurrency, resource
classes, exclusivity, reserve-before-dispatch, no blind retries,
cancellation that stops spending). Every one of those obligations (R-055/
R-056 with their Scheduler half) has no runtime home.

## Solution

Two composites in a new `scheduler` module:

1. **`TaskDag`** — typed nodes (id, priority class, dependencies, inputs/
   outputs read/write sets, retry policy, timeout, declared cost, trivial
   flag, idempotency/retryable flag) with a fail-closed `validate()`:
   acyclicity (cycles name their members), dependency existence,
   dependency-output coverage for every read, write-write conflicts unless
   explicitly ordered, well-formed declarations. An invalid DAG never
   enters the scheduler.
2. **Bounded scheduler over a `TaskExecutor` capability contract** —
   ready-set computation, priority-class drain order, `max_in_flight`
   bound, resource-unit capacity, exclusive-task mutual exclusion,
   **reserve-before-dispatch** through the T-008 `BudgetLedger`
   (observable as: a refused reservation stops the dispatch with zero
   executor calls, and every hold settles exactly once; commit exact on
   success, release on failure/cancel, ambiguous effects →
   `mark_unresolved` with state-independent mapping), bounded retries with non-idempotent work never
   auto-retried, executor-enforced timeouts arriving as `TimedOut`
   outcomes, cancellation that refuses new reservations, and trivial-task
   batching through `run_batch`.

## User Stories

1. As a caller, I want `validate()` to reject cyclic, dangling,
   output-starved, or conflicting DAGs with named reasons, so that
   unschedulable plans die before any budget moves.
2. As a caller, I want every read either satisfied by an initial input or
   produced by a transitive dependency, so that "dependency outputs" is
   checked, not assumed.
3. As a caller, I want concurrent writers to the same key refused unless a
   dependency orders them, so that parallelism cannot corrupt state.
4. As a scheduler, I want a capability-contract executor seam, so that
   T-010's sandboxed worker plugs in without touching scheduling logic.
5. As a scheduler, I want dispatch order = highest priority class among
   ready tasks, bounded by `max_in_flight`, resource capacity, and
   exclusivity, so that MASTER_SPEC:367's bounds hold by construction.
6. As an auditor, I want a cost reservation placed *before* every priced
   dispatch, so that AT-055's "reserve before execution" is structural.
7. As an auditor, I want the reservation committed exactly on success and
   released on failure/cancel, so that budget never leaks.
8. As an auditor, I want an executor's ambiguous-effect report mapped to
   the ledger's unresolved limb, so that MASTER_SPEC:375's
   awaiting-reconciliation state exists.
9. As an auditor, I want non-idempotent tasks attempted exactly once, so
   that MASTER_SPEC:371's no-blind-retry rule holds even when a retry
   policy would allow more.
10. As a caller, I want retries bounded by `max_attempts` with attempt
    counts visible in outcomes, so that retry semantics are data, not
    folklore.
11. As a caller, I want timeouts validated on the spec and enforced by the
    executor (the only party that owns real time), so that the scheduler
    stays deterministic and testable.
12. As an operator, I want `cancel` to drop pending work with **no new
    reservations**, signal running work, release held budget, and never
    claim more than actually stopped, so that MASTER_SPEC:375 is honest.
13. As an operator, I want exclusive tasks to run alone and resource units
    to cap concurrent work, so that declared limits are enforced.
14. As a caller, I want simultaneously-ready trivial tasks grouped into one
    `run_batch` call, so that field checks do not spawn per-task work.
15. As a reviewer, I want R-055/R-056 (and AT ids) cited in tests and
    R-057 explicitly deferred to T-011, so that traceability is honest.

## Implementation Decisions

- **Module:** `scheduler/` in the control-plane crate (MASTER_SPEC:439).
- **DAG validation** exactly as grill Q3; error types name the offending
  nodes/keys; validation is pure and side-effect free.
- **Executor contract** (grill Q4): `run`, `run_batch` (default = per-task
  loop; scheduler only batches trivial ready sets), `signal_cancel`;
  outcomes `Succeeded { cost } | Failed { reason } | TimedOut | Cancelled |
  AmbiguousEffect { reason }`.
- **Dispatch cycle:** pick ready task by class order → capacity/exclusivity
  check → reserve (if priced) → executor call → map outcome (commit exact /
  release / mark_unresolved / retry or fail). Every mutator is `&mut self`
  (in-process bound, same discipline as T-008).
- **Costs fixed at dispatch** (grill Q5): `Succeeded` cost must equal the
  declared reservation or the task fails `COST_MISMATCH` — deterministic M1
  pricing; variable-cost reconciliation deferred with T-010/T-011.
- **Timeouts** spec-validated, executor-enforced (grill Q6); **retries**
  per grill Q7 (`retryable: false` ⇒ exactly one attempt, no exceptions).
- **Cancellation** per grill Q8; **batching** per grill Q9; **priority
  classes/capacity/exclusivity** per grill Q10.
- **Dependencies:** reuses `BudgetLedger` (T-008) for money; nothing else.
  Zero new dependencies.
- **Glossary:** task DAG, scheduler, trivial batch — decision row **before**
  the edit. No ADR; no schema/requirements/generated changes.

## Testing Decisions

- Seam-level tests only: `TaskDag::validate` outcomes, scheduler run/cancel/
  status behavior through a **test mock executor** (the true external —
  mocks only there, per policy), budget totals via BudgetLedger read model.
- **Prior art:** `tests/budget_ledger.rs` (deny-first + race + invariant
  helpers), `tests/policy_engine.rs` (fixed-order determinism).
- **Bound tests cite R-055/R-056/AT-055/AT-056:** validation matrix
  (cycle/dangling/output-starved/write-conflict/bad declarations), ready-set
  + bound enforcement, reserve-before-dispatch ordering (refused
  reservation ⇒ zero executor calls; mixed-wave hold pairing), exact
  commit / release paths (incl. COST_MISMATCH), attempt visibility via
  RunReport::attempts,
  non-idempotent single-attempt, retry attempt counts, timeout outcome
  mapping, cancel semantics (no new reservations; budget released;
  ambiguous → unresolved), batching call counts, determinism of a full run.
- **Suite:** fmt, clippy `-D warnings`, `cargo test --workspace`,
  `make doc-check`, `make ci` — evidence lines to the state LOG; new files
  staged + allowlisted so manifest-check covers them (standing procedure).

## Out of Scope

- Durability/replay of scheduler state and "record before dispatch" event
  writing (T-011 owns recovery; R-057's recovery half deferred with it).
- Real sandboxed workers (T-010), variable actual-cost reconciliation,
  wall-clock backoff, provider-level fairness beyond priority classes,
  per-provider concurrency matrices, UI (R-067/068 are M4).
- Any change to schemas, `requirements.json`, generated code, or gate
  files beyond the standing allowlist/stage procedure.

## Further Notes

- Limitation: bounds are in-process (one writer) — same guarantee class as
  T-008 and MASTER_SPEC:379's local MVP.
- Limitation: "fair" means documented class order, not provable starvation
  freedom; no time-based aging exists without a clock.
- Limitation: `AmbiguousEffect` → `mark_unresolved` records the charge but
  reconciliation of the underlying external system is T-010/T-011 work —
  the scheduler never retries non-idempotent work meanwhile.
