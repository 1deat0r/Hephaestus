# T-061 grill - E2E mission driver (goal -> test-ready hypothesis -> dossier)

Goal (goal_source: user): close the gap between "all modules unit-
tested" and "one real end-to-end run" — the missing compositor I
identified as 1-3 runs from an E2E smoke; the user asked the distance
to a benchmark workflow and said Continue.

## Facts (probe results)

- CLI exists but only `fixture run|recover` (M1 DAG). NO test
  chains mission + discovery + genesis + experiment (grep: zero
  partial chains).
- Mission: `compile(intake) -> Compiled::{Mission, NeedsAuthorization}`
  (auth fixtures in mission_auth.rs; some goals compile straight to
  Mission).
- Discovery: `analyze(&Corpus, &[TraceRecord]) -> AnalysisOutcome`
  — TraceRecords are span-cited over ingested source bytes
  (discovery test precedent: trace text ingested, records cite line
  spans).
- Genesis: `registry::apply_all(opportunity, ...) -> OperatorOutcome`
  (mechanisms) -> `hypothesis::compile(&MechanismRecord)` ->
  `validators::validate -> Readiness::TestReady`.
- Experiment: `experiment::compile(&CompileRequest, PlanInputs) ->
  Result<ExperimentPlan, Blocker>`; evaluation `interpret` needs
  Measurements{interval, threshold,...} + qualified method registry.
- **No interval computation exists anywhere** (grep: only grant time
  intervals + pilot variance) — ticket 02 must add a
  methods-qualified computation (R-033: computed, never fabricated).
- synthetic-evaluator is a SEPARATE crate (serde/sha2 only, its own
  19 tests, ZERO callers): hidden World + observations + probes +
  `evaluate -> Assessment{verdict, reason}`. hephaestus does not
  depend on it yet (dev-dep to add for e2e tests).
- Dossier: hand-assembled in tests; `export` gates exist (R-044/076/
  082).

## Q1 - Shape: test-first driver or src module?
**A:** Tracer bullets as integration tests (`tests/e2e_m2.rs`,
`tests/e2e_m3.rs`) producing receipts (the M0/M1 assessment pattern:
named tests are receipts); glue only promoted to src when reused.
CLI subcommand is ticket 03, after the chains prove out.
(agent-default)

## Q2 - Where does the M2 fixture data come from?
**A:** A fixture trace source derived FROM the hidden-world corpus
(synthetic-evaluator `load_corpus` worlds -> serialized trace text
ingested via knowledge::ingest_bytes; TraceRecords cite those spans
— discovery-test span discipline). Ground-truth VERDICTS stay out of
M2 (test never reads World truth; assertions are structural:
grounded opportunities + TestReady readiness), matching M2 exit
wording ("on hidden synthetic worlds ... exit is a hypothesis-
generation milestone, not proof of quality").
(agent-default)

## Q3 - Ticket 02's interval honesty?
**A:** Implement the qualified interval inside `methods` as a
registered MethodSpec (name/version/estimand/assumptions) + a
computation fn over observation outputs (documented formula, unit-
tested against hand-computed fixtures); interpret consumes it
exactly as today. Fixture executor: world.sample_observations -> fn
-> Measurements{interval, threshold from plan}; hidden
`evaluate()` supplies the M3 VERDICT (Assessment) beside the typed
result. No ad-hoc numerics anywhere (R-033 continuity).
(agent-default)

## Q4 - Authorization in the chain?
**A:** Intake fixture shaped so `compile -> Compiled::Mission`
directly (mission_auth precedent shows which goals do); the
NeedsAuthorization path stays ticket-03 scope only if the CLI needs
it. (agent-default)

## Q5 - Negative scenario for M3 exit?
**A:** Second e2e case in ticket 02: fixture world variant yielding
inconclusive/contradicted typed result -> honest negative dossier
export (M3 exit: "a negative result is exported honestly").
(agent-default)

## Q6 - Scope cuts?
**A:** No provider/model arms (benchmark baselines = later, needs
consent); no campaign execution (pilot machinery untouched); no
live-trigger/canary wiring (T-033 limitations stay recorded); no
commit (rule 5). (agent-default)
