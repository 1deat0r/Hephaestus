# T-026 spec - evaluation suite contract layer

Status: ready-for-agent

## Problem Statement

Comparisons against baselines (R-074) need a typed harness that enforces
matched budgets and access across arms and registers single-ingredient
ablations. Model arms are typed and enforced here; their execution needs
provider integrations (capability contracts) and stays out of scope.
Scores stay separated per R-075.

## Requirements trace

- R-074 / AT-074: baselines compared at matched budgets; active-retrieval
  and structured-search baselines present.
- R-075 / AT-075: judge scores, rediscovery, prospective novelty,
  independent replication kept SEPARATE.
- Campaign: all arms get the same approved envelope; ablation removes the
  claimed causal ingredient; tuning/engineering time recorded.

## Acceptance Criteria

1. Four typed baseline arms with per-arm Envelope (compute budget, tool
   access, model access, tuning record).
2. `check_matched` refuses any inequality — named per arm + resource.
3. Ablations: single-ingredient removal; four named kinds; ingredient-not-
   present refused.
4. ArmResult keeps the four R-075 fields separate.
5. Twin-run byte-identical.

## Risks

- Fabricating model behavior: model arms are typed only — no simulated
  outcomes.
- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
