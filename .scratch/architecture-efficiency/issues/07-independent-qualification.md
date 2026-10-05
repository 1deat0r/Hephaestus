# AE-07: Independent qualification

**Status:** blocked
**Verify:** cargo test -p hephaestus --test architecture_independent_qualification
**Covers:** 19, 20, 21
**Blocked by:** 01, 02, 03, 04, 05, 06
**Requirements:** R-018, R-074, R-106, R-107, R-111

**Read first:** [Pi development protocol](../pi-development.md).
**Reference tests:** `crates/hephaestus/tests/evaluator_access.rs`, `crates/hephaestus/tests/eval_suite.rs`, `crates/hephaestus/tests/acceleration_eval.rs`, `crates/hephaestus/tests/reproduction_record.rs`

## Scope

This TASK implements the acceptance criteria listed in Covers. The feature specification defines shared correctness and evidence rules.
Test names below are planned. Existing passing tests cannot substitute for these acceptance checks.

**Small tasks:**

1. [ ] **S1** Protect generation and evaluation separation
   **Status:** pending
   **Verify:** cargo test -p hephaestus --test architecture_independent_qualification s1_isolation
   Deny generator access to protected evaluation artifacts; preserve advisory-only authority and current-version qualification gates.
   **Micro-tasks:**
   1. [ ] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_independent_qualification s1_isolation
      - [ ] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_independent_qualification.rs`.
      - [ ] Implement the specified behavior in `crates/hephaestus/src/security/sealed.rs`.

2. [ ] **S2** Verify independent evidence checks
   **Status:** pending
   **Verify:** cargo test -p hephaestus --test architecture_independent_qualification s2_evidence
   Verify claims against captured spans; count shared origins once; retain disagreements and inconclusive outcomes.
   **Micro-tasks:**
   1. [ ] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_independent_qualification s2_evidence
      - [ ] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_independent_qualification.rs`.
      - [ ] Implement the specified behavior in `crates/hephaestus/src/knowledge/service.rs`.

3. [ ] **S3** Measure qualified efficiency
   **Status:** pending
   **Verify:** cargo test -p hephaestus --test architecture_independent_qualification s3_measurement
   Compare baseline and candidate at matched envelopes; record correctness, false promotion, tokens, cost, latency, throughput, and peak memory.
   **Micro-tasks:**
   1. [ ] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_independent_qualification s3_measurement
      - [ ] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_independent_qualification.rs`.
      - [ ] Implement the specified behavior in `crates/hephaestus/src/evalsuite/mod.rs`.

## Comments

Spec: [Architecture efficiency plan](../spec.md).
Record baseline and candidate receipts before marking work complete. Report unresolved cases without favorable assumptions.
