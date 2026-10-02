# T-037 grill - R-117 negative case: leaked-holdout / confounded challengers

Goal: R-117 negative case (OBLIGATIONS:1869-1871): "Submit self-attested,
inconclusive, confounded or leaked-holdout challengers with favorable
self-ratings" -> "No challenger deploys without protected version/
payload-bound evidence of benefit and passing guardrails."

Covered already: self-attestation (self_improvement.rs:134), inconclusive
(rejection_classes_undeployed), unsupported benefit. MISSING: confounded
and leaked-holdout challenger rejection in the promote gate.

## Q1 - What is missing exactly?
**A:** promote() rejects FalseBenefit/Inconclusive/UnverifiedArtifacts/
self-attestation/budget-multiplication, but has no check for (a) a
challenger evaluated on a LEAKED holdout (holdout previously exposed to
the selection process — R-101 continuity, amendment::admit_for_challenger
exists but is not wired into promotion), or (b) a CONFOUNDED evaluation
(manifest declares confounds present). (agent-default)

## Q2 - Module placement + seam?
**A:** Extend selfimprove::record (new typed rejection variants) +
promote() gate parameters (holdout admissibility + evaluation confound
declaration). No changes to existing rejection semantics. (agent-default)

## Q3 - Leaked-holdout check?
**A:** promote() takes a `holdout_admissible: bool` derived from
amendment::admit_for_challenger (fresh OR separately qualified). False ->
PromotionRejection::LeakedHoldout. (agent-default)

## Q4 - Confounded check?
**A:** Assessment gains `confounds_present: bool`; true ->
PromotionRejection::ConfoundedEvaluation (a favorable self-rating on a
confounded evaluation must not deploy). (agent-default)

## Q5 - Vocabulary?
**A:** GLOSSARY rows FIRST: Leaked holdout, Confounded evaluation.
Decision row before edit. (agent-default)

## Q6 - TDD seams?
**A:** Red-first: two new rejection classes through promote(). (agent-default)
