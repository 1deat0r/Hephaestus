# 01: Typed task DAG with fail-closed validation

**What to build:** A typed DAG whose `validate()` refuses — with named
offenders — cycles, dangling dependencies, reads no dependency output can
satisfy, unordered concurrent writers, and malformed declarations, so an
unschedulable plan dies before any budget moves.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] `TaskDag` nodes carry id, priority class, dependencies, inputs/outputs
      (read/write sets), retry policy, timeout, declared cost, trivial flag,
      retryable/idempotency flag
- [x] `validate()` denies: cycles (error names the members), missing
      dependencies, read not covered by transitive-dependency outputs or
      initial inputs, write-write conflict without an ordering path,
      `max_attempts < 1`, non-positive timeout, negative cost
- [x] Validation is pure: a refused DAG is unchanged and no scheduler state
      exists yet
- [x] Valid diamond/linear DAGs validate clean; in-module unit tests cover
      topo/ordering internals; deny-first integration tests in
      `tests/scheduler_deny.rs` citing plan T-009
- [x] Zero new dependencies
## Comments

2026-09-30T21:38:22Z — Done; ACs ticked after review fix cycle 1 (mixed-wave reservation-pairing bug fixed; COST_MISMATCH, budget-refusal-zero-dispatch, and attempt-visibility tests added; Settlement errors loud, never swallowed).

Closed 2026-10-02 — work landed earlier; verified green: typed DAG and bounded scheduler covered by the green suite and the M1 CLI fixture proof.
