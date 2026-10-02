# T-028 spec - release packet

Status: ready-for-agent

## Problem Statement

The release layer needs a typed packet: scope label (EXPERIMENTAL default),
security findings with a critical-failure gate (R-077), finite-suite
uncertainty instead of universal reliability (R-078), all five outcome
classes (R-080), reproducibility report, quantity-only cost/latency
measurements, unresolved research questions.

## Requirements trace

- R-077 / AT-077: applicable release gates blocked on critical unresolved
  failures.
- R-078 / AT-078: finite-suite uncertainty, never universal reliability.
- R-080 / AT-080: all five outcome classes handled end-to-end.
- Campaign dispositions: EXPERIMENTAL vs QUALIFIED_FOR_DECLARED_SCOPE.

## Acceptance Criteria

1. Scope label defaults EXPERIMENTAL; QUALIFIED requires the five
   qualification inputs (campaign dispositions).
2. Critical unresolved in-scope failures block assembly (named block);
   out-of-scope criticals recorded but non-blocking.
3. Uncertainty struct required; no universal-reliability field exists.
4. All five outcome classes present in counts.
5. Cost/latency quantities-only; reproducibility report present.
6. Twin-run byte-identical.

## Risks

- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
