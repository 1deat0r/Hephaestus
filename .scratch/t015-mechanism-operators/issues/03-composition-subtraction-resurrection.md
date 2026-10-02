# 03: Composition + subtraction + failure resurrection (registry complete)

**What to build:** Three operators completing the six. (a) Composition:
checks interface compatibility, units, resource budgets, interactions;
mismatched parts → rejected applicability. (b) Subtraction: output states
exactly one of function-disappears / moves-elsewhere / never-necessary.
(c) Failure resurrection: requires the recorded earlier failure plus
evidence that its boundary conditions changed; unchanged conditions →
rejected (MASTER_SPEC:162, §7 reactivation). Registry sweep then runs all
six deterministically.

**Blocked by:** 01 (MechanismRecord + registry seam).

**Status:** done

- [x] Composition: compatible parts → record; unit/interface mismatch → rejection
- [x] Subtraction record states exactly one three-way outcome (spec AC 6)
- [x] Failure resurrection without changed boundary conditions → rejected (spec AC 7)
- [x] `apply_all` sweep exercises all six operators deterministically (spec AC 9)

## Comments

## Comments
Done 2026-10-01T06:40Z: 7 new tests, 18/18. Sweep test caught a REAL provenance bug: stamp() overwrote earlier operators' attributions (last-operator-wins) — fixed to first-stamp-wins. apply_all routes candidates by input-contract prefix.
