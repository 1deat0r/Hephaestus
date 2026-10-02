# T-040 spec - R-105: separated yield accounting

Status: ready-for-agent

## Problem Statement

R-105 requires separating useful positive yield, useful negative-result
yield, originality, and economics. Pilot campaigns retain failure
denominators (R-103) but no typed yield separation exists.

## Requirements trace

- R-105 (requirements.json).
- R-032 continuity: no single universal score.
- R-103 continuity: complete denominators, failures retained.

## Acceptance Criteria

1. Four quantities computed separately: positive yield, useful
   negative-result yield, originality count, cost-per-useful-outcome.
2. Negative results counted as useful when they inform (typed).
3. Empty useful set -> cost value 0 (no fabricated denominator).
4. Twin-run byte-identical.

## Risks

- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
