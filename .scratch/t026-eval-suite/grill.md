# T-026 grill - evaluation suite: baselines + ablations

Goal: evaluation suite with active-retrieval model baseline, fixed
generate-review baseline, simple search baseline, relevant existing domain
methods; matched budgets and access; graph/review/decomposition/
optional-decision-model ablations (IMPLEMENTATION_PLAN:90, R-074, R-075).

## Q1 - What does this module actually DO (no model available)?
**A:** It is the CONTRACT + HARNESS layer: typed baseline definitions, a
matched-envelope enforcer (budgets/access must be identical across arms),
and an ablation registry (remove ONE claimed causal ingredient per
ablation). Actual model calls are provider integrations (behind capability
contracts per AGENTS.md) - here each arm declares its tool access +
budget; the enforcer REFUSES unmatched arms. Deterministic fixture arms
(search baseline) run fully; model arms are typed and enforced but their
execution is out of scope. (agent-default)

## Q2 - Module placement + seam?
**A:** New `crates/hephaestus/src/evalsuite/` module. Seam:
`define_baseline(arm) -> Result<ArmId, ArmError>`,
`check_matched(arms) -> Result<(), MismatchReport>`,
`ablation(of, remove) -> AblationSpec`. (agent-default)

## Q3 - The four baseline arms (roadmap)?
**A:** ActiveRetrievalModel (model + active retrieval + identical tools),
FixedGenerateReview (bounded generate/review harness), SimpleSearch
(structured search), DomainMethod (named existing domain method, e.g. the
campaign's tuned exact caching). Typed as BaselineArm enum with common
Envelope fields (compute budget, tool access list, model access). (agent-default)

## Q4 - Matched budgets and access (R-074, campaign: "same approved
compute/retrieval/experiment envelope")?
**A:** `check_matched` compares every pair of arms: budgets equal, tool
access sets equal, model access equal; any inequality is a named
MismatchReport entry (which arm, which resource). (agent-default)

## Q5 - Ablations (roadmap: graph, review, decomposition,
optional-decision-model)?
**A:** `ablation(of, ingredient)` - one ingredient per ablation (the
campaign's "removes the candidate's claimed causal ingredient"); the four
named ablation kinds typed; an ablation referencing an ingredient the arm
does not have is refused. (agent-default)

## Q6 - Separation of scores (R-075)?
**A:** ArmResult carries judge_score, rediscovery_status, novelty_status,
replication_status as FOUR separate optional fields - never collapsed into
one number. (agent-default)

## Q7 - Budget accounting?
**A:** Every arm's envelope carries explicit compute budget; the enforcer
also refuses an arm whose declared budget exceeds the shared envelope.
(agent-default)

## Q8 - Vocabulary?
**A:** GLOSSARY rows FIRST: Baseline arm, Ablation, Matched envelope.
Decision row before edit. (agent-default)

## Q9 - TDD seams?
**A:** Red-first per ticket: arms+matching (01), ablations+results (02).
(agent-default)
