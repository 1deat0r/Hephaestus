# T-042 spec - R-102: import-closure oracle

Status: ready-for-agent

## Problem Statement

R-102's negative case requires an independently qualified import-closure
oracle for the first domain (the workspace's own Rust import graph):
cycles, aliases, relative imports, deletion/rename, dynamic imports, and
oracle disagreement must be handled — declared closure matches the
qualified oracle, or conservative block without omission.

## Requirements trace

- R-102 (OBLIGATIONS:1625-1634).
- M6 exit continuity: no inherited proof (the oracle must be qualified
  against adjudicated fixtures before use).

## Acceptance Criteria

1. Closure check: declared set vs resolved imports — matches or named
   mismatch (stale/missing).
2. Cycles detected (no infinite loop); aliases and `crate::`/`super::`
   paths resolved.
3. Deletion/rename: stale declared entries flagged.
4. Dynamic/unsupported resolution: conservative BLOCK with named
   unresolved list (no silent omission).
5. Oracle disagreement: two implementations disagreeing -> closure not
   accepted (conservative).
6. Twin-run byte-identical.

## Risks

- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
