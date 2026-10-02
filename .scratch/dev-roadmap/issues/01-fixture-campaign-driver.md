# Fixture campaign driver — 20 missions, variance from actual runs

**Size:** micro task(s)

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

**Verify:** cargo test --test fixture_campaign

**Covers:** 1

**Micro-tasks:** (each ends in one commit and one push)

1. Micro — seam + red test for run_fixture_batch
   - nano: add the campaign module signature and the failing empty-batch test
   - nano: run cargo test --test fixture_campaign until red is confirmed
   - nano: implement the seam, run the test to green
   - nano: cargo fmt + ticket-status, commit and push
2. Micro — 20-mission loop with R-103 receipts
   - nano: implement the loop over fixture variants through missionrun
   - nano: add denominator and budget receipts to the report
   - nano: run the test to green, cargo fmt, ticket-status
   - nano: commit and push, poll CI
3. Micro — close: allowlist, seal, status
   - nano: add runtime_allowlist entries and make seal
   - nano: run make ci + make doc-check
   - nano: flip Status, tick boxes, commit and push


## Comments

Decomposed 2026-10-02 from the development roadmap (user request: micro/nano tasks).
