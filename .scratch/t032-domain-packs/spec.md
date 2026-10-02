# T-032 spec - independently qualified domain packs

Status: ready-for-agent

## Problem Statement

New capabilities must be qualified separately rather than inheriting proof
from the software domain (M6 exit). A typed DomainPack contract is needed:
governing assumptions, units, simulator validity, equipment authorization,
human review, physical-actuation separation, and an independent
qualification gate.

## Requirements trace

- R-102 continuity: independently qualified bounded oracle.
- IMPLEMENTATION_PLAN:112: pack field list; physical actuation separate
  from ordinary code execution.
- M6 exit: new capabilities qualified separately (no inherited proof).

## Acceptance Criteria

1. DomainPack carries all required fields.
2. `qualify` refuses self-qualification; requires verifier identity +
   qualified oracle + scope-limited claims.
3. `validate_measurement` checks units against the pack's system and
   simulator-validity conditions.
4. Physical actuation typed separately; code/actuation mismatch refused.
5. Twin-run byte-identical.

## Risks

- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
