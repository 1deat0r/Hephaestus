# T-041 spec - R-104: four guardrail classes before qualification

Status: ready-for-agent

## Problem Statement

R-104 requires scoped error, correctness, precision/power, and
noninferiority guardrails before qualification. `check_guardrails`
evaluates declared triples, but nothing requires ALL FOUR classes to be
declared before a plan qualifies.

## Requirements trace

- R-104 (requirements.json).
- R-037 continuity: declared before confirmation.
- Existing: evaluation::check_guardrails (semantics unchanged).

## Acceptance Criteria

1. Four guardrail classes typed.
2. `require_guardrails` refuses qualification when any class is missing
   (named gap per class).
3. Complete set passes; evaluation semantics unchanged.
4. Twin-run byte-identical.

## Risks

- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
