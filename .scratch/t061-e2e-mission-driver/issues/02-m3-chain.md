# 02: M3 chain e2e — plan, qualified interval, interpret, dossier + honest negative

**What to build:** methods-qualified interval computation (registered
MethodSpec + fn over the checked-in trace fixture durations,
hand-computed fixture test — R-033) → experiment::compile → interpret
→ TypedResult → Dossier (evidence_label Measured with a
fixture-data receipt) → export receipt; second scenario exporting an
honest negative (inconclusive) result. NO evaluator linkage
(evalutor_access guard): hidden verdicts stay in the evaluator suite;
this chain consumes data only (guard-driven redesign, decisions row
00:20).

**Blocked by:** 01 (M2 chain proves the upstream links first).

**Status:** done

- [x] Interval fn matches hand-computed fixtures; registry +
      interpret gates exercised
- [x] Supported-path: typed result + exported dossier receipt
- [x] Negative scenario: inconclusive/contradicted exported honestly
- [x] Gates green

## Comments

Grill: `.scratch/t061-e2e-mission-driver/grill.md`.
