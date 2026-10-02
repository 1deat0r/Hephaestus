# T-031 spec - advanced search evaluation, matched ablations

Status: ready-for-agent

## Problem Statement

Advanced search mechanisms (graph retrieval, vector search, GPU workers,
MCTS, quality-diversity, learned allocation) may only be promoted when
MATCHED ablations demonstrate value within quality guardrails — evaluated
separately, disablable, with the core path functional. Infrastructure for
most is absent; honest NotEvaluable outcomes are required, not fabricated
results.

## Requirements trace

- Campaign ablation rule: remove exactly the claimed causal ingredient.
- Campaign exit: each optional feature can be disabled; the core path
  stays functional; more parallelism is not automatically better.
- R-074 continuity: matched budgets.

## Acceptance Criteria

1. Six mechanism kinds typed; each independently evaluable.
2. AblationPair enforces identical envelopes (else refused).
3. Promotion verdict: Promoted (value + guardrails + disablable) /
   Retained / NotEvaluable.
4. Non-disablable mechanisms refused promotion.
5. Twin-run byte-identical.

## Risks

- Fabricating measurements: values recorded, never generated.
- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
