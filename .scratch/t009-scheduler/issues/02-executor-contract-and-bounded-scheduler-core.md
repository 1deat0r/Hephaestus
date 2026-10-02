# 02: Executor capability contract and bounded scheduler core

**What to build:** The scheduler that drains ready tasks in priority-class
order under `max_in_flight`, resource-unit capacity, and exclusive-task
exclusivity — reserving budget *before* every priced dispatch, committing
exactly on success, releasing on failure — behind a `TaskExecutor`
capability contract a future sandboxed worker can plug into.

**Blocked by:** 01 (typed task DAG).

**Status:** done

- [x] `TaskExecutor` trait (`run`, `run_batch` with per-task default,
      `signal_cancel`) with typed outcomes
      (`Succeeded{cost}|Failed{reason}|TimedOut|Cancelled|AmbiguousEffect`)
      — the capability-contract seam (AGENTS.md)
- [x] Ready-set respects dependency completion; drain order = priority
      class then declaration order; in-flight never exceeds
      `max_in_flight`, remaining resource capacity, or exclusivity (a
      running exclusive task admits nothing else, and an exclusive task
      dispatches only when nothing else runs)
- [x] Every priced dispatch **reserves via BudgetLedger first** — a refused
      reservation stops dispatch with zero executor calls, every hold
      settles exactly once even when batching reorders execution
      (AT-055 / R-055); success with matching cost commits exactly;
      failure and cancel release; cost mismatch fails `COST_MISMATCH`
      (tested)
- [x] `AmbiguousEffect` outcome maps to `BudgetLedger::mark_unresolved`
      (R-056) rather than any retry
- [x] Full-run determinism: equal DAGs + equal mock outcomes ⇒ equal
      status/outcome sequence; deny-first tests cite R-055/R-056/AT ids
## Comments

2026-09-30T21:38:22Z — Done; ACs ticked after review fix cycle 1 (mixed-wave reservation-pairing bug fixed; COST_MISMATCH, budget-refusal-zero-dispatch, and attempt-visibility tests added; Settlement errors loud, never swallowed).

Closed 2026-10-02 — work landed earlier; verified green: typed DAG and bounded scheduler covered by the green suite and the M1 CLI fixture proof.
