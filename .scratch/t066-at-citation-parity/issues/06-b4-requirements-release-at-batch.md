# 06: Triage batch B4 — requirements, release, and self-improvement ATs (9)

**What to build:** The same read-both-sides audit for this batch.

**ATs:** AT-081 AT-085 AT-086 AT-087 AT-088 AT-089 AT-090 AT-093
AT-097 (AT-097 is already a recorded goal candidate from ticket 02 —
confirm and keep, do not silently cite)

**Blocked by:** None (run after 05).

**Status:** ready-for-agent
**Covers:** 3, 4

**Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/06-b4-requirements-release-at-batch.md

- [ ] All 9 dispositioned: cited or goal candidate (AT-097 keeps its
      recorded candidate status unless a true mirror appears)
- [ ] Final re-scan: the worklist equals the recorded goal candidates
      exactly; gates green

**Candidate goals:**
- AT-097 — recorded goal candidate from ticket 02; confirm and keep.


**Micro-tasks:** (each ends in one commit and one push)

1. Micro — audit the first seven listed IDs and confirm the recorded candidate stays
   - nano: extract the listed negative cases from OBLIGATIONS.md
   - nano: locate one mirror test per ID (grep by concept, read both sides)
   - nano: apply cite edits only at verified mirrors; never force a match
   - nano: cargo fmt --check + ticket-status, then commit and push
2. Micro — audit the last listed ID and produce the final worklist proof
   - nano: extract the final negative case
   - nano: re-run the full AT report and save the worklist proof
   - nano: fmt + ticket-status, then commit and push
3. Micro — close the AT-parity goal
   - nano: run the batch Verify command
   - nano: record goal candidates under Candidate goals (never silent)
   - nano: flip Status, tick every box, commit and push


## Comments

Grill: `.scratch/t066-at-citation-parity/grill.md`. Split 2026-10-02
(ticket 06 of the four-batch split; closes the AT-parity goal).
