# AE-04: Incremental invalidation

**Status:** blocked
**Verify:** cargo test -p hephaestus --test architecture_incremental_invalidation
**Covers:** 10, 11, 12
**Blocked by:** 03
**Requirements:** R-015, R-017, R-107, R-108, R-111

**Read first:** [Pi development protocol](../pi-development.md).
**Reference tests:** `crates/hephaestus/tests/trust_propagation.rs`, `crates/hephaestus/tests/memory_quarantine.rs`, `crates/hephaestus/tests/evidence_lifecycle.rs`

## Scope

This TASK implements the acceptance criteria listed in Covers. The feature specification defines shared correctness and evidence rules.
Test names below are planned. Existing passing tests cannot substitute for these acceptance checks.

**Small tasks:**

1. [ ] **S1** Index artifact dependencies
   **Status:** pending
   **Verify:** cargo test -p hephaestus --test architecture_incremental_invalidation s1_index
   Build rebuildable reverse dependencies using artifact versions; handle cycles and multiple dependency paths.
   **Micro-tasks:**
   1. [ ] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_incremental_invalidation s1_index
      - [ ] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_incremental_invalidation.rs`.
      - [ ] Implement the specified behavior in `crates/hephaestus/src/trustprop/mod.rs`.

2. [ ] **S2** Invalidate affected derivatives
   **Status:** pending
   **Verify:** cargo test -p hephaestus --test architecture_incremental_invalidation s2_invalidation
   Propagate correction, quarantine, and retraction; preserve audit originals; block stale qualification while review remains pending.
   **Micro-tasks:**
   1. [ ] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_incremental_invalidation s2_invalidation
      - [ ] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_incremental_invalidation.rs`.
      - [ ] Implement the specified behavior in `crates/hephaestus/src/trustprop/mod.rs`.

3. [ ] **S3** Authorize exact cache reuse
   **Status:** pending
   **Verify:** cargo test -p hephaestus --test architecture_incremental_invalidation s3_reuse
   Bind cache keys to content, transformations, provider versions, and policy; recheck current grants and dependency validity.
   **Micro-tasks:**
   1. [ ] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_incremental_invalidation s3_reuse
      - [ ] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_incremental_invalidation.rs`.
      - [ ] Implement the specified behavior in `crates/hephaestus/src/knowledge/service.rs`.

## Comments

Spec: [Architecture efficiency plan](../spec.md).
Record baseline and candidate receipts before marking work complete. Report unresolved cases without favorable assumptions.
