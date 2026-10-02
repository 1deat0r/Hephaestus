# T-056 spec - citation parity for audited false-positive obligations

Status: ready-for-agent

## Problem Statement

The coverage scan reports 25 uncited R-ids: 24 obligations whose
implementations exist (audited across runs 17–23) but whose code
carries no R citation, plus R-054 (deferred). Each run's derivation
re-audits the same list because the signal lies. Code-verified
obligations should be named at the tests/docs that exercise them;
doc-process obligations must NOT be named in code (fabrication).

## Requirements trace

- Retrofit set (code-verified, R-IDs only): R-004, R-006, R-008,
  R-028, R-030, R-038, R-039, R-041, R-042, R-043, R-047, R-048,
  R-053, R-064, R-093.
- Excluded: R-079/081/085-089 (doc-process, ledger-documented),
  R-054 (deferred), R-007/R-058 (re-audited: genuinely open → future
  goals).

## Design

Comment-only edits at 14 verified sites (grill Q1 map). No schema,
generator, behavior, or test changes.

## Acceptance Criteria

1. Every retrofit R-id appears in crates/python at its verified site.
2. Re-run scan: uncited set = doc-process (079/081/085-089) + R-007 +
   R-054 + R-058 exactly (10), nothing else.
3. Gates green (`make ci` + `make doc-check`) — comment edits only.
4. No fabricated claims: R-IDs only, no AT claims, no schema edits.

## Out of Scope

Schema/generated contracts, new tests (R-007/R-058 goals), R-054,
committing (rule 5).

## Further Notes

Grill: `.scratch/t056-citation-parity/grill.md`.
