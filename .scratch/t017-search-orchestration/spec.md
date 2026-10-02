# T-017 spec — bounded search, lineage, dedup, diversity archive, rejection audits

Status: ready-for-agent

## Problem Statement

The discovery→genesis chain (T-014→T-016) produces opportunities, mechanism
records, and hypotheses — but nothing orchestrates them into a bounded,
lineage-tracked search. Nothing stops endless paraphrase generation, keeps
a diversity archive, or audits rejections (IMPLEMENTATION_PLAN:58,
MASTER_SPEC §11, R-031, R-024).

## Solution

A pure `orchestrator` module: a search graph whose nodes carry parents,
operator, evidence snapshot, cost estimate, and rejection reasons (§11:217);
a transparent best-first beam with configurable width; normalized-key
deduplication that stops paraphrase loops with recorded reasons (AT-031);
`SearchBounds` enforcing depth/candidate/duplicate/operator-call limits; a
diversity archive retaining every non-kept candidate with its rejection
reason; a versioned allocation policy (provisional 55/25/15/5, §11:221);
and a full rejection audit. No learned controller, no fabricated posteriors
(§11:225).

## User Stories

1. As an orchestrator caller, I want expansion bounded by depth, candidate
   count, and operator calls, so that generation always terminates with a
   recorded reason (R-031).
2. As a dedup caller, I want paraphrases to map to existing candidates and
   stop expansion, so that "endless paraphrases" cannot loop (AT-031
   negative).
3. As an auditor, I want every non-kept candidate retained with its
   rejection reason, so that heuristic rejections are auditable (R-024).
4. As a lineage caller, I want each node to carry parents + operator +
   evidence snapshot, so that any candidate's provenance walks to its seed.
5. As a budget caller, I want a versioned allocation policy with the
   provisional 55/25/15/5 defaults, so that exploration and exploitation
   are both preserved and the split is visible (§11:221).

## Out of Scope

- MCTS/QD/Bayesian/bandit extensions (§11:219 — interchangeable, not
  prerequisites); experiment execution nodes (M3); expected-information-gain
  computation (needs a defensible predictive model — qualitative
  discriminability used instead, §11:225); full R-097 version-bound dossier
  lineage (M3, T-033).

## Acceptance Criteria

1. Paraphrase stream → dedup stops expansion with `duplicate_generation`
   reason (AT-031).
2. Bounds enforced: max_depth, max_candidates, max_operator_calls — each
   exceeding stops with its recorded reason.
3. Beam keeps ≤ configured width per wave; non-kept candidates go to the
   archive with reasons.
4. Every node carries parents + operator + evidence snapshot; lineage
   walks to the seed.
5. Allocation policy versioned, 55/25/15/5 defaults, visible in the
   outcome.
6. `audit()` returns every rejection (reason + candidate + wave).
7. Twin-run byte-identical; no fabricated probabilities.
8. `cargo test --workspace` green; clippy `-D warnings`; fmt; `make ci`
   EXIT=0.

## Success Risks

- Priority heuristics becoming opaque: the beam sorts by (discriminability
  present, evidence count) — named, deterministic, documented.
- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
