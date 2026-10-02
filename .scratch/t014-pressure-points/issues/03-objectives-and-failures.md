# 03: Conflicting objectives + failure patterns (R-021 in the operator set)

**What to build:** Two operators. (a) Conflicting objectives: detects two
objectives/metrics moving against each other, but a trade-off opportunity
requires evidence that the variables are coupled in the relevant design
regime (MASTER_SPEC:144) — bare counter-movement without coupling evidence
is rejected with reason. (b) Failure patterns: recurring same-kind failures
across trace events surface as one opportunity with the pattern's evidence
(the distinct failing records cited), not one opportunity per failure.
A single isolated failure is not a pattern (rejected with reason).

**Blocked by:** 01 (seam + types).

**Status:** done

- [x] Coupled counter-movement (coupling evidence present) → trade-off opportunity citing both spans
- [x] Counter-movement without coupling evidence → rejected with reason (spec AC 4)
- [x] Recurring same-kind failures → one pattern opportunity citing the distinct failing records
- [x] Single isolated failure → rejected with reason (not a pattern)

## Comments
Done 2026-10-01T06:21Z: 4 new tests, 13/13. Fixture bug fixed honestly en route: the original fixture had latency RISING with throughput (co-movement, not counter-movement) — replaced with throughput↑/coverage↓. Counter-movement = opposite trends on matched series; coupling = explicit `coupling` record with evidence. Two-occurrence cases: below pattern threshold, no rejection row (documented).
