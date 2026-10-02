# 05: Triage batch B3 — advisory, backend, and evaluation ATs (16)

**What to build:** The same read-both-sides audit for this batch.

**ATs:** AT-053 AT-054 AT-064 AT-065 AT-066 AT-067 AT-068 AT-069
AT-070 AT-071 AT-072 AT-074 AT-075 AT-077 AT-078 AT-079

**Blocked by:** None (run after 04).

**Status:** ready-for-agent
**Covers:** 3

**Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/05-b3-advisory-backend-eval-at-batch.md

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
(ticket 05 of the four-batch split).
