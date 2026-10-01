# T-041 grill - R-104: four guardrail classes before qualification

Goal: R-104 (amendment row T-003/T-026-T-028): "Require scoped error,
correctness, precision/power and noninferiority guardrails before
qualification."

Existing: `evaluation::check_guardrails` evaluates declared (name, value,
limit) triples — but nothing TYPES the four required guardrail classes or
gates qualification on ALL of them being present. A campaign could
qualify with only one guardrail declared.

## Q1 - What is missing exactly?
**A:** (a) the four named classes (ScopedError, Correctness,
PrecisionPower, Noninferiority) as an enum; (b) a qualification gate:
`require_guardrails(declared: &[class]) -> Result<(), GuardrailGap>` that
refuses qualification unless ALL FOUR classes have declared guardrails.
Evaluation semantics stay in check_guardrails; this is the presence gate.
(agent-default)

## Q2 - Module placement + seam?
**A:** Extend `evaluation` module: `GuardrailClass` enum +
`require_guardrails`. (agent-default)

## Q3 - Why "before qualification"?
**A:** The gate runs on the DECLARED plan (pre-registration continuity,
R-037): all four classes must be declared BEFORE results are
interpreted. (agent-default)

## Q4 - Refusals?
**A:** One named variant per missing class (GuardrailGap::Missing(class))
so the gap is actionable. (agent-default)

## Q5 - Vocabulary?
**A:** GLOSSARY rows FIRST: Guardrail class. Decision row before edit.
(agent-default)

## Q6 - TDD seams?
**A:** Red-first: presence gate. (agent-default)
