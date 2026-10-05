# AE-05: Task context assembly

**Status:** blocked
**Verify:** cargo test -p hephaestus --test architecture_task_context_assembly
**Covers:** 13, 14, 15
**Blocked by:** 04
**Requirements:** R-017, R-018, R-106, R-107, R-108

**Read first:** [Pi development protocol](../pi-development.md).
**Reference tests:** `crates/hephaestus/tests/import_closure.rs`, `crates/hephaestus/tests/domain_verifier.rs`, `crates/hephaestus/tests/prototype_worker.rs`

## Scope

This TASK implements the acceptance criteria listed in Covers. The feature specification defines shared correctness and evidence rules.
Test names below are planned. Existing passing tests cannot substitute for these acceptance checks.

**Small tasks:**

1. [ ] **S1** Define context assembly interface
   **Status:** pending
   **Verify:** cargo test -p hephaestus --test architecture_task_context_assembly s1_interface
   Accept authorized snapshot, task, evidence requirements, and token limit; return provenance and explicit missing evidence.
   **Micro-tasks:**
   1. [ ] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_task_context_assembly s1_interface
      - [ ] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_task_context_assembly.rs`.
      - [ ] Implement the specified behavior in `crates/hephaestus/src/knowledge/record.rs`.

2. [ ] **S2** Assemble bounded evidence context
   **Status:** pending
   **Verify:** cargo test -p hephaestus --test architecture_task_context_assembly s2_assembly
   Deduplicate evidence; retain counterevidence and shared origins; preserve exact spans; refuse budgets that omit mandatory evidence.
   **Micro-tasks:**
   1. [ ] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_task_context_assembly s2_assembly
      - [ ] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_task_context_assembly.rs`.
      - [ ] Implement the specified behavior in `crates/hephaestus/src/knowledge/service.rs`.

3. [ ] **S3** Verify incremental context correctness
   **Status:** pending
   **Verify:** cargo test -p hephaestus --test architecture_task_context_assembly s3_correctness
   Compare Python dependency contexts against the protected oracle; handle cycles, deletions, renames, and unsupported imports conservatively.
   **Micro-tasks:**
   1. [ ] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_task_context_assembly s3_correctness
      - [ ] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_task_context_assembly.rs`.
      - [ ] Implement the specified behavior in `crates/hephaestus/src/knowledge/service.rs`.

## Comments

Spec: [Architecture efficiency plan](../spec.md).
Record baseline and candidate receipts before marking work complete. Report unresolved cases without favorable assumptions.
