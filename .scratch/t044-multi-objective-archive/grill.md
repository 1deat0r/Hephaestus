# T-044 grill - R-032: multi-objective archive gate

Goal: R-032 (Portfolio scheduler, runtime M2): "Keep a diverse
multi-objective archive rather than a single universal score."
AT-032 negative case: "Compare a cheap uncertain candidate and an
expensive plausible candidate. Required outcome: Both can remain in
the frontier when neither dominates."

Existing: orchestrator has SearchNode { estimated_cost, ... },
ArchiveEntry, beam frontier. But there is NO multi-objective archive:
selection is single-axis priority. Yield accounting reports
separately but the search frontier itself has no dominance test.

## Q1 - Seam?
**A:** `archive` module (or extend orchestrator): a MultiObjectiveArchive
over SearchNode with per-axis vectors. Axes chosen from what nodes
actually carry: estimated_cost (min), and statement-derived diversity
(dedup_key distance). Confidence axis: nodes have no confidence field
today — but "cheap uncertain vs expensive plausible" needs both cost
and value/uncertainty axes. (agent-default)

## Q2 - Minimal honest design?
**A:** Two-axis Pareto: cost (minimize) x information value (maximize,
declared per node by the producer — NOT fabricated; nodes without a
declared value default to the uncertainty-floor value so they are
never auto-eliminated by a missing score, R-024 continuity). Dominance:
a dominates b iff a is <= on ALL axes and < on >=1. Frontier = all
non-dominated nodes. Cheap-uncertain and expensive-plausible are
mutually non-dominating -> both stay. (agent-default)

## Q3 - Bounded archive?
**A:** Yes — archive growth is bounded (R-031 continuity): when the
frontier exceeds a cap, evict dominated nodes first, then the least
recently admitted among the same objective corner; eviction is
recorded in ArchiveEntry-style reason records, never silent.
(agent-default)

## Q4 - Where does "value" come from without fabricating probabilities (R-033)?
**A:** Declared, not computed: the producing operator states its value
claim as a qualitative band (Low/Medium/High), serialized on the node.
No numeric probability is invented; Pareto works on band rank.
(agent-default)

## Q5 - TDD seams?
**A:** Red-first tests: (1) AT-032 scenario — cheap+uncertain vs
expensive+plausible both survive; (2) dominated node is evictable;
(3) cap eviction leaves an audit reason; (4) twin-run determinism.
(agent-default)

## Q6 - Vocabulary?
**A:** GLOSSARY row: Multi-objective archive. (agent-default)
