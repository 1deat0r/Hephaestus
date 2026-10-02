# 04: Triage batch B2 — discovery, experiment, release ATs (16)

**What to build:** The same read-both-sides audit for this batch.

**ATs:** AT-028 AT-029 AT-030 AT-033 AT-037 AT-038 AT-039 AT-040
AT-041 AT-042 AT-043 AT-044 AT-046 AT-047 AT-048 AT-051

**Blocked by:** None (01 and 02 are done; run after 03 for a clean
sequential frontier).

**Status:** ready-for-agent
**Covers:** 3

**Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md

- [ ] All 16 dispositioned: cited or goal candidate
- [ ] Re-scan: batch absent; gates green

**Candidate goals:** (none yet)


**Micro-tasks:** (each ends in one commit and one push)

1. Micro — audit the first eight listed IDs
   - nano: extract the listed negative cases from OBLIGATIONS.md
   - nano: locate one mirror test per ID (grep by concept, read both sides)
   - nano: apply cite edits only at verified mirrors; never force a match
   - nano: cargo fmt --check + ticket-status, then commit and push
2. Micro — audit the remaining eight listed IDs
   - nano: extract the listed negative cases from OBLIGATIONS.md
   - nano: locate one mirror test per ID (grep by concept, read both sides)
   - nano: apply cite edits only at verified mirrors; never force a match
   - nano: cargo fmt --check + ticket-status, then commit and push
3. Micro — close the batch
   - nano: run the batch Verify command
   - nano: record goal candidates under Candidate goals (never silent)
   - nano: flip Status, tick every box, commit and push


## Comments

Grill: `.scratch/t066-at-citation-parity/grill.md`. Split 2026-10-02
(ticket 04 of the four-batch split).
