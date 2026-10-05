# Fixture campaign driver — 20 missions, variance from actual runs

**Size:** task

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

**Verify:** cargo test --test fixture_campaign

**Covers:** 1

**Small tasks:** (max 8)

1. [x] **S1** — seam + red-to-green for run_fixture_batch
   **Status:** in_progress
   **Verify:** cargo test --test fixture_campaign
   **Micro-tasks:** (max 6)
   1. [ ] **M1** — red-to-green cycle at the campaign seam
      **Verify:** cargo test --test fixture_campaign
      - [x] add the campaign module signature and the failing test
      - [x] run cargo test --test fixture_campaign to confirm red
      - [x] implement the seam
      - [x] run the test to green

2. [ ] **S2** — 20-mission loop with R-103 receipts
   **Status:** ready-for-agent
   **Verify:** cargo test --test fixture_campaign
   **Micro-tasks:** (max 6)
   1. [ ] **M1** — red-to-green cycle for the loop and receipts
      **Verify:** cargo test --test fixture_campaign
      - [ ] implement the loop over fixture variants through missionrun
      - [ ] add denominator and budget receipts to the report
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
