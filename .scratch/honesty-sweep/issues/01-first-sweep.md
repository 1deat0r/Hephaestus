# 01: First scheduled honesty sweep

**Status:** done
**Verify:** cargo test -p hephaestus --test cli reproduce_v1_committed && ./target/debug/hephaestus fixture verify-manifest --manifest docs/CAMPAIGN_MANIFEST.json --trace crates/hephaestus/tests/fixtures/e2e-trace.log --pilot docs/PILOT_CAMPAIGN_REPORT.md --baseline docs/BASELINE_COMPARISON_REPORT.md > /dev/null && python3 tools/check_ticket_status.py --format
**Covers:** 1, 2, 3
**Blocked by:** None

## Scope

Run the sweep, the repro guard, and manifest verification.
Record results. File tickets for rot found (none this round).

**Small tasks:**

1. [x] **S1** Run the sweep triple and record
   **Status:** done
   **Verify:** cargo test -p hephaestus --test cli reproduce_v1_committed && ./target/debug/hephaestus fixture verify-manifest --manifest docs/CAMPAIGN_MANIFEST.json --trace crates/hephaestus/tests/fixtures/e2e-trace.log --pilot docs/PILOT_CAMPAIGN_REPORT.md --baseline docs/BASELINE_COMPARISON_REPORT.md > /dev/null && python3 tools/check_ticket_status.py --format
   Sweep clean; guard green; manifest Valid; no rot filed.
   **Micro-tasks:**
   1. [x] **M1** Execute and record
      **Verify:** cargo test -p hephaestus --test cli reproduce_v1_committed && ./target/debug/hephaestus fixture verify-manifest --manifest docs/CAMPAIGN_MANIFEST.json --trace crates/hephaestus/tests/fixtures/e2e-trace.log --pilot docs/PILOT_CAMPAIGN_REPORT.md --baseline docs/BASELINE_COMPARISON_REPORT.md > /dev/null && python3 tools/check_ticket_status.py --format
      atomic

## Comments

Landed with the sweep evidence in the receipt. AC 4 (rot
filing) stays unexercised: no rot found, so no filing path
ran. A round that finds rot covers it.
