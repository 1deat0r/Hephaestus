# 01: MechanismRecord + registry skeleton + abduction (tracer bullet)

**What to build:** The `genesis` module with the mechanism contract —
`MechanismRecord` (all 8 MASTER_SPEC:158 fields) refusing placeholder
language at construction ("use ai", "add a graph", "make it adaptive" →
refusal naming the mechanism requirement, AT-022); `OperatorRegistry`
(versioned operators, deterministic Vec-backed iteration, bounded outputs,
provenance stamping); and the first operator: abduction — proposes
explanations for an observation and MUST enumerate ≥1 competing
explanation, else rejected applicability. A `plausibility` advisory flag
that never eliminates (AT-024 groundwork).

**Blocked by:** None (can start immediately).

**Status:** done

- [x] `MechanismRecord` with all §8:158 fields + placeholder-language refusal (AT-022)
- [x] `OperatorRegistry` with versioned operators, deterministic order, bounded output, provenance (operator name+version + source opportunity)
- [x] `apply` / `apply_all` seam, red-first; twin-run byte-identical sweep
- [x] Abduction: ≥2 explanations → record; single explanation → `no_competing_explanation` rejection
- [x] Advisory plausibility flag: low-plausibility valid mechanism STAYS in output (AT-024 negative)

## Comments
Done 2026-10-01T06:35Z: `genesis/{record,registry,mod}.rs`; 6/6 in `genesis_registry.rs`. Each valid competing explanation becomes its own candidate mechanism (bounded 4/invocation). Plausibility flags stamp the record + auditable heuristic log; nothing is eliminated.
