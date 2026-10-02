# T-036 grill - R-116 negative case: stale/quarantine invalidation of learning reuse

Goal: R-116 negative case (OBLIGATIONS:1849-1852): "Restart after a
recorded failure and challenger decision; alter or quarantine its source"
-> "A subsequent mission uses current valid learning; stale/quarantined
dependencies invalidate reuse and holdout history survives." The
selfimprove ledger persists (R-116 positive covered in
tests/self_improvement.rs) but STALE/QUARANTINE INVALIDATION is missing.

## Q1 - What exactly is missing?
**A:** The ledger has no staleness/quarantine model: a subsequent mission
can reuse a learned entry whose source was altered or quarantined. Need:
typed dependency states (Valid / Stale / Quarantined) + a reuse gate that
refuses stale/quarantined entries while holdout history survives.
(agent-default)

## Q2 - Module placement + seam?
**A:** Extend `selfimprove` module with a new `learning` submodule:
`LearningEntryState`, `mark_stale` / `quarantine`, `usable_for_reuse`.
No existing API changes (ledger is append-only; state changes are
append-only records too). (agent-default)

## Q3 - What invalidates reuse (obligation text)?
**A:** (a) source ALTERED after recording — digest of the source no
longer matches the digest recorded at learning time; (b) source
QUARANTINED — explicit quarantine record. Both make the entry unusable;
valid entries remain usable. (agent-default)

## Q4 - Holdout history survives (obligation text)?
**A:** Staleness/quarantine NEVER erases history: the entry and its
fresh_partition_id remain in the ledger (append-only), only reuse is
gated. (agent-default)

## Q5 - Subsequent mission semantics?
**A:** `usable_for_reuse` filters: entry state Valid AND source digest
matches current source digest. Stale/quarantined entries are excluded
from reuse candidates but visible for audit. (agent-default)

## Q6 - Vocabulary?
**A:** GLOSSARY rows FIRST: Learning reuse, Stale dependency,
Quarantine. Decision row before edit. (agent-default)

## Q7 - TDD seams?
**A:** Red-first: state records + gates in `selfimprove::learning`.
(agent-default)
