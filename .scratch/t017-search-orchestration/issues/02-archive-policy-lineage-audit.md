# 02: Diversity archive, allocation policy, lineage, rejection audit

**What to build:** Complete the orchestrator: the diversity archive
retaining every non-kept candidate with its rejection reason (R-024); the
versioned `AllocationPolicy` with provisional 55/25/15/5 defaults visible
in the outcome (§11:221); `lineage(node)` walking parents to the seed; and
`audit()` returning every rejection (reason + candidate + wave).

**Blocked by:** 01 (search graph + beam).

**Status:** done

- [x] Archive retains every non-kept candidate + reason (R-024, spec AC 3)
- [x] `AllocationPolicy` versioned, 55/25/15/5 defaults, visible (spec AC 5)
- [x] `lineage(node)` walks to seed through parents+operators (spec AC 4)
- [x] `audit()` returns all rejections with reason+candidate+wave (spec AC 6)

## Comments

## Comments
Done 2026-10-01T06:56Z: 3 new tests, 10/10. TDD deviation: archive/policy/lineage/audit implemented in ticket 01's run loop; ticket 02 tests are deep coverage (all green first run).
