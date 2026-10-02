# T-015 grill — mechanism-operator registry (self-interview, rules 1–2)

Goal: mechanism-operator registry — abduction, contradiction resolution,
structural transfer, composition, subtraction, failure resurrection —
emitting structured mechanism records and rejected-applicability reasons
(IMPLEMENTATION_PLAN:54, MASTER_SPEC §8, R-022/023/024).

## Q1 — Module placement?
Evidence: §8 owner is the Hypothesis Genesis Engine; T-016 (hypothesis
compiler) depends on "the mechanism contract" (plan:62) — so the MECHANISM
RECORD itself is the contract T-016 builds on. T-014's discovery module is
the upstream (opportunities in).
**A:** New `crates/hephaestus/src/genesis/` module, pure (no I/O): record.rs
(MechanismRecord + operator I/O types), registry.rs (operator registry),
operators/*.rs. Consumes discovery Opportunities; emits mechanism records +
rejected-applicability reasons. (agent-default)

## Q2 — What is the MechanismRecord? (MASTER_SPEC:158 enumerates)
Evidence: "entities, relevant variables, causal or computational
relationships, prerequisites, expected effects, operating regime, failure
modes, and a minimal realization." AT-022 negative: "add AI"/"make adaptive"
must be refused.
**A:** All 8 §8:158 fields present; a `is_placeholder_language` gate on
construction refuses mechanism-free phrases ("use ai", "add a graph", "make
it adaptive", "make it adaptive"-variants) — the compiler-contract anchor.
Every operator output is a MechanismRecord with provenance (which operator,
from which opportunity, version). (agent-default)

## Q3 — Registry shape?
Evidence: "Operators are versioned transformations with input contracts,
applicability checks, bounded outputs, and provenance" (:160).
**A:** `OperatorRegistry` mapping stable operator names → boxed operators;
each operator carries name + version, an input contract (what
Opportunity/mechanism kinds it accepts), an applicability check returning
`Ok` or a structured `RejectedApplicability { reason, detail }`, bounded
output (max N mechanisms per call), and provenance stamping. Registry
iteration is deterministic (registration order, Vec-backed). (agent-default)

## Q4 — Six operators' contracts (plan:54 names them)?
- **Abduction** (:162): proposes explanations for an observation AND must
  enumerate ≥1 competing explanation — an abduction with a single
  explanation is rejected (reason: `no_competing_explanation`).
- **Contradiction resolution** (:162): searches decoupling through spatial
  separation, temporal separation, different control variables, or
  substituted mechanism — the candidate must name which decoupling axis and
  why it applies.
- **Structural transfer** (:162, R-023): must map relational structure, not
  vocabulary; output includes source mechanism, target entities, preserved
  relations, broken relations, boundary conditions. Incompatible-regime
  transfer is flagged with boundary-specific reasons (AT-023 negative).
- **Composition** (:162): checks interface compatibility, units, resource
  budgets, interactions; incompatible parts → rejected applicability.
- **Subtraction** (:162): tests whether a component's required function
  disappears, moves elsewhere, or was never necessary — output must state
  which of the three.
- **Failure resurrection** (:162; §7 reactivation): reactivates earlier
  failures whose boundary conditions have changed — requires the recorded
  failure + the changed condition evidence; unchanged conditions → rejected.
**A:** All six, each with the contract above tested at the seam.
(agent-default)

## Q5 — Hard filters vs plausibility (R-024)?
Evidence: hard filters restricted to justified constraints: malformed
records, contradictory requirements, type/unit errors, exceeded resource
bounds, exact duplicates, formally checked impossibility. "Plausibility
models may prioritize or flag candidates, but cannot irreversibly eliminate"
(:164). AT-024 negative: low-plausibility valid mechanism stays eligible.
**A:** The registry has ONLY justified hard filters; a `plausibility_flag`
is advisory metadata (never blocks); heuristic rejections land in a
documented rejection audit (`rejected_applicability` is retained, and an
explicit `audit_heuristic_rejections` accessor lists them). (agent-default)

## Q6 — What is the input seam?
**A:** `registry::apply(operator_name, opportunity: &Opportunity) ->
OperatorOutcome { mechanisms: Vec<MechanismRecord>, rejected_applicability:
Vec<RejectedApplicability> }`, plus `registry::apply_all(opportunity)`
deterministic sweep. The discovery Opportunity is the input; the corpus
stays out of genesis (evidence flows through the Opportunity's span
references). (agent-default)

## Q7 — Value estimate / novelty language?
Evidence: R-009 boundary; T-018 owns prior art. A zero-hit search cannot
yield global-novelty language.
**A:** No novelty claims anywhere in genesis. Mechanism records carry no
novelty field; rediscovery_hint style labeling stays in discovery/T-018.
(agent-default)

## Q8 — GLOSSARY terms (row-first)?
**A:** `Mechanism record`, `Operator registry`, `Rejected applicability` —
decision row FIRST, then append. (agent-default)

## Q9 — Acceptance tests to write?
**A:** (1) AT-022 negative: placeholder-language mechanism refused;
(2) AT-023 negative: incompatible-regime transfer flagged with
boundary-specific reasons; (3) AT-024 negative: low-plausibility valid
mechanism stays eligible (advisory flag, not filter); (4) abduction without
competing explanation rejected; (5) subtraction states the three-way
outcome; (6) failure resurrection requires changed boundary conditions;
(7) registry determinism (twin-run byte-identical); (8) bounded output.
(agent-default)

## Q10 — TDD seams?
**A:** Public seam = `genesis::registry::apply/apply_all` + record
constructors. Red-first per ticket. (agent-default)
