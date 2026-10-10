# AE-01/S3 evaluation matrix (recorded BEFORE implementation)

Date: 2026-10-06 NZDT (host clock 2026-10-05 UTC). Task: AE-01 S3 Preserve cancellation and recovery.
Verify: `cargo test -p hephaestus --test architecture_concurrent_execution s3_recovery`

Basis: simulated-expert / trajectory-tracing eval-driven development.
Each row is a concrete behavioral evaluation over actual public contracts.
Scenarios are MiMo-designed proposals. They are not independent scientific
qualification.

Seams used (public only): `hephaestus::operations::{OperationRecorder,
recover, apply, RetryPosture, OperationView}`, `hephaestus::budget::BudgetLedger`,
`hephaestus::scheduler::{Scheduler, cancel_handle, TaskExecutor}`.

| ID | Scenario | Req IDs | Expected observable outcome | Forbidden transition | Evidence |
|----|----------|---------|------------------------------|----------------------|----------|
| E1 | Crash replay of planned -> dispatched -> cancel_requested, recover with retryable posture (max_attempts 3) | R-055 R-056 R-057 | op lands in `plan.unresolved` (reconcile in-flight effect) | op in `plan.requeue` (new reservation for cancelled work, MASTER_SPEC:375); op in `plan.cancelled` (uncertain effect hidden) | test `s3_recovery_cancel_intent_race_never_requeues_dispatched_work`, red log `ae01-s3-red-behavior.log` |
| E2 | planned -> cancel_requested, no dispatch, crash, recover | R-057 | op in `plan.cancelled`; `apply` records nothing; budget reserved 0, unresolved_count 0 | op requeued (new reservation); op unresolved (no effect ever ran) | test `s3_recovery_cancel_before_dispatch_never_touches_budget` |
| E3 | Duplicate completion: dispatched + two identical AmbiguousEffect receipts, crash, recover, apply twice | R-056 | state resolves once; first `apply` records op once; second `apply` returns empty; unresolved_count stays 1 | second charge (unresolved_count > 1); op requeued (blind retry of ambiguous effect) | test `s3_recovery_duplicate_completion_records_one_unresolved_cost` |
| E4 | Crash replay of a mixed ledger (planned, succeeded, dispatched-without-receipt, cancel-before-dispatch) | R-051 R-057 | every op classified exactly once (`plan.total() == 4`); recomputing recover on the same view yields identical buckets (stable recorded replay) | op counted twice; unknown bucket; unstable classification across replays | test `s3_recovery_crash_replay_classifies_each_operation_once` |
| E5 | Unresolved cost with stale pre-crash hold, apply, then apply again | R-055 R-056 | stale hold released; known cost recorded once as unresolved; re-apply records nothing | hold survives (`reserved != 0`); duplicate effect (`unresolved_count > 1` after re-apply) | test `s3_recovery_unresolved_cost_applies_once_over_stale_hold` |
| E6 | Scheduler cancellation race: PRE cancelled before dispatch, LONG cancelled while in flight (flag flipped by helper thread after executor start) | R-055 R-057 | both in `report.cancelled()`; PRE never reaches executor and never reserves; budget reserved 0, spent 0, unresolved_count 0 (each hold settles once) | PRE dispatched (signal only, no settle); double settle (`spent != 0` or `reserved != 0`); task silent in report | test `s3_recovery_cancel_race_settles_each_hold_exactly_once` |

Red rule: E1 is the designated first red. It must fail on an observable
assertion against current behavior (compiles, runs, fails). Compile failure,
zero executed tests, or missing API never counts as red.

Status: recorded before any implementation edit. Full logs go to
`.scratch/architecture-efficiency/receipts/ae01-s3-*.log`.
