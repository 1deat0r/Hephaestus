# 01: Candidate, assessment, promote, rejection classes

**Status:** done

**What to build:** `selfimprove` module: ImprovementCandidate (§R-117
fields + frozen manifest), Assessment (benefit_status, guardrail_status,
external evaluator identity), `evaluate_candidate` (self-attestation +
budget-multiplication checks), `promote` (named rejections; deployment
with retained incumbent + rollback target).

**Acceptance:**
- [x] Candidate binds all fields (spec AC 1)
- [x] Named rejections incl. self-attestation + budget multiplication (spec AC 2/5)
- [x] Success deployment retains incumbent (spec AC 3)

## Comments
