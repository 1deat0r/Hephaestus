# honesty-sweep spec — periodic drift checks

Status: ready-for-agent

## Problem Statement

Claims age. Reports, counts, and verdicts recorded weeks ago
may no longer hold. This track re-runs the sweep, the
reproduction guard, and manifest verification on schedule,
filing tickets for anything rotten.

## Acceptance Criteria

1. The ticket-status sweep reports zero stale, broken, or
   vacuous entries.
2. The reproduction guard passes against committed artifacts.
3. The frozen manifest verifies Valid against live inputs.
4. Rot found lands as new tickets, never silent edits.

## Notes

A sweep records; it fixes nothing by itself. Findings become
tickets with their own red-to-green cycles.
