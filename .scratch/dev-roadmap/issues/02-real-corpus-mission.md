# Real-corpus mission — repo docs as authorized corpus

**Size:** micro task(s)

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

**Verify:** cargo test --test e2e_real_corpus

**Covers:** 2

**Micro-tasks:** (each ends in one commit and one push)

1. Micro — corpus ingest test over docs/ files
   - nano: pick a fixed list of repo docs and write the ingest test
   - nano: run cargo test --test e2e_real_corpus to red, then green
   - nano: cargo fmt + ticket-status, commit and push
2. Micro — grounded opportunities from real bytes
   - nano: build span-cited TraceRecords from the ingested docs
   - nano: assert at least one grounded opportunity plus twin-run identity
   - nano: run the test to green, then commit and push
3. Micro — close: allowlist, seal, status
   - nano: runtime_allowlist entries + make seal
   - nano: make ci + make doc-check
   - nano: flip Status, tick boxes, commit and push


## Comments

Decomposed 2026-10-02 from the development roadmap (user request: micro/nano tasks).
