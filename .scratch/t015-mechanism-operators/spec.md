# T-015 spec — mechanism-operator registry

Status: ready-for-agent

## Problem Statement

M2 hypothesis generation needs mechanisms, not ideas: "Use AI," "add a
graph," or "make it adaptive" is not a mechanism (MASTER_SPEC:158). T-014
produces grounded Opportunities; nothing yet transforms them into
structured mechanism records through versioned operators with input
contracts, applicability checks, bounded outputs, and provenance
(IMPLEMENTATION_PLAN:54, MASTER_SPEC §8:160).

## Solution

A pure `genesis` module: a `MechanismRecord` carrying all eight §8:158
fields (entities, variables, relationships, prerequisites, expected
effects, operating regime, failure modes, minimal realization) with
placeholder-language refusal (AT-022), plus an `OperatorRegistry` of six
versioned operators — abduction, contradiction resolution, structural
transfer, composition, subtraction, failure resurrection — each with an
applicability check producing structured `RejectedApplicability` reasons
(plan:54). Hard filters restricted to justified constraints (R-024);
plausibility is advisory and never eliminating. Deterministic, bounded
outputs with provenance back to the source Opportunity.

## User Stories

1. As a genesis caller, I want a mechanism record that refuses
   mechanism-free placeholder language at construction, so that "add AI"
   can never pass as a mechanism (R-022/AT-022 negative).
2. As a genesis caller, I want abduction to enumerate at least one
   competing explanation, so that single-story causal claims are rejected
   at the operator boundary (MASTER_SPEC:162).
3. As a genesis caller, I want contradiction resolution to name its
   decoupling axis (spatial, temporal, control-variable, substituted
   mechanism), so that decoupling claims are checkable.
4. As a genesis caller, I want structural transfer to map relational
   structure with preserved relations, broken relations, and boundary
   conditions, so that incompatible-regime transfers are flagged with
   boundary-specific reasons (R-023/AT-023).
5. As a genesis caller, I want composition to check interface
   compatibility, units, and budgets, and subtraction to state whether the
   component's function disappears, moves, or was never necessary.
6. As a genesis caller, I want failure resurrection to require changed
   boundary conditions, so that old failures are never blindly retried
   (MASTER_SPEC:162, §7 reactivation).
7. As a readiness caller, I want low-plausibility valid mechanisms to
   remain eligible with an advisory flag, so that plausibility never
   irreversibly eliminates (R-024/AT-024 negative).

## Out of Scope

- Hypothesis compilation (T-016), search orchestration/lineage (T-017),
  prior-art/novelty (T-018). No novelty field exists on MechanismRecord.
- Numeric plausibility scores: the advisory flag is qualitative with a
  reason; no invented numbers (T-014 spec precedent).
- Learning/adaptation of operators: fixed, versioned transformations.

## Acceptance Criteria

1. Placeholder-language record construction is refused with a reason
   naming the mechanism requirement (AT-022).
2. Abduction with a single explanation → rejected applicability
   (`no_competing_explanation`); with ≥2 → mechanism record.
3. Contradiction-resolution candidate without a named decoupling axis →
   rejected; with axis → mechanism record.
4. Structural transfer across incompatible regimes → flagged with
   boundary-specific reasons, mechanism record carries broken relations +
   boundary conditions (AT-023).
5. Composition with unit/interface mismatch → rejected applicability.
6. Subtraction record states exactly one of: function-disappears /
   moves-elsewhere / never-necessary.
7. Failure resurrection without changed boundary conditions → rejected.
8. Low-plausibility valid mechanism stays in the mechanism list with an
   advisory flag; audit accessor lists heuristic rejections (AT-024).
9. Every emitted mechanism record carries operator name+version and source
   opportunity provenance; registry sweep output is twin-run
   byte-identical; per-operator output bounded.
10. `cargo test --workspace` green; clippy `-D warnings` clean; fmt clean;
    `make ci` EXIT=0.

## Success Risks

- Operators becoming plausible-sounding stubs: each contract above has a
  refusal path tested at the seam.
- Vocabulary drift: GLOSSARY rows FIRST (`Mechanism record`, `Operator
  registry`, `Rejected applicability`), decision row before edit.
