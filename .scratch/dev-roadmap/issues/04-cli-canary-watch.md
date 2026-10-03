# CLI canary-watch command (bounded monitor loop)

**Size:** task

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

**Verify:** cargo test --test cli -- --list | grep canary_watch && cargo test --test cli canary_watch

**Covers:** 4

**Small tasks:** (max 8)

1. [ ] **S1** — fixture canary-watch command
   **Status:** ready-for-agent
   **Verify:** cargo test --test cli -- --list | grep canary_watch && cargo test --test cli canary_watch
   **Micro-tasks:** (max 6)
   1. [ ] **M1** — red-to-green cycle for the subcommand
      **Verify:** cargo test --test cli -- --list | grep canary_watch && cargo test --test cli canary_watch
      - [ ] add subcommand wiring in main.rs with usage and help
      - [ ] run monitor_deployment over a fixture stream
      - [ ] add the twin-run byte-identical cli test and run it green
      - [ ] run cargo fmt --check and make ticket-status

2. [ ] **S2** — close: status flip
   **Status:** ready-for-agent
   **Verify:** make ci && make doc-check
   **Micro-tasks:** (max 6)
   1. [ ] **M1** — run the closing gates and flip
      **Verify:** make ci && make doc-check
      - [ ] run make ci and make doc-check
      - [ ] flip Status to done and tick every box


## Comments

Decomposed 2026-10-02 from the development roadmap (user request: micro/nano tasks).

**Verify repaired 2026-10-03 (ADR-031):** the old command
`cargo test --test cli canary_watch` exited 0 with `running 0 tests`
because no test matches a name that does not exist yet — the sweep read
that as a landed label. `canary_watch` appears in no source file: the
feature is not built, so this ticket stays open. The new command must
first FIND the test (`-- --list | grep`) and then RUN it, so it is red
until the work exists and green only after it passes.
