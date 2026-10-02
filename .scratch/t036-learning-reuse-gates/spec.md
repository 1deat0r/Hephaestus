# T-036 spec - R-116 negative case: stale/quarantine invalidation

Status: ready-for-agent

## Problem Statement

R-116's negative case requires that after a restart following a recorded
failure/challenger decision, altering or quarantining a learning source
invalidates reuse — while holdout history survives. The selfimprove
ledger persists but has no staleness/quarantine gate.

## Requirements trace

- R-116 (OBLIGATIONS:1849-1852): stale/quarantined dependencies
  invalidate reuse; holdout history survives.
- Existing: ImprovementLedger append-only + restart reconstruction
  (tests/self_improvement.rs:184).

## Acceptance Criteria

1. Typed entry states: Valid / Stale / Quarantined; state changes are
   append-only records (ledger untouched).
2. Reuse gate: only Valid entries with matching current source digest
   are usable; altered source -> Stale; quarantined -> excluded.
3. History survives: entries + fresh_partition_id remain visible after
   invalidation.
4. Twin-run byte-identical.

## Risks

- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
