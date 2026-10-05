# AE-06: Qualified reasoning routing

**Status:** blocked
**Verify:** cargo test -p hephaestus --test architecture_qualified_reasoning_routing
**Covers:** 16, 17, 18
**Blocked by:** 05
**Requirements:** R-005, R-051, R-055, R-106

**Read first:** [Pi development protocol](../pi-development.md).
**Reference tests:** `crates/hephaestus/tests/advisory_provider.rs`, `crates/hephaestus/tests/backend_adapter.rs`

## Scope

This TASK implements the acceptance criteria listed in Covers. The feature specification defines shared correctness and evidence rules.
Test names below are planned. Existing passing tests cannot substitute for these acceptance checks.

**Small tasks:**

1. [ ] **S1** Route deterministic operations
   **Status:** pending
   **Verify:** cargo test -p hephaestus --test architecture_qualified_reasoning_routing s1_deterministic
   Keep parsing, arithmetic, authorization, hashing, and statistical decisions deterministic; avoid model calls for these operations.
   **Micro-tasks:**
   1. [ ] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_qualified_reasoning_routing s1_deterministic
      - [ ] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_qualified_reasoning_routing.rs`.
      - [ ] Implement the specified behavior in `crates/hephaestus/src/backend/mod.rs`.

2. [ ] **S2** Select qualified providers
   **Status:** pending
   **Verify:** cargo test -p hephaestus --test architecture_qualified_reasoning_routing s2_routing
   Select the least costly eligible provider using qualified task evidence; keep optional providers behind capability contracts.
   **Micro-tasks:**
   1. [ ] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_qualified_reasoning_routing s2_routing
      - [ ] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_qualified_reasoning_routing.rs`.
      - [ ] Implement the specified behavior in `crates/hephaestus/src/advisory/mod.rs`.

3. [ ] **S3** Bound escalation and alternatives
   **Status:** pending
   **Verify:** cargo test -p hephaestus --test architecture_qualified_reasoning_routing s3_escalation
   Escalate from independent checks; enforce retry and token limits; count failures; avoid treating self-confidence as qualification.
   **Micro-tasks:**
   1. [ ] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_qualified_reasoning_routing s3_escalation
      - [ ] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_qualified_reasoning_routing.rs`.
      - [ ] Implement the specified behavior in `crates/hephaestus/src/advisory/mod.rs`.

## Comments

Spec: [Architecture efficiency plan](../spec.md).
Record baseline and candidate receipts before marking work complete. Report unresolved cases without favorable assumptions.
