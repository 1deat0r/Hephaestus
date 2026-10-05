# AE-01/S2 behavior red (nano step 1)

- Test: s2_dispatch_child_runs_before_unrelated_completion
- File: crates/hephaestus/tests/architecture_concurrent_execution.rs
- Command: CARGO_BUILD_JOBS=2 cargo test -p hephaestus --test architecture_concurrent_execution s2_dispatch_child_runs_before_unrelated_completion
- Result: FAIL, exit 101; running 1 test; test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 3.00s.
- Actual failing assertion (verbatim from log): child must start while unrelated work is still active;          UNRELATED outcome reason: Some("child never started while unrelated work was active"); dispatch order: ["PARENT", "UNRELATED", "CHILD"]
- Interpretation: the test compiled, executed the current scheduler, and failed its observable assertion after the bounded 3 s deadline. The legacy wave barrier is visible: CHILD can dispatch only after the whole wave (PARENT + UNRELATED) settles, so UNRELATED never observes CHILD while active. No deadlock; failure terminates.
- Requirement IDs: R-051, R-055, R-056, R-057 (ticket Covers 1, 2, 3).
- Seam setup: none. The test uses only the existing public seam hephaestus::scheduler::{Scheduler, SchedulerConfig, TaskDag, Task, TaskExecutor, TaskOutcome} plus hephaestus::budget::BudgetLedger. No new declaration, no delegation, no stub, no production code change in this step.
- Full log: .scratch/architecture-efficiency/receipts/ae01-s2-red-behavior.log
- Status: no commit at this nano step. Next: implement streaming dispatch in crates/hephaestus/src/scheduler/core.rs, rerun to green, then gates and the single S2 implementation commit.
