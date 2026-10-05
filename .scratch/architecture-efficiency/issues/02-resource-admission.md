# AE-02: Resource admission

**Status:** done
**Verify:** cargo test -p hephaestus --test architecture_resource_admission
**Covers:** 4, 5, 6
**Blocked by:** None
**Requirements:** R-055, R-056, R-110, R-111

**Read first:** [Pi development protocol](../pi-development.md).
**Reference tests:** `crates/hephaestus/tests/budget_ledger.rs`, `crates/hephaestus/tests/scheduler_deny.rs`, `crates/hephaestus/tests/sandbox_deny.rs`

## Scope

This TASK implements the acceptance criteria listed in Covers. The feature specification defines shared correctness and evidence rules.
Test names below are planned. Existing passing tests cannot substitute for these acceptance checks.

**Small tasks:**

1. [x] **S1** Define resource reservations
   **Status:** done
   **Verify:** cargo test -p hephaestus --test architecture_resource_admission s1_envelope
   Declare memory, CPU, token, and cost envelopes; refuse oversized or invalid requests before dispatch.
   **Micro-tasks:**
   1. [x] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_resource_admission s1_envelope
      - [x] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_resource_admission.rs`.
      - [x] Implement the specified behavior in `crates/hephaestus/src/scheduler/dag.rs`.

2. [x] **S2** Bound admission and queues
   **Status:** done
   **Verify:** cargo test -p hephaestus --test architecture_resource_admission s2_admission
   Reserve capacity for active work and queued bytes; release reservations exactly once on terminal outcomes.
   **Micro-tasks:**
   1. [x] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_resource_admission s2_admission
      - [x] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_resource_admission.rs`.
      - [x] Implement the specified behavior in `crates/hephaestus/src/scheduler/core.rs`.

3. [x] **S3** Enforce measured memory limits
   **Status:** done
   **Verify:** cargo test -p hephaestus --test architecture_resource_admission s3_limits
   Measure worker peaks; enforce hard limits; include buffers and resident workers; preserve receipts on violations.
   **Micro-tasks:**
   1. [x] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_resource_admission s3_limits
      - [x] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_resource_admission.rs`.
      - [x] Implement the specified behavior in `crates/hephaestus/src/sandbox/mod.rs`.

## Comments

Spec: [Architecture efficiency plan](../spec.md).
Record baseline and candidate receipts before marking work complete. Report unresolved cases without favorable assumptions.
