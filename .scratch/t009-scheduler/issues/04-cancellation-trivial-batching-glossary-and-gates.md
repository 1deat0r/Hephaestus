# 04: Cancellation, trivial batching, glossary, and gate evidence

**What to build:** Honest cancellation (pending work stops with **no new
reservations**, running work is signaled and resolved, budget released,
ambiguous effects land in the unresolved limb) plus `run_batch` grouping of
simultaneously-ready trivial tasks, the domain vocabulary, and green gates.

**Blocked by:** 02 (executor contract and bounded scheduler core).

**Status:** ready-for-agent

- [x] `cancel(task_id)`: pending → `Cancelled` with zero new reservations;
      running → `signal_cancel` then resolution; all held budget for
      cancelled work released; ambiguity mapping is state-independent —
      `AmbiguousEffect` always maps to `mark_unresolved` (MASTER_SPEC:375 /
      R-056; cancel-then-ambiguity ordering is unreachable pre-dispatch and
      recorded as such)
- [x] Cancelled ancestors stop descendants from ever dispatching (cascade,
      each cancelled without reservations)
- [x] Simultaneously-ready `trivial: true` tasks issued as **one**
      `run_batch` call (mock counts calls: N trivial → 1 batch; non-trivial
      never batched)
- [x] GLOSSARY gains: task DAG, scheduler, trivial batch (decision row
      written **first**)
- [x] fmt / clippy `-D warnings` / `cargo test --workspace` /
      `make doc-check` / `make ci` green with evidence in the state LOG;
      new files staged + allowlisted so manifest-check covers them;
      limitations (in-process bounds, class-order fairness, T-011 deferral)
      recorded in spec/report
## Comments

2026-09-30T21:38:22Z — Done; ACs ticked after review fix cycle 1 (mixed-wave reservation-pairing bug fixed; COST_MISMATCH, budget-refusal-zero-dispatch, and attempt-visibility tests added; Settlement errors loud, never swallowed).
