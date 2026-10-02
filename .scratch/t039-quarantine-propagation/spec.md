# T-039 spec - R-109: quarantine propagation

Status: ready-for-agent

## Problem Statement

R-109's negative case: unauthorized behavioral-memory writes denied;
quarantining a source invalidates dependent CURRENT artifacts and
labels; immutable audit history survives. The knowledge service excludes
quarantined sources from retrieval but propagation does not exist.

## Requirements trace

- R-109 (OBLIGATIONS:1737-1748).
- T-036 pattern: append-only invalidation, history survives.

## Acceptance Criteria

1. Unauthorized write to policy/behavioral memory denied (typed).
2. Quarantining a source invalidates derived current artifacts +
   labels.
3. Invalidated records remain in the ledger (audit survives).
4. Twin-run byte-identical.

## Risks

- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
