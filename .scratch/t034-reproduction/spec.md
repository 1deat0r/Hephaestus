# T-034 spec - independent reproduction record

Status: ready-for-agent

## Problem Statement

The first candidate whose novelty and utility claims warrant it needs
external or genuinely independent reproduction — with Inconclusive and
Refuted retained as valid outcomes and no fabricated external confirmation.
No external reproducer exists yet; the honest deliverable is the typed
reproduction contract.

## Requirements trace

- IMPLEMENTATION_PLAN:114: first warranting candidate; inconclusive or
  negative outcome possible.
- R-037 continuity: immutable experiment endpoints registered before
  confirmation.
- M6 exit: no inherited proof; independence enforced by identity.

## Acceptance Criteria

1. Warrant: novelty + utility claims + first-candidate-only.
2. Reproducer identity distinct from claimant (refused otherwise).
3. Outcomes: Confirmed / Refuted / Inconclusive / PendingExternal;
   negative and inconclusive never coerced.
4. Request binds candidate_id + original claim digest + registered
   analysis endpoint.
5. Twin-run byte-identical.

## Risks

- Fabricating external confirmation: PendingExternal until a real
  independent reproducer exists.
- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
