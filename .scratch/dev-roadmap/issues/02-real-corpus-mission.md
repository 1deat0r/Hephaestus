# Real-corpus mission — repo docs as authorized corpus

**Size:** task

**Blocked by:** None (can start immediately)

**Status:** done

**Verify:** cargo test --test e2e_real_corpus

**Covers:** 2

**Small tasks:** (max 8)

1. [x] **S1** — corpus ingest test over docs/ files
   **Status:** done
   **Verify:** cargo test --test e2e_real_corpus
   **Micro-tasks:** (max 6)
   1. [x] **M1** — red-to-green cycle for ingest
      **Verify:** cargo test --test e2e_real_corpus
      - [x] write the ingest test over a fixed list of repo docs
      - [x] run cargo test --test e2e_real_corpus to confirm red
      - [x] implement the ingest path
      - [x] run the test to green

2. [x] **S2** — grounded opportunities from real bytes
   **Status:** done
   **Verify:** cargo test --test e2e_real_corpus
   **Micro-tasks:** (max 6)
   1. [x] **M1** — red-to-green cycle for grounded records
      **Verify:** cargo test --test e2e_real_corpus
      - [x] build span-cited TraceRecords from the ingested docs
      - [x] assert one grounded opportunity plus twin-run identity
      - [x] run the test to green, then cargo fmt --check
      - [x] run make ticket-status

3. [x] **S3** — close: allowlist, seal, status
   **Status:** done
   **Verify:** make ci && make doc-check
   **Micro-tasks:** (max 6)
   1. [x] **M1** — run the closing gates
      **Verify:** make ci && make doc-check
      - [x] add runtime_allowlist entries as needed
      - [x] run make seal, then make ci and make doc-check
      - [x] flip Status to done and tick every box


## Comments

Decomposed 2026-10-02 from the development roadmap (user request: micro/nano tasks).
