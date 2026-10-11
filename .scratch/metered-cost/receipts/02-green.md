# 02 receipt — compute in the dossier, finding resolved

Date: 2026-10-12 UTC.
Spec AC: 3, 4. No architectural departure.

## Red

Ticket Verify grep for `compute-wall-time-metered` failed:
dossier carried the unmetered finding.

## Green

Dossier regenerated from a real gated run: wall-time and
cpu-time quantities listed, metered successor finding marked
resolved, currency absence still explicit. Verify passes.

## Implementation

`release/mod.rs`: dossier threads measured arm compute into
cost entries (gap-safe) and replaces the unmetered finding
with the resolved metered successor. No gate logic changed.

## Regressions

`cli` dossier suites, `make ci` gate.

## Limitations

Compute is local wall plus process CPU, not provider cost.
Measured sections vary run to run by design.
