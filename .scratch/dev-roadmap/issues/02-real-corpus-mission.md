# Real-corpus mission — repo docs as authorized corpus

**Size:** micro task(s)

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

**Verify:** cargo test --test e2e_real_corpus

**Covers:** 2

**Micro-steps:**

1. M1: test ingests docs/ files via knowledge::ingest_bytes (fixed file list) + span-cited TraceRecords -> commit+push
2. M2: run discovery chain over that corpus; assert >=1 grounded opportunity from REAL bytes + twin-run identical -> commit+push
3. N: allowlist + reseal + status flip -> commit+push

## Comments

Decomposed 2026-10-02 from the development roadmap (user request: micro/nano tasks).
