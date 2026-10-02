# T-024 grill - evidence graph connection, invalidation, portfolio queue

Goal: connect results back into the evidence graph, opportunity miner, and
portfolio queue; corrections and retractions invalidate dependent
interpretations; restarting research still requires available budget
(IMPLEMENTATION_PLAN:76, MASTER_SPEC §16, R-017, R-097).

## Q1 - What exists already?
**A:** `knowledge` module (T-013) has records; `genesis` + `discovery` +
`evaluation` produce typed objects. T-024 adds the CONNECTION layer:
state machines + invalidation traversal + portfolio queue. (agent-default)

## Q2 - Module placement + seam?
**A:** New `crates/hephaestus/src/lifecycle/` module. Seam:
`advance(opportunity|hypothesis, event) -> Result<State, TransitionError>`,
`invalidate(graph, correction) -> InvalidationReport`, `queue(portfolio,
candidate, budget) -> Result<(), QueueError>`. (agent-default)

## Q3 - Opportunity states (§16:311)?
**A:** discovered, grounded, prioritized, explored, parked, closed - linear
with parked/closed reachable from relevant stages; illegal jumps are named
TransitionErrors. Hypothesis states: exploratory, compiled, reviewed,
test-ready, testing, assessed (+ blocked, archived from relevant stages).
(agent-default)

## Q4 - Orthogonal fields (§16:312)?
**A:** A Candidate carrier holds scientific conclusion, engineering
attainment, novelty assessment, execution status as FOUR separate fields
never collapsed (R-046 continuity). (agent-default)

## Q5 - Corrections/retractions (§16:314, R-017)?
**A:** `invalidate` traverses dependent claims from the corrected record,
marks affected dossiers stale, queues re-evaluation where permitted.
Append-only: the correction is a NEW event referencing the original; the
prior record is never erased. Historical result remains intact while
current labels go stale (§16:317). (agent-default)

## Q6 - No automatic expensive rework (§16:314)?
**A:** Re-evaluation queue entries carry a `requires_budget_and_permission`
flag; the queue does NOT auto-dispatch expensive/external work - they wait
(here: represented as `QueuedAwaitingAuthorization` state, never silently
run). (agent-default)

## Q7 - Restarting research requires budget (roadmap line)?
**A:** `queue` with an empty/exhausted budget envelope returns
QueueError::BudgetUnavailable - a reactivated (parked->prioritized) item
cannot enter the queue without reserved budget. Reactivation (§16:316)
creates a NEW version referencing the original failure + the changed-
condition evidence; the failure is not erased. (agent-default)

## Q8 - Failure reactivation (§16:316)?
**A:** `reactivate(failure, changed_condition_evidence) -> NewVersion` -
new version referencing original failure record ID + evidence; original
retained verbatim. (agent-default)

## Q9 - Snapshots (§16:318)?
**A:** Every state transition binds a snapshot ID (versioned selection
reference) so later knowledge does not silently alter historical
interpretation. (agent-default)

## Q10 - Vocabulary?
**A:** GLOSSARY rows FIRST: Evidence lifecycle, Invalidation, Portfolio
queue. Decision row before edit. (agent-default)

## Q11 - TDD seams?
**A:** Red-first per ticket at `advance`/`invalidate` (01) and
`queue`/`reactivate` (02). (agent-default)
