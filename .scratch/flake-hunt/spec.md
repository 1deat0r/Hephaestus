# flake-hunt spec — sandbox CI flake characterization

Status: ready-for-agent

## Problem Statement

One CI failure in sandbox_deny passed alone and on retry.
Cause stays unestablished. Gaps-register row 10 needs data:
repeated runs solo and under recorded load, with a verdict
limited to what the data supports.

## Acceptance Criteria

1. At least five solo runs recorded with pass counts.
2. At least three loaded runs recorded with load averages.
3. The note states a verdict bounded by the data: correlated,
   uncorrelated, or inconclusive.
4. Raw outputs persist in a receipt log.

## Notes

Measurement only; no source changes expected. A fix lands
only under a separate ticket with its own failing test.
