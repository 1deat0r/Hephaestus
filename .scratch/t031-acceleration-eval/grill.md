# T-031 grill - advanced search evaluation, matched ablations

Goal: evaluate graph-specific retrieval, vector search, GPU workers, MCTS,
quality-diversity search, learned allocation SEPARATELY; promote only when
matched ablations demonstrate value within quality guardrails
(IMPLEMENTATION_PLAN:104, campaign exit: each optional feature can be
disabled, core path stays functional).

## Q1 - Reality: no vector DB / GPU / learned models available?
**A:** Correct — and none may be fabricated. This goal is the EVALUATION
CONTRACT: each candidate acceleration is a typed CandidateMechanism with a
matched-ablation protocol (same envelope, remove exactly that mechanism)
and a promotion gate requiring measured value within quality guardrails.
Mechanisms whose infrastructure is absent are recorded NotEvaluable — an
honest outcome, not a failure and not a pass. (agent-default)

## Q2 - Module placement + seam?
**A:** New `crates/hephaestus/src/acceleration/` module. Seam:
`CandidateMechanism` records, `matched_ablation(mechanism, baseline) ->
AblationPair`, `evaluate_pair(pair, measurements) -> PromotionVerdict`.
(agent-default)

## Q3 - The six mechanism kinds?
**A:** GraphRetrieval, VectorSearch, GpuWorkers, MonteCarloTreeSearch,
QualityDiversitySearch, LearnedAllocation — one enum, each independently
evaluable/disablable. (agent-default)

## Q4 - Matched ablation protocol (campaign ablation rule)?
**A:** AblationPair = (with_mechanism, without_mechanism) — identical
envelopes (compute budget, tool access — reuse evalsuite Envelope), the
ONLY difference is the mechanism. `evaluate_pair` refuses pairs whose
envelopes differ (that would not be a matched ablation). (agent-default)

## Q5 - Promotion verdict (roadmap: "promote only when matched ablations
demonstrate value within quality guardrails")?
**A:** Promoted requires ALL of: with-mechanism measurably better than
without (improvement > 0 on the declared metric), quality guardrails
passing (declared guardrail values within limits), and the mechanism
disablable (a `disable()` path exists — core stays functional).
Otherwise Retained (incumbent) or NotEvaluable (infrastructure absent).
(agent-default)

## Q6 - Disablability (campaign exit)?
**A:** Every mechanism record carries `disablable: bool`; promotion
REFUSES non-disablable mechanisms (more parallelism is not automatically
better; the core path must stay functional). (agent-default)

## Q7 - Measurements?
**A:** `PairMeasurement { metric: String, with_value: f64,
without_value: f64, guardrail_violations: usize }` — recorded values only
(no model). (agent-default)

## Q8 - Vocabulary?
**A:** GLOSSARY rows FIRST: Candidate mechanism, Matched ablation,
Promotion verdict. Decision row before edit. (agent-default)

## Q9 - TDD seams?
**A:** Red-first per ticket: mechanisms+pairs (01), verdicts (02).
(agent-default)
