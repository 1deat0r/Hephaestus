# AE-03: Compact evidence storage

**Status:** ready-for-agent
**Verify:** cargo test -p hephaestus --test architecture_compact_evidence_storage
**Covers:** 7, 8, 9
**Blocked by:** None
**Requirements:** R-014, R-015, R-017, R-107

**Read first:** [Pi development protocol](../pi-development.md).
**Reference tests:** `crates/hephaestus/tests/event_ledger.rs`, `crates/hephaestus/tests/corpus_ingest.rs`, `crates/hephaestus/tests/corpus_search.rs`

## Scope

This TASK implements the acceptance criteria listed in Covers. The feature specification defines shared correctness and evidence rules.
Test names below are planned. Existing passing tests cannot substitute for these acceptance checks.

**Small tasks:**

1. [ ] **S1** Stream ledger verification
   **Status:** pending
   **Verify:** cargo test -p hephaestus --test architecture_compact_evidence_storage s1_ledger
   Verify exact-byte chains incrementally; retain compact offsets; preserve corruption and torn-tail recovery behavior.
   **Micro-tasks:**
   1. [ ] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_compact_evidence_storage s1_ledger
      - [ ] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_compact_evidence_storage.rs`.
      - [ ] Implement the specified behavior in `crates/hephaestus/src/ledger/event_ledger.rs`.

2. [ ] **S2** Load source bytes on demand
   **Status:** pending
   **Verify:** cargo test -p hephaestus --test architecture_compact_evidence_storage s2_source
   Store immutable source bytes once; retrieve verified spans through artifact identities without copying entire corpora.
   **Micro-tasks:**
   1. [ ] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_compact_evidence_storage s2_source
      - [ ] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_compact_evidence_storage.rs`.
      - [ ] Implement the specified behavior in `crates/hephaestus/src/knowledge/service.rs`.

3. [ ] **S3** Bound storage caches
   **Status:** pending
   **Verify:** cargo test -p hephaestus --test architecture_compact_evidence_storage s3_cache
   Limit resident cache bytes; evict reloadable data; retain referenced originals; measure cold and warm memory peaks.
   **Micro-tasks:**
   1. [ ] **M1** Demonstrate the missing behavior; implement it; pass the same acceptance tests.
      **Verify:** cargo test -p hephaestus --test architecture_compact_evidence_storage s3_cache
      - [ ] Add positive, refusal, and regression cases in `crates/hephaestus/tests/architecture_compact_evidence_storage.rs`.
      - [ ] Implement the specified behavior in `crates/hephaestus/src/ledger/artifact_store.rs`.

## Comments

Spec: [Architecture efficiency plan](../spec.md).
Record baseline and candidate receipts before marking work complete. Report unresolved cases without favorable assumptions.
