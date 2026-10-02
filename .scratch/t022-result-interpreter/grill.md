# T-022 grill - independent evaluator + typed result interpreter

Goal: independent evaluation of artifacts against protected evaluators,
typed results preserving the three §14 dimensions (execution validity,
scientific conclusion, engineering target), threshold interpretation
(above/contradict/inconclusive), independent guardrail checks
(IMPLEMENTATION_PLAN:74, MASTER_SPEC §14, R-026, R-046-adjacent).

## Q1 - Module placement + seam?
**A:** New `crates/hephaestus/src/evaluation/` module. Seam:
`interpret(plan, measurements) -> TypedResult` and `check_guardrails(plan,
measurements) -> Vec<GuardrailOutcome>`. It consumes frozen ExperimentPlans
(T-019) and the method registry (T-021) — the integration point named by
the roadmap. (agent-default)

## Q2 - The three result dimensions (§14:280)?
**A:** `ExecutionValidity { Valid, Invalid, Incomplete }` — preserved, not
collapsed. `ScientificConclusion { Supported, Contradicted, Inconclusive,
NotAssessed }`. `EngineeringTarget { Met, NotMet, Inconclusive, NotAssessed
}`. TypedResult carries all three + the claim and conditions each
conclusion applies to (§14:280 "identify the claim and conditions").
(agent-default)

## Q3 - Threshold semantics (§14:281)?
**A:** Interval entirely above theta -> supports the scoped target;
entirely below -> contradicts; overlap -> Inconclusive. Mechanistic support
assessed separately (NOT inferred from the interval). Noninferiority:
lower bound of the quality difference exceeding -m, not a nonsignificant
difference. Inputs: interval (low, high) + threshold — computed by the
qualified method, carried here. (agent-default)

## Q4 - No "proven true" (§14:282)?
**A:** TypedResult cannot represent proof: no True variant exists; the
strongest scientific state is Supported (scoped). Compile-time guarantee:
the enum has no such variant; doc-comment states the invariant. (agent-default)

## Q5 - Invalid execution semantics?
**A:** Invalid or incomplete execution -> ScientificConclusion and
EngineeringTarget are NotAssessed (never interpreted from invalid data) —
this preserves valid vs invalid execution per the roadmap line.
(agent-default)

## Q6 - Independent guardrails (roadmap: "Check guardrails independently")?
**A:** Guardrail outcomes computed from the plan's declared guardrails +
measurements, SEPARATELY from the scientific/engineering dimensions; a
failed guardrail does not flip the science dimension, it records its own
outcome and blocks promotion ( promotion blocking is T-023+ scope; here
just the independent record). (agent-default)

## Q7 - Threshold provenance (R-026)?
**A:** TypedResult carries the estimand, units, comparator, and threshold
provenance (who declared it, where) copied from the plan. (agent-default)

## Q8 - What measurements shape?
**A:** `Measurements { interval: Option<(f64, f64)>, threshold: f64,
quality_margin: Option<f64>, guardrail_values: Vec<(String, f64, f64)> }`
— interval from the qualified analysis; guardrail values as (name, value,
limit). Missing interval (analysis not run) -> NotAssessed, not guessed.
(agent-default)

## Q9 - Integration checks (roadmap: T-022 integrates T-020+T-021)?
**A:** `interpret` requires the plan's evaluator digest to match the
protected digest (R-095 continuity) and the method to be registry-qualified
(R-100); otherwise Invalid execution with a named reason. (agent-default)

## Q10 - Vocabulary?
**A:** GLOSSARY rows FIRST: Typed result, Execution validity, Guardrail
outcome. Decision row before edit. (agent-default)

## Q11 - TDD seams?
**A:** Red-first per ticket at `interpret` + `check_guardrails`. (agent-default)
