# T-037 spec - R-117 negative case: leaked-holdout + confounded rejection

Status: ready-for-agent

## Problem Statement

The promote gate lacks two R-117 negative-case rejection classes:
leaked-holdout challengers (holdout exposed to selection — R-101
continuity) and confounded evaluations (favorable self-rating on a
confounded run must not deploy).

## Requirements trace

- R-117 negative case (OBLIGATIONS:1869-1871).
- amendment::admit_for_challenger (T-035): fresh OR separately-qualified
  holdout admissibility — wired in as a gate input.
- Existing rejections unchanged.

## Acceptance Criteria

1. `PromotionRejection::LeakedHoldout`: promotion refused when the
   evaluation holdout is not admissible.
2. `PromotionRejection::ConfoundedEvaluation`: promotion refused when
   the assessment declares confounds present.
3. Existing rejection classes unchanged (all prior tests pass).
4. Twin-run byte-identical.

## Risks

- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
