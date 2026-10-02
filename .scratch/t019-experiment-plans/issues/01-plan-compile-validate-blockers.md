# 01: Plan record, compile, completeness validation, blockers

**Status:** done

**What to build:** `experiment` module: `ExperimentPlan` (all §13:255
fields), `Blocker` enum (§13:262), `compile` (R-040: analysis + stopping
rule required; freezes hypothesis version), `validate` (R-096: named
denials for endpoint/control/unit/guardrail gaps; R-094/095 evaluator
digest match).

**Acceptance:**
- [x] compile binds all fields; missing analysis/stopping rule fails (spec AC 1)
- [x] validate named denials + pass path (spec AC 2)
- [x] digest mismatch rejected (spec AC 3)
- [x] blocker enum precise (spec AC 5)

## Comments
