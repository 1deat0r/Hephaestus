# 02: Discrimination matrix, alternatives, controls

**Status:** done

**What to build:** `discrimination_matrix(plan)` rows (mechanism /
strongest alternative / artifact-null predictions), `cannot_discriminate`
flag on indistinguishable predictions, named alternative explanations
(§13:256), control structure (positive/negative/randomization/repeatability/
environment/missing-observations, §13:259) with `invalidates_inference`
marker.

**Acceptance:**
- [x] Matrix rows cover all three prediction targets (spec AC 4)
- [x] Indistinguishable predictions flagged (spec AC 4)
- [x] Twin-run byte-identical (spec AC 7)

## Comments
