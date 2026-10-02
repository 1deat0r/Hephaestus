# Real-corpus mission — repo docs as authorized corpus

**Size:** task

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

**Verify:** cargo test --test e2e_real_corpus

**Covers:** 2

**Small tasks:** (max 8)

1. [ ] **S1** — corpus ingest test over docs/ files
   **Status:** ready-for-agent
   **Verify:** cargo test --test e2e_real_corpus
   **Micro-tasks:** (max 6)
   1. [ ] **M1** — red-to-green cycle for ingest
      **Verify:** cargo test --test e2e_real_corpus
      - [ ] write the ingest test over a fixed list of repo docs
      - [ ] run cargo test --test e2e_real_corpus to confirm red
      - [ ] implement the ingest path
      - [ ] run the test to green

2. [ ] **S2** — grounded opportunities from real bytes
   **Status:** ready-for-agent
   **Verify:** cargo test --test e2e_real_corpus
   **Micro-tasks:** (max 6)
   1. [ ] **M1** — red-to-green cycle for grounded records
      **Verify:** cargo test --test e2e_real_corpus
      - [ ] build span-cited TraceRecords from the ingested docs
      - [ ] assert one grounded opportunity plus twin-run identity
      - [ ] run the test to green, then cargo fmt --check
      - [ ] run make ticket-status

3. [ ] **S3** — close: allowlist, seal, status
   **Status:** ready-for-agent
   **Verify:** make ci && make doc-check
   **Micro-tasks:** (max 6)
   1. [ ] **M1** — run the closing gates
      **Verify:** make ci && make doc-check
      - [ ] add runtime_allowlist entries as needed
      - [ ] run make seal, then make ci and make doc-check
      - [ ] flip Status to done and tick every box


## Comments

Decomposed 2026-10-02 from the development roadmap (user request: micro/nano tasks).
