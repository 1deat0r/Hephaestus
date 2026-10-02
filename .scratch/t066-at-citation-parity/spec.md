# T-066 spec - AT-scope citation parity

Status: ready-for-agent

## Problem Statement

77 acceptance tests are cited nowhere in implementation code (fresh
scan). The R-signal is gated and true (ADR-027); the AT-signal is an
ad-hoc grep whose gaps are re-audited from scratch every run — and
some ATs may genuinely be untested (a finding worth surfacing, not
burying).

## Requirements trace

- Serves R-081's spirit and the derivation-quality discipline
  (ADR-024 advisory tier for the report; ADR-027 lineage for the
  eventual graduation into `ci`). No new R obligation.

## Design

1. `tools/report_at_citations.py` + `make at-coverage` (advisory,
   exit 0): uncited ATs with R id, OBLIGATIONS negative case, required
   outcome, and the R's existing cite sites.
2. Ticket 02: audit AT-094..AT-119 (amendment family) against their
   negative cases → honest cites at mirrors; non-mirrors → goal
   candidates recorded (ticket + decisions).
3. Ticket 03: same for the remaining core ATs — SPLIT 2026-10-02 into
   four small batches (03: 14 ATs, 04: 16, 05: 16, 06: 9) so each step
   is one small task with one commit and one push.

## Acceptance Criteria

1. `make at-coverage` prints the full worklist with negative cases;
   `make ci` unchanged-green (R-gate semantics untouched).
2. Batch A (AT-094..119): every AT dispositioned — cited at a
   verified mirror OR recorded as a goal candidate in the ticket;
   re-scan shows those ATs no longer in the silent-gap set.
3. Batch B: same for the rest.
4. Gates green; allowlist/seal as needed (+files); no fabricated
   cites (each cite site re-readable against its negative case).

## Out of Scope

AT allowlists, auto-graduation into `ci`, commit (rule 5).

## Further Notes

Grill: `.scratch/t066-at-citation-parity/grill.md`.
