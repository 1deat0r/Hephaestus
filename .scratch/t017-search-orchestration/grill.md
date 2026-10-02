# T-017 grill — bounded search, lineage, dedup, diversity archive, rejection audits

Goal: search orchestration over the discovery→genesis chain with bounded
expansion, lineage, deduplication, a diversity archive, and rejection
audits — transparent beam/best-first, not a learned controller
(IMPLEMENTATION_PLAN:58, MASTER_SPEC §11, R-031, R-024).

## Q1 — Module placement?
**A:** New `crates/hephaestus/src/discovery_search/`? No — this orchestrates
discovery (T-014) THROUGH genesis (T-015/16); it sits above both. New
`crates/hephaestus/src/search_orchestrator/`? Terse: `orchestrator`. Pure
(no I/O), consumes opportunities/records/hypotheses given by callers.
(agent-default)

## Q2 — Search graph nodes (§11:217)?
Evidence: "Nodes store their parents, operator, evidence snapshot, evaluated
outcomes, estimated costs, and rejection reasons."
**A:** `SearchNode { id, parents: Vec<NodeId>, operator: String,
payload: NodePayload (Opportunity|MecanismRecord|Hypothesis as enum),
estimated_cost, outcome: Option<Outcome>, rejection: Option<String> }`.
(agent-default)

## Q3 — Beam policy (§11:219)?
**A:** Transparent best-first with beam width B (configured, bounded):
expand frontier nodes by a qualitative priority (discriminability +
evidence coverage — NO fabricated Bayesian posterior, §11:225); keep the
top-B per wave; the rest are archived, not deleted (diversity archive).
(agent-default)

## Q4 — Dedup (R-031/AT-031 negative: endless paraphrases)?
**A:** Exact + normalized-key dedup: candidate key = normalized
(kind + statement lowercase, whitespace-collapsed, sorted token multiset).
A paraphrase of an existing candidate maps to the same key → expansion
stops with reason `duplicate_generation` and a counter. (agent-default)

## Q5 — Bounds (R-031)?
Evidence: "bounds on expansion depth, duplicate generation, concurrent
workers, model calls, retrieval volume, elapsed budget, and total candidate
count" (§11:229).
**A:** `SearchBounds { max_depth, max_candidates, max_duplicate_strikes,
max_operator_calls }` enforced at the expansion loop; exceeding any stops
with a recorded reason. (agent-default)

## Q6 — Diversity archive + exploration/exploitation (§11:221)?
Evidence: 55/25/15/5 provisional allocation.
**A:** The archive retains every non-kept candidate with its rejection
reason (audit substrate, R-024). The allocation policy is a versioned
`AllocationPolicy { exploit, diversify, replicate, audit }` defaults
55/25/15/5 — logged, changeable only via policy version. The scheduler
reports the split; the orchestrator does not self-modify it.
(agent-default)

## Q7 — Rejection audits (§11:231, R-024)?
**A:** `audit()` returns every rejection: reason, candidate, wave. Nothing
unauditable. (agent-default)

## Q8 — Lineage (R-097 groundwork)?
**A:** Every node carries parents + operator + evidence snapshot hash
(sha of the source spans); `lineage(node) -> Vec<NodeId>` walks to roots.
Version-bound full lineage is T-033/M3; here: structural parents. Out of
scope: experiment nodes (no runner yet). (agent-default)

## Q9 — Acceptance tests?
**A:** (R-031) endless paraphrase stream → dedup stops expansion with
reason; bounds enforced (depth, candidate count, operator calls); beam
keeps ≤B per wave; archive retains everything non-kept with reasons;
allocation policy versioned with 55/25/15/5 defaults; lineage walks;
determinism twin-run. (agent-default)

## Q10 — GLOSSARY terms (row-first)?
**A:** `Search node`, `Diversity archive`, `Search bounds`. (agent-default)

## Q11 — TDD seams?
**A:** Public seam = `orchestrator::run(seed, sources, bounds, beam) ->
SearchOutcome` + `orchestrator::audit`. Red-first per ticket. (agent-default)
