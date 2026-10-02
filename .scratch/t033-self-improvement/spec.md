# T-033 spec - self-improvement service core

Status: ready-for-agent

## Problem Statement

M3 requires a functioning self-improvement loop core: candidates proposed
from mission outcomes are evaluated against protected fresh evaluations
under frozen manifests, promoted only on authenticated supported benefit +
passing guardrails + current scoped grant, deployed with a transactional
champion pointer (incumbent retained), rolled back on violation, and all
outcomes persisted in an append-only ledger. False, inconclusive,
over-budget, and permission-expanding challengers stay undeployed; a
candidate cannot attest itself; budgets cannot multiply.

## Requirements trace

- R-070: challengers evaluated under protected matched-budget tests
  before promotion.
- R-071: self-improvement cannot expand budgets, permissions, protected
  gates.
- R-072 continuity: outcomes + rejections persisted with provenance.
- R-115: bounded loop, no budget multiplication, negative results kept.
- R-116: durable learning, provenance-bearing, survives restart.
- R-117: candidate fields, promotion requirements, no self-attestation.
- R-118: canary scope record, transactional pointer, rollback receipt,
  retry needs new version.

## Acceptance Criteria

1. Candidate carries all §R-117 fields; manifest frozen.
2. `promote` rejects: false benefit, inconclusive, over-budget,
   permission-expanding, self-attested, missing artifacts — each named.
3. Promotion succeeds on supported benefit + passing guardrails + scoped
   grant + verified artifacts; deployment retains incumbent + rollback
   target.
4. `rollback` restores verified incumbent; same-digest retry rejected.
5. Budget multiplication rejected.
6. Ledger append-only; survives "restart" (reconstructed from ledger).
7. Twin-run byte-identical.

## Limitations (recorded, not hidden)

- Live trigger wiring (mission-completion, interval, drift) is typed but
  not wired to running services in this goal.
- Canary monitoring is a record + policy check, not a live watchdog.
- Crash reconciliation delegates to the existing recovery module's
  semantics; end-to-end crash demonstration is T-033's exit demo, later.

## Risks

- Over-engineering: no statistical computation — benefit status arrives
  from the qualified evaluation (T-022 semantics).
- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
