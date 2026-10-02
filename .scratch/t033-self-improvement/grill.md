# T-033 grill - self-improvement service (bounded loop core)

Goal: outcome-triggered autonomous improvement loop - observe, diagnose,
propose, qualify against protected fresh evaluation, deploy permitted,
monitor, roll back; bounded budget; durable provenance-bearing learning;
rejection of false/inconclusive/permission-expanding changes
(IMPLEMENTATION_PLAN:84, docs/SELF_IMPROVEMENT.md, R-070-R-072, R-115-R-119).

## Q1 - Scope for ONE goal?
**A:** The full loop is M3-mandatory but enormous. This goal implements the
CORE deterministic service: candidate evaluation + promotion decision +
rollback semantics + ledger persistence + the rejection classes (false/
inconclusive/over-budget/permission-expanding). Trigger sources + canary
monitoring loop + crash reconciliation are represented as typed records
and entry points but NOT wired to live triggers (their provers are the
T-020 evaluation path + existing recovery module). Remaining: live trigger
wiring, actual canary runtime - recorded in the report as limitations.
(SELF_IMPROVEMENT §R-115, agent-default)

## Q2 - Module placement + seam?
**A:** New `crates/hephaestus/src/selfimprove/` module. Seam:
`evaluate_candidate(challenger, manifest) -> Assessment`,
`promote(assessment, grant) -> Result<Deployment, PromotionRejection>`,
`rollback(deployment, reason) -> RollbackReceipt`. (agent-default)

## Q3 - ImprovementCandidate fields (§R-117)?
**A:** incumbent id+digest, challenger id+digest, target, supporting
observation record ids, immutable evaluation manifest (frozen bounds,
sampling units, unsuccessful-run denominators, tooling access, method
qualification, stopping+multiplicity rules), fresh partition/family id,
total resource budget, intended primary benefit, guardrails, rollout and
rollback artifact digests. (agent-default)

## Q4 - Promotion requirements (§R-117)?
**A:** Promote requires ALL of: authenticated assessment bound to the exact
payload digest; verified candidate+evaluation+rollback artifacts (digests
in a protected registry, R-095 continuity); benefit_status == Supported
(predeclared benefit, not post-hoc); guardrail_status == passing; current
scoped grant covering the change type. Missing ANY -> named
PromotionRejection. (agent-default)

## Q5 - Rejection classes that must stay undeployed (§R-119)?
**A:** False (supported=false), Inconclusive (benefit interval overlaps
bound), OverBudget (challenger cost > manifest budget), PermissionExpanding
(grant scope does not cover target - e.g. touching protected evaluator or
spending limits through the improvement loop). Each a named variant.
(agent-default)

## Q6 - Self-attestation forbidden (§R-117)?
**A:** Assessment must carry evaluator identity EXTERNAL to the challenger
digest; a challenger whose digest == evaluator digest is rejected
(model/agent cannot attest its own qualification). (agent-default)

## Q7 - Deploy + champion pointer (§R-118)?
**A:** `promote` success -> Deployment with transactional champion-pointer
update WITHOUT overwriting the incumbent (incumbent retained verbatim,
rollback target recorded). Deployment receipt: scope, assessment digest,
rollback target. (agent-default)

## Q8 - Rollback (§R-118)?
**A:** `rollback` restores the verified incumbent (digest check), records
reason + observations; the rolled-back candidate retains its reason -
retries need new evidence/version (callers must supply a NEW candidate
version; same-digest retry rejected). (agent-default)

## Q9 - Budget non-multiplication (§R-115)?
**A:** Recursive improvement proposals stay within the ORIGINAL envelope:
`evaluate_candidate` carries `budget_from` (parent budget id); a proposal
whose declared budget source is itself is rejected (BudgetMultiplication).
(agent-default)

## Q10 - Learning persistence (§R-116)?
**A:** An append-only `ImprovementLedger`: every candidate (deployed or
rejected) with reason, observations, champion/challenger identities +
provenance. Restart = re-read ledger (no in-memory-only state).
(agent-default)

## Q11 - Vocabulary?
**A:** GLOSSARY rows FIRST: Improvement candidate, Champion,
Rollback receipt. Decision row before edit. (agent-default)

## Q12 - TDD seams?
**A:** Red-first per ticket at `evaluate_candidate`/`promote` (01) and
`rollback`+ledger (02). (agent-default)
