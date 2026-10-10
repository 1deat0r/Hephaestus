# AE-01: Concurrent execution

**Status:** done
**Verify:** python3 .scratch/architecture-efficiency/workflow.py verify AE-01
**Covers:** 1, 2, 3
**Blocked by:** None (AE-02 done: 8604d25)
**Requirements:** R-051, R-055, R-056, R-057

**Read first:** [Pi development protocol](../pi-development.md).
**Reference tests:** `crates/hephaestus/tests/scheduler_run.rs`, `crates/hephaestus/tests/scheduler_deny.rs`, `crates/hephaestus/tests/operations_deny.rs`

## Scope

This TASK implements the acceptance criteria listed in Covers. The feature specification defines shared correctness and evidence rules.
Test names below are planned. Existing passing tests cannot substitute for these acceptance checks.

**Small tasks:**

1. [x] **S1** Define concurrent executor contracts
   **Status:** done
   **Verify:** cargo test -p hephaestus --test architecture_concurrent_execution s1_contract
   Define bounded submission, completion identities, cancellation, and ambiguous-effect outcomes.
   **Micro-tasks:**
   1. [x] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_concurrent_execution s1_contract
      - [x] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_concurrent_execution.rs`.
      - [x] Implement the specified behavior in `crates/hephaestus/src/scheduler/executor.rs`.

2. [x] **S2** Dispatch without wave barriers
   **Status:** done
   **Verify:** cargo test -p hephaestus --test architecture_concurrent_execution s2_dispatch
   Overlap independent tasks; unblock children after completion; retain one owner for budget and state.
   **Micro-tasks:**
   1. [x] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_concurrent_execution s2_dispatch
      - [x] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_concurrent_execution.rs`.
      - [x] Implement the specified behavior in `crates/hephaestus/src/scheduler/core.rs`.

3. [x] **S3** Preserve cancellation and recovery
   **Status:** done
   **Verify:** cargo test -p hephaestus --test architecture_concurrent_execution s3_recovery
   Test cancellation races, duplicate completions, crash replay, and unresolved costs without duplicate effects.
   **Micro-tasks:**
   1. [x] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_concurrent_execution s3_recovery
      - [x] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_concurrent_execution.rs`.
      - [x] Implement the specified behavior in `crates/hephaestus/src/operations/recover.rs`.

## Comments

Spec: [Architecture efficiency plan](../spec.md).
Record baseline and candidate receipts before marking work complete. Report unresolved cases without favorable assumptions.
