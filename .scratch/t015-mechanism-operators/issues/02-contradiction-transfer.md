# 02: Contradiction resolution + structural transfer (R-023)

**What to build:** Two operators. (a) Contradiction resolution: a candidate
must name its decoupling axis — spatial separation, temporal separation,
different control variables, or substituted mechanism (MASTER_SPEC:162) —
else rejected applicability. (b) Structural transfer: maps RELATIONAL
structure, not vocabulary; output carries source mechanism, target
entities, preserved relations, broken relations, and boundary conditions;
transfer across incompatible regimes is flagged with boundary-specific
reasons (R-023/AT-023 negative).

**Blocked by:** 01 (MechanismRecord + registry seam).

**Status:** done

- [x] Contradiction resolution: named decoupling axis → record; missing axis → rejection
- [x] Structural transfer: preserved + broken relations + boundary conditions on the record; relation-structure map present (not vocabulary-only)
- [x] Incompatible-regime transfer → boundary-specific rejection reasons (AT-023)

## Comments

## Comments
Done 2026-10-01T06:36Z: 5 new tests, 11/11. Axis syntax: `axis=` segment; transfer syntax: `target=`, `relations=` (relational structure required: `->` or `keyed-by`), `broken=`, `boundaries=`, `regime match`/`regime mismatch`. Cross-regime emits flagged (boundary-limited); mismatch with unanalyzed broken-relations rejected.
