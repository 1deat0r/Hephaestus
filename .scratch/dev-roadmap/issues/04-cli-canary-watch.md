# CLI canary-watch command (bounded monitor loop)

**Size:** nano task(s)

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

**Verify:** cargo test --test cli canary_watch

**Covers:** 4

**Micro-steps:**

1. M1: `fixture canary-watch --trace <FILE>` in main.rs runs monitor_deployment over a fixture stream, prints Stopped/Completed receipt; usage+help updated -> commit+push
2. N: twin-run byte-identical cli test + status flip -> commit+push

## Comments

Decomposed 2026-10-02 from the development roadmap (user request: micro/nano tasks).
