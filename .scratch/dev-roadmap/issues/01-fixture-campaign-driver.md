# Fixture campaign driver — 20 missions, variance from actual runs

**Size:** micro task(s)

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

**Verify:** cargo test --test fixture_campaign

**Covers:** 1

**Micro-steps:**

1. M1: `campaign::run_fixture_batch(trace_variants) -> CampaignReport` seam + red test (deny-first: empty batch refuses) -> commit+push
2. M2: 20-mission loop over fixture variants through missionrun; R-103 denominators (failed/blocked retained) + budget receipts in the report -> commit+push
3. N: allowlist + reseal + ticket status flip with Verify green -> commit+push

## Comments

Decomposed 2026-10-02 from the development roadmap (user request: micro/nano tasks).
