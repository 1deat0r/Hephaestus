# T-050 grill - R-083: compare proposed context caching against a credible existing baseline

Goal: R-083/AT-083 (Protected context oracle and evaluator; contract
M0, runtime M3, MASTER_SPEC §28) — negative: "Use only full
reconstruction despite a stronger exact-cache implementation" →
required: "Benchmark review requests the stronger comparator or
justifies exclusion."

Existing surfaces (facts):
- §28 worked example verbatim: "current caching is a credible
  baseline ... A full-rebuild straw baseline alone is inadequate."
- `acceleration` (T-031) = matched-ablation discipline (envelope-equal
  pairs, recorded measurements, promotion verdicts) — zero `cach`
  hits; MechanismKind is the six M5 mechanisms (context caching is
  §28's concern, NOT one of them — no enum change).
- Tests home exists: tests/acceleration_eval.rs (extend, no new file).
- R-041/R-042 confirmed false positives this intake (noninferiority
  test exists in result_interpreter).

## Q1 - What is the surface?
**A:** `CachingComparator { FullReconstruction, ExactCache }` (named
after the obligation's two implementations) plus
`CachingComparison { comparators_used: Vec<CachingComparator>,
exclusion_justification: Option<String> }` and
`review_caching_comparison(&CachingComparison)` in the acceleration
module — the comparison-review seam that context-oracle/evaluator
callers compose with (wiring out of scope, per standing precedent).
(agent-default)

## Q2 - Outcomes and refusals?
**A:** `Ok(StrongerComparatorUsed)` when ExactCache was used;
`Ok(ExclusionJustified { justification })` when it was excluded with
a non-empty justification; `Err(MissingStrongerComparatorOrJustification)`
when review ran with neither — the AT-083 negative (full
reconstruction alone, unjustified) is exactly that error. The
obligation text carries no availability nuance, so neither does the
API. (agent-default)

## Q3 - Files/allowlist?
**A:** types into acceleration/record.rs + fns into acceleration/mod.rs
(both already allowlisted → content edits only); tests extend
acceleration_eval.rs (allowlisted); allowlist +3 = .scratch/t050
grill/spec/issue; reseal. Glossary rows: Stronger comparator,
Exclusion justification. Citations R-083/AT-083 in module docs + test
names. Red-first at review_caching_comparison. (agent-default)

## Q4 - Scope cuts?
**A:** (1) oracle/evaluator wiring — composition contract in spec;
(2) actually building an exact-cache implementation — the gate is
about COMPARISON rigor, not implementing caching; (3) MechanismKind /
ablation pair machinery untouched; (4) no commit (rule 5).
(agent-default)
