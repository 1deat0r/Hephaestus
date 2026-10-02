# T-035 spec - amendment row: migration gates + qualified holdouts

Status: ready-for-agent

## Problem Statement

The T-032/T-033 amendment row (IMPLEMENTATION_PLAN:138) requires new-domain
validity/migration gates and fresh or separately qualified holdouts for
challenger evaluation — R-100, R-101, R-114.

## Requirements trace

- R-100: typed experiment families, selection lineage, method
  qualification, explicit error allocation.
- R-101: sealed-data access tracked per campaign; no holdout exposure
  resets.
- R-114: explicit transitions + schema/policy migration with immutable
  historical identities.

## Acceptance Criteria

1. Migration requires declared gates; receipt preserves old identity
   verbatim (immutable).
2. Undeclared transitions refused.
3. Holdout exposure is monotonic (no resets); challenger evaluation on
   fresh/separately-qualified holdouts only.
4. Family binding: lineage + method qualification + declared error
   allocation.
5. Twin-run byte-identical.

## Risks

- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
