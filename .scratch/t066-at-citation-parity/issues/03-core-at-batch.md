# 03: Triage batch B1 — mission and corpus-era ATs (14)

**What to build:** Honest audit of these ATs: read the OBLIGATIONS
negative case AND the candidate test; cite at a true mirror; record
every non-mirror as a goal candidate in this ticket + a decisions row.

**ATs:** AT-001 AT-002 AT-003 AT-004 AT-005 AT-006 AT-008 AT-009
AT-010 AT-011 AT-012 AT-016 AT-017 AT-018

**Blocked by:** None (01 and 02 are done).

**Status:** done
**Covers:** 3

**Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/03-core-at-batch.md

- [x] All 14 dispositioned: cited (verified mirror) or goal candidate
- [x] Re-scan: batch absent from the silent-gap set; gates green

**Candidate goals:**
- AT-016 — citation with a matching title but no supporting passage: no test exercises that exact scenario (nearest is generic span verification) -> goal candidate, never a silent skip.


**Micro-steps:**

1. M1: extract negative cases for the first seven listed IDs from OBLIGATIONS; locate mirror tests; apply cites -> commit+push
2. M2: same for the remaining seven listed IDs -> commit+push
3. N: record any goal candidates, run batch Verify, flip status -> commit+push

## Comments

Grill: `.scratch/t066-at-citation-parity/grill.md`.
Split from the original 55-AT ticket on 2026-10-02 so each step stays
a small task (14 ATs). Batches B2/B3/B4 = tickets 04/05/06.
