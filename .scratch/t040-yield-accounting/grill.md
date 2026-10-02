# T-040 grill - R-105: separated yield accounting

Goal: R-105 (amendment row T-003/T-026-T-028): "Separate useful positive
outcome yield, useful negative-result yield, originality and economics."
Gap: pilot campaigns compute variance/denominators (R-103) but no typed
separation of yield classes; R-105 has no tests.

## Q1 - What are the four separated quantities?
**A:** (1) useful POSITIVE yield — outcomes that advanced the mission;
(2) useful NEGATIVE-result yield — refuted/inconclusive outcomes that
still inform (a valid negative is not waste); (3) originality — count of
outcomes not derivable from prior-art (discovery continuity); (4)
economics — resource cost per useful outcome. Each computed and reported
SEPARATELY — never merged into one universal score (R-032 continuity).
(agent-default)

## Q2 - Module placement + seam?
**A:** New `crates/hephaestus/src/yieldaccount/` module. Seam:
`YieldClass` enum, `account(outcomes) -> YieldReport` over typed pilot
outcomes (OutcomeKind: UsefulPositive / UsefulNegative / Uninformative,
with cost + originality flag per outcome). (agent-default)

## Q3 - Where do outcomes come from?
**A:** Typed input records (kind, cost_minor_units, original) supplied by
the caller from pilot campaign records — computed here from RECORDED
data only, never guessed. (agent-default)

## Q4 - Economics?
**A:** cost per useful outcome = total cost / (positive + negative
useful count); empty useful set -> named None/0 (no division-by-zero
fabrication; denominators complete, R-103 continuity). (agent-default)

## Q5 - Vocabulary?
**A:** GLOSSARY rows FIRST: Useful negative result, Yield class. Decision
row before edit. (agent-default)

## Q6 - TDD seams?
**A:** Red-first: class separation + economics denominator. (agent-default)
