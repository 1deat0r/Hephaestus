# T-019 grill - Experiment Compiler (plans, discrimination, blockers)

Goal: experiment-plan compilation with primary endpoints, comparators,
controls, sampling unit, analysis method, stopping rule, protected evaluator
digest, budget, frozen hypothesis version (IMPLEMENTATION_PLAN:68,
MASTER_SPEC §13, R-040, R-096).

## Q1 - Module placement?
**A:** New `crates/hephaestus/src/experiment/` module. Pure logic, no I/O,
consistent with discovery/genesis/priorart. (agent-default)

## Q2 - Public seam?
**A:** `compile(hypothesis, request) -> Result<ExperimentPlan, Blocker>` +
`discrimination_matrix(plan) -> Vec<MatrixRow>` + `validate(plan) ->
Verdict`. Red-first at this seam. (agent-default)

## Q3 - What binds an ExperimentPlan (§13:255)?
**A:** All of: hypothesis version + claim IDs, operating conditions,
comparator, intervention artifact, measurement procedure, units, primary
endpoints, guardrails, sampling unit, analysis specification, stopping rule,
evaluator hash, resource limit (budget), authorization scope. Compilation
freezes the hypothesis version - plans bind a version, not a mutable ref.
(agent-default)

## Q4 - Completeness gate (R-096/AT-096)?
**A:** `validate` refuses qualification until primary endpoint, control,
sampling unit, and guardrail coverage are all present; each missing item is
a named denial. (agent-default)

## Q5 - Predeclared analysis + stopping rule (R-040)?
**A:** Analysis spec + stopping rule are REQUIRED fields; absent -> compile
fails (compile-time, not validate-time, since §13:255 lists them as plan
fields). (agent-default)

## Q6 - Discrimination matrix (§13:257)?
**A:** Rows map predicted observations to (proposed mechanism, strongest
alternative, artifact/null). No invented probabilities - prediction text
only. If candidate and alternative are indistinguishable in the proposed
test, the matrix flags `cannot_discriminate` (the test cannot establish the
mechanism's distinct contribution). (agent-default)

## Q7 - Competing explanations (§13:256)?
**A:** Alternatives are explicit on the plan: warmed cache, dropped work,
weaker baseline, changed dataset, lower-quality outputs - supplied per
hypothesis kind; a mechanism-sensitive ablation is one of the enum choices.
(agent-default)

## Q8 - Controls (§13:259)?
**A:** positive/known-working where appropriate, negative control,
randomization, repeatability, environmental capture, missing-observation
plan. Control failure semantics: a control failure invalidates the affected
inference - represented as a flag on the plan evaluated at result time (M3+
scope: here just the structural fields + an `invalidates_inference` marker
type). (agent-default)

## Q9 - Blockers (§13:262)?
**A:** Enum: MissingInstrument, InaccessibleDataset, UnvalidatedSimulator,
InadequatePower, ForbiddenAction, ResourceOverrun - precise, no imagined
outcomes. (agent-default)

## Q10 - Evaluator digest (R-094/R-095)?
**A:** SHA-256 over evaluator identity bytes, carried on the plan;
plans with mismatched evaluator digests are rejected at validate. Protected
context provenance: digest comes from the caller-supplied protected value,
never from candidate artifacts. (agent-default)

## Q11 - Frozen hypothesis version (R-097/R-098)?
**A:** Plan carries `hypothesis_version: String`; stale/retracted versions
are caller-checked (R-098 invalidation is lifecycle scope, not this module);
the plan binds immutably. (agent-default)

## Q12 - Vocabulary?
**A:** GLOSSARY rows FIRST: Experiment plan, Discrimination matrix,
Blocker. Decision row before edit. (agent-default)

## Q13 - TDD seams?
**A:** Red-first per ticket at `compile`/`validate`/`discrimination_matrix`.
(agent-default)
