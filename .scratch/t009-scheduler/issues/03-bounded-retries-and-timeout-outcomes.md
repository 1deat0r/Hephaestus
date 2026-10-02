# 03: Bounded retries and timeout outcomes

**What to build:** Retry semantics as data: `Failed`/`TimedOut` outcomes
re-dispatch up to `max_attempts` with attempt counts visible — while
non-idempotent (`retryable: false`) work is attempted exactly once no
matter what the policy says, honoring MASTER_SPEC's no-blind-retry rule;
timeouts are spec-validated and executor-enforced, never scheduler-clocked.

**Blocked by:** 02 (executor contract and bounded scheduler core).

**Status:** done

- [x] `retryable: false` ⇒ exactly one executor call even when
      `max_attempts > 1` (MASTER_SPEC:371)
- [x] `retryable: true` with `max_attempts: 3` ⇒ at most 3 calls;
      `RunReport::attempts` exposes per-task dispatch counts (retries
      re-listed in dispatch_order); terminal failure after exhaustion
      releases budget and marks the node failed
- [x] Timeout declarations validated at DAG build/validate time; a
      `TimedOut` outcome follows the same retry rules as `Failed`
- [x] No clock anywhere in the scheduler (determinism retained); tests
      cite R-055 (dispatch accounting) and plan T-009 retry semantics
## Comments

2026-09-30T21:38:22Z — Done; ACs ticked after review fix cycle 1 (mixed-wave reservation-pairing bug fixed; COST_MISMATCH, budget-refusal-zero-dispatch, and attempt-visibility tests added; Settlement errors loud, never swallowed).

Closed 2026-10-02 — work landed earlier; verified green: typed DAG and bounded scheduler covered by the green suite and the M1 CLI fixture proof.
