# AE-01/S3 behavior red (nano step 1)

- Test: s3_recovery_cancel_intent_race_never_requeues_dispatched_work
- File: crates/hephaestus/tests/architecture_concurrent_execution.rs
- Command: CARGO_BUILD_JOBS=2 cargo test -p hephaestus --test architecture_concurrent_execution s3_recovery_cancel_intent_race_never_requeues_dispatched_work
- Result: FAIL; running 1 test; test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out.
- Actual failing assertion (verbatim from log): cancelled in-flight work must never be requeued; requeue=["OP-0-race"] cancelled=[] unresolved=[]
- Interpretation: the test compiled, reopened the durable ledger, replayed it, and failed its observable assertion. planned -> dispatched -> cancel_requested folds to Ambiguous (correct), but recover() requeues it because the retryable posture outranks durable cancel intent. A requeue would make a new reservation for cancelled work (MASTER_SPEC:375) and would never reconcile the in-flight effect (R-056/R-057). This is the missing behavior, not a compile failure or empty run.
- Requirement IDs: R-055, R-056, R-057 (ticket Covers 1, 2, 3; spec AC3).
- Seam setup: none. The test uses only existing public contracts hephaestus::operations::{OperationRecorder, recover, RetryPosture} and OperationView::state_of. No new declaration, no stub.
- Evaluation matrix: .scratch/architecture-efficiency/receipts/ae01-s3-evaluation-matrix.md (recorded before this step).
- Full log: .scratch/architecture-efficiency/receipts/ae01-s3-red-behavior.log
- Status: no commit at this nano step. Next: implement cancel-intent precedence in crates/hephaestus/src/operations/recover.rs, rerun to green, then the remaining S3 evaluations E2-E6, gates, and the single S3 implementation commit.
