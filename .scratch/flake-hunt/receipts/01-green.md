# 01 receipt — sandbox flake characterization

Date: 2026-10-12 UTC.
Spec AC: 1, 2, 3, 4. No code changed; measurement only.

## Red

Ticket Verify `test -f docs/SANDBOX_FLAKE_NOTE.md`
failed: file absent.

## Green

5 solo runs green at load 2; 6 spinner-loaded and 4
CI-contention runs green. An early draft claimed logged
correlation it did not have; review caught it. The note now
reports 15 logged green runs, two unlogged anecdotes, and an
inconclusive verdict.

## Implementation

Two measurement series with load averages, persisted logs,
one verdict note. No source fix attempted.

## Regressions

None touched; `make ci` gate runs before landing.

## Limitations

Correlation, not root cause. Fix needs its own ticket.

## Reviews

First review FAIL: note claims exceeded logs
(unlogged failures counted as evidence). Full logged series
re-run (5 solo, 6 spinner, 4 CI-contention: 15/15 green);
note rewritten to inconclusive with anecdotes labeled.
Second review PASS.
