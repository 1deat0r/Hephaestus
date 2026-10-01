# T-044 spec - R-032: multi-objective archive gate

Status: ready-for-agent

## Problem Statement

R-032 (Portfolio scheduler, runtime M2) requires a diverse
multi-objective archive rather than a single universal score. AT-032:
a cheap uncertain candidate and an expensive plausible candidate can
both remain in the frontier when neither dominates. The orchestrator
(T-017) has a single-axis beam frontier; no dominance semantics exist.

## Requirements trace

- R-032 / AT-032 (OBLIGATIONS; runtime M2, contract M0).
- Continuity: R-031 bounds, R-024 no-silent-elimination, R-033 no
  fabricated probabilities.

## Design

`orchestrator::archive`: two-axis Pareto over search nodes —

- cost axis: `estimated_cost` (minimize), already on SearchNode.
- value axis: producer-declared qualitative band (Low/Medium/High),
  serialized on the node; band rank drives dominance. Missing band ->
  Low (never fabricated, never auto-eliminated as a score).
- Dominance: a dominates b iff a <= b on ALL axes and < on >= 1.
- Frontier = non-dominated set. Cheap-uncertain vs expensive-plausible
  -> mutually non-dominating -> both stay (AT-032).
- Bounded: cap; eviction order = dominated first, then LRU within the
  same corner; every eviction carries a retained reason (R-024).

## Acceptance Criteria

1. AT-032 scenario: both candidates remain in the frontier.
2. Dominated node is evictable; non-dominated never auto-evicted.
3. Cap eviction leaves an audit reason; nothing is silently dropped.
4. Twin-run byte-identical.

## Risks

- Do NOT invent numeric confidence (R-033); bands only.
