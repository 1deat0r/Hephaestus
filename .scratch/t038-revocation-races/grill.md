# T-038 grill - R-111: revocation/cancellation races — safe stop + event preservation

Goal: R-111 (amendment row T-007/T-010/T-011, OBLIGATIONS): "Monitor
containment violations, stop safely and preserve events during revocation
and cancellation races." Gaps found by grep: no revocation/cancellation
race handling anywhere; R-111 has no tests.

## Q1 - What is the race?
**A:** A dispatch is running inside a sandbox while its grant is revoked
or the task is cancelled. Required behavior: in-flight work stops SAFELY
(no partially-authorised side effects), containment violations observed
during the race are recorded, and the event ledger preserves ALL events
from both sequences in order — no event loss, no retroactive grant
resurrection. (agent-default)

## Q2 - Module placement + seam?
**A:** New `crates/hephaestus/src/containment/` module. Seam:
`race_guard(grant_state, cancel_state, events) -> RaceOutcome`,
`record_violation`. Pure/typed layer over the existing event ledger +
grant state (no live process wiring — that is the same honest boundary
as T-033's canary record). (agent-default)

## Q3 - What does the guard guarantee?
**A:** (a) Revoked grant + running dispatch -> outcome StoppedSafely;
the dispatch may not emit further authorized events; (b) cancellation +
completed dispatch -> CompletedBeforeCancel (no fabricated rollback of
finished work); (c) all events from both sequences preserved in the
ledger (append-only, in order). (agent-default)

## Q4 - Containment violation during race?
**A:** `record_violation` appends a typed Violation event; the violation
does NOT un-revoke the grant or resurrect any authorization — it is
evidence only, feeding the existing policy engine's GrantRevoked reason.
(agent-default)

## Q5 - Vocabulary?
**A:** GLOSSARY rows FIRST: Revocation race, Safe stop. Decision row
before edit. (agent-default)

## Q6 - TDD seams?
**A:** Red-first: race outcomes + event preservation. (agent-default)
