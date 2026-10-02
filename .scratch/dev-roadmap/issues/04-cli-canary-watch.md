# CLI canary-watch command (bounded monitor loop)

**Size:** nano task(s)

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

**Verify:** cargo test --test cli canary_watch

**Covers:** 4

**Micro-tasks:** (each ends in one commit and one push)

1. Micro — fixture canary-watch command
   - nano: add the subcommand wiring in main.rs with usage and help lines
   - nano: run monitor_deployment over a fixture stream, print the receipt
   - nano: add the twin-run byte-identical cli test, run it green
   - nano: cargo fmt + ticket-status, commit and push
2. Micro — close: status flip
   - nano: make ci + make doc-check
   - nano: flip Status, tick boxes, commit and push


## Comments

Decomposed 2026-10-02 from the development roadmap (user request: micro/nano tasks).
