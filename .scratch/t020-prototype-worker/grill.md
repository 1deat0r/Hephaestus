# T-020 grill - prototype worker: protected boundary + assembly verification

Goal: a prototype worker that may change candidate artifacts but NOT
protected evaluators or baselines; interface + end-to-end assembly
verification (IMPLEMENTATION_PLAN:70, MASTER_SPEC §15, R-064, R-094/095).

## Q1 - What is the unit of protection?
**A:** The protected set = evaluator identity (crate + function digest) and
baseline identity. The worker's `PrototypeChange` declares the artifact it
modifies; `authorize` rejects any change whose target path/digest overlaps
the protected set. Trust comes from the protected context, not the
worker's claims (R-095). (agent-default)

## Q2 - Where does this live?
**A:** New `crates/hephaestus/src/prototype/` module. It is the
authorization + assembly logic layer over the existing
`synthetic-evaluator` crate (which remains the protected evaluator). No
process isolation here - T-010 owns worker isolation; this module owns the
CHANGE-level contract. (agent-default)

## Q3 - Public seam?
**A:** `authorize(change, protected) -> Result<Receipt, Rejection>` +
`assemble(components, guardrails) -> AssemblyReport`. Red-first. (agent-default)

## Q4 - What does a PrototypeChange carry (§15:296)?
**A:** target artifact path, new artifact digest, claimed claims it tests,
build receipt, test receipt. Worker-supplied - hence everything is verified
against protected values before authorization. (agent-default)

## Q5 - Receipts (R-094)?
**A:** `BuildReceipt` / `TestReceipt`: artifact digest bytes-verified
against the protected registry before the change is accepted. A receipt
referencing an unverified digest is rejected (R-095: trust from protected
context). (agent-default)

## Q6 - Assembly verification (§15:297)?
**A:** `assemble` checks: interface compatibility (declared interfaces
match pairwise), version compatibility, global invariants (supplied as
named predicate strings with pass/fail), resource aggregation (sum of
component budgets within the mission limit), interaction effects declared
(a cost shift to another stage must be accounted: total-system-cost row).
End-to-end mission guardrails checked last; any failure names the failing
component. (agent-default)

## Q7 - What does a Component carry?
**A:** name, version, interface signatures (typed as strings here - the
contract layer, not a type system), resource budget, declared invariants,
provides/requires lists. (agent-default)

## Q8 - Baseline protection?
**A:** Baseline identity = digest in the protected context. A change
targeting the baseline (or evaluator) is rejected with reason; heuristic
rejections audited (R-024 pattern). (agent-default)

## Q9 - Negative space?
**A:** A worker that claims "evaluator unchanged" without the protected
digest is rejected - claims are not trust (R-095). Assembly with missing
interfaces fails with the missing edge named. (agent-default)

## Q10 - Vocabulary?
**A:** GLOSSARY rows FIRST: Prototype contract, Protected evaluator,
Assembly report. Decision row before edit. (agent-default)

## Q11 - TDD seams?
**A:** Red-first per ticket at `authorize` + `assemble`. (agent-default)
