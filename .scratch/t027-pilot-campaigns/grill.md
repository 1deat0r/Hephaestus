# T-027 grill - prospective pilot campaigns + variance estimation

Goal: prospective pilot campaigns with variance estimation for the larger
qualification study; metrics and analysis predeclared BEFORE the
confirmatory campaign; twenty pilot missions is a possible debugging batch,
NOT a mandatory evidence threshold (IMPLEMENTATION_PLAN:92, campaign
pilot/confirmatory partition paragraphs, R-103).

## Q1 - What does a pilot campaign need (contract layer)?
**A:** A typed PilotPlan: preregistered metrics + analysis reference (from
T-021 preregistration), stratification (size, dependency topology per the
campaign), train/pilot/confirmatory partition separation BY REPOSITORY,
mission batch definition with budget, and a variance-estimation OUTPUT
type. No live missions here — the campaign contract + variance records.
(agent-default)

## Q2 - Module placement + seam?
**A:** New `crates/hephaestus/src/pilot/` module. Seam:
`plan_pilot(spec) -> Result<PilotPlan, PlanError>` +
`record_outcome(plan, outcomes) -> VarianceEstimate` +
`ready_for_confirmation(plan, estimate) -> Result<(), ConfirmationBlock>`.
(agent-default)

## Q3 - Partition separation (campaign)?
**A:** Repositories are assigned to exactly ONE partition
(TrainTune/Pilot/Confirmatory); a repository appearing in two partitions
is a named PlanError. Change episodes sampled BEFORE confirmation
(recorded as episode ids per partition). (agent-default)

## Q4 - Preregistration linkage (R-103)?
**A:** PilotPlan carries the preregistration id from T-021's
`preregister`; a pilot without a frozen preregistration is refused
(no metrics/analysis predeclared = no pilot). (agent-default)

## Q5 - Twenty-missions rule?
**A:** NOT a threshold: batch size is free; the plan records the batch
size and the debug-vs-evidence intent label. Nothing enforces twenty.
(campaign: "a possible debugging batch, not a mandatory evidence
threshold") (agent-default)

## Q6 - Variance estimation?
**A:** `record_outcome` computes from the pilot outcomes: per-arm mean,
per-arm variance (population), paired-difference variance across
repositories, and the count of failed/blocked/invalid runs retained in
the denominator (R-103). Pure arithmetic on recorded outcomes — no
fabrication; empty outcomes -> an explicit None, not zero. (agent-default)

## Q7 - Confirmation gate?
**A:** `ready_for_confirmation` refuses: missing variance estimate,
unresolved oracle (link to T-021), pilot outcomes from the confirmatory
partition (contamination), or a plan whose analysis is unqualified.
(agent-default)

## Q8 - Vocabulary?
**A:** GLOSSARY rows FIRST: Pilot campaign, Variance estimate, Partition
separation. Decision row before edit. (agent-default)

## Q9 - TDD seams?
**A:** Red-first per ticket: plan+partitions (01), variance+gate (02).
(agent-default)
