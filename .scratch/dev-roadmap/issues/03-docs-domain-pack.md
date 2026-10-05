# Docs domain pack qualified (R-006)

**Size:** task

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

**Verify:** cargo test --test docs_domain_pack

**Covers:** 3

**Small tasks:** (max 8)

1. [x] **S1** — docs DomainPack fixture + qualify green
   **Status:** done
   **Verify:** cargo test --test docs_domain_pack
   **Micro-tasks:** (max 6)
   1. [x] **M1** — red-to-green cycle for qualify
      **Verify:** cargo test --test docs_domain_pack
      - [x] build the pack fixture with an adjudicated oracle
      - [x] write the failing qualify test and run it red
      - [x] make qualify pass
      - [x] run the test to green

2. [ ] **S2** — close: gates + status
   **Status:** in_progress
   **Verify:** make ci && make doc-check
   **Micro-tasks:** (max 6)
   1. [ ] **M1** — run the closing gates
      **Verify:** make ci && make doc-check
      - [x] add runtime_allowlist entries as needed
      - [ ] run make seal, then make ci and make doc-check
      - [ ] flip Status to done and tick every box


## Comments

Decomposed 2026-10-02 from the development roadmap (user request: micro/nano tasks).
