# T-024 spec - evidence graph connection, invalidation, portfolio queue

Status: ready-for-agent

## Problem Statement

Results and dossiers exist (T-022/T-023) but nothing connects them back
into an evidence lifecycle: explicit states (§16:311), orthogonal fields
(§16:312), append-only corrections that invalidate dependents (§16:314),
budget-gated restarts, and a portfolio queue. R-017 requires correction
preservation; R-097 requires version-bound lineage.

## Requirements trace

- R-017 / AT-017: validity time, ingestion time, corrections preserved.
- R-097: version-bound chain IDs recorded through transitions.
- R-046 continuity: orthogonal status fields never collapsed.
- §16:311: explicit state machines for opportunities + hypotheses.
- §16:314: append-only invalidation traversal; no auto expensive rework.
- §16:316: reactivation creates a new version referencing the failure.
- §16:318: every transition binds a snapshot ID.

## Acceptance Criteria

1. Opportunity + hypothesis state machines with named illegal transitions.
2. Orthogonal four-field candidate carrier.
3. `invalidate` traverses dependents, marks dossiers stale, queues
   re-evaluation; prior record intact (append-only).
4. Re-evaluation entries await budget+permission; never auto-dispatched.
5. `queue` refuses without budget; `reactivate` creates new version
   referencing the original failure.
6. Every transition binds a snapshot ID.
7. Twin-run byte-identical reports.

## Risks

- Over-engineering: state machines as enums + match tables, not a rules
  engine.
- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
