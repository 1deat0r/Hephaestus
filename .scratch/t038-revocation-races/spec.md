# T-038 spec - R-111: revocation/cancellation races

Status: ready-for-agent

## Problem Statement

R-111 requires monitoring containment violations, safe stopping, and
event preservation during revocation and cancellation races. No race
handling exists; R-111 has no tests. Typed race-guard layer over the
event ledger + grant state.

## Requirements trace

- R-111: containment violation monitoring; safe stop; event
  preservation during revocation and cancellation races.
- R-070 continuity: the grant ledger stays append-only; no
  resurrection of revoked grants.

## Acceptance Criteria

1. Revoked grant + in-flight dispatch -> StoppedSafely; no further
   authorized events from that dispatch.
2. Cancellation after completion -> CompletedBeforeCancel (no
   fabricated undo of finished work).
3. Events from both sequences preserved append-only, in order.
4. Violations recorded as typed evidence; never un-revoke.
5. Twin-run byte-identical.

## Risks

- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
