# T-016 grill — Hypothesis Compiler + semantic validators (self-interview)

Goal: transform mechanism candidates into operational hypotheses with
comparator, units, conditions, predictions, competitors, and operational
falsifiers for test-ready candidates; engineering targets separate from
mechanism claims (IMPLEMENTATION_PLAN:56, MASTER_SPEC §9, R-025/026/027).

## Q1 — Module placement?
Evidence: §9 owner is the Hypothesis Genesis Engine; T-015's mechanism
contract is the input.
**A:** Extend `crates/hephaestus/src/genesis/` with `hypothesis.rs`
(record + compiler) and `validators.rs` (semantic validation). The
mechanism contract stays in record.rs. (agent-default)

## Q2 — What is the Hypothesis record? (§9:178 enumerates)
Evidence: "context, intervention, comparator, proposed mechanism, measurable
predictions, estimands, meaningful effect bounds, boundary conditions,
competing explanations, required observations, analysis requirements, and
explicit outcomes that would count against the claim."
**A:** All 12 fields, `Option`-bearing where unknowns are explicit. A
`readiness` field: `TestReady | Exploratory | BlockedTestability` — "no
credible discriminator, no test-ready hypothesis" (§9:182), never deletion.
(agent-default)

## Q3 — Mechanistic claims vs engineering targets (§9:180)?
**A:** Two separate fields: `mechanism_claim` (from the MechanismRecord) and
`engineering_target` (Option, with threshold provenance REQUIRED — R-026).
Conclusions reference them separately; a target met without the mechanism is
recorded as inconclusive-for-mechanism. (agent-default)

## Q4 — Prediction structure (§9:182)?
**A:** Empirical predictions carry: measured quantity, unit, direction,
observation scope, uncertainty treatment, decision rule. Formal claims
carry proposition + assumptions + checker. No compiler-invented thresholds:
every threshold cites provenance (user requirement / prior study / pilot /
explicit provisional choice) or the hypothesis is returned for completion
(AT-026 negative). (agent-default)

## Q5 — Semantic validators (R-025/AT-025)?
Evidence: §9:184 check questions; negative case: removing falsifiers and
competing explanations must DENY test-ready promotion.
**A:** `validate()` denies test-ready when: no operational discriminator;
falsifiers or competing explanations empty; comparator missing/irrelevant;
variables unmeasurable; hypothesis restates its own success metric
(self-fulfilling check); preconditions unrealizable-in-principle. Denial is
structured with reasons; the hypothesis stays with readiness downgraded.
(agent-default)

## Q6 — Versioning + lineage (R-027/AT-027)?
Evidence: "Revision creates a new hypothesis version with lineage. A changed
mechanism, boundary condition, primary endpoint, or falsifier cannot inherit
confirmatory support without an explicit assessment."
**A:** `revise()` mints version+1 with `supersedes`; substantive-change
detection (mechanism/boundary/endpoint/falsifier diff) forces
`inherited_support: NeedsReassessment` — silent inheritance impossible.
(agent-default)

## Q7 — No invented numbers?
**A:** Thresholds never minted by the compiler; effect bounds carry
provenance or the hypothesis returns for completion. Same rule as T-014's
value estimates. (agent-default)

## Q8 — Acceptance tests?
**A:** (AT-025) hypothesis with falsifiers/competitors stripped → semantic
validation denies test-ready with reasons; (AT-026) arbitrary target without
reason/comparator → returned for completion; (AT-027) mechanism changed
post-observation → new version cannot inherit support silently;
(EXPLORATORY) no-discriminator hypothesis stays exploratory, not deleted;
determinism twin-run; compiler never invents thresholds. (agent-default)

## Q9 — GLOSSARY terms (row-first)?
**A:** `Hypothesis`, `Operational discriminator`, `Falsifier`. (agent-default)

## Q10 — TDD seams?
**A:** Public seam = `genesis::hypothesis::compile/revise` +
`validators::validate`. Red-first per ticket. (agent-default)
