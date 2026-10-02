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


**Small tasks:** (max 8)

1. [ ] **S1** — cite the first half of this batch
   **Status:** ready-for-agent
   **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md --min-disposed 8
   **Micro-tasks:** (max 6)
   1. [ ] **M1** — cite group 1 of half 1
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md --min-disposed 4
      - [ ] extract the negative cases for this group from OBLIGATIONS.md
      - [ ] locate one mirror test per ID; read both sides
      - [ ] apply cite edits only at verified mirrors
      - [ ] run this micro's Verify until green
   2. [ ] **M2** — cite group 2 of half 1
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md --min-disposed 8
      - [ ] extract the negative cases for this group from OBLIGATIONS.md
      - [ ] locate one mirror test per ID; read both sides
      - [ ] apply cite edits only at verified mirrors
      - [ ] run this micro's Verify until green

2. [ ] **S2** — cite the second half of this batch and close it
   **Status:** ready-for-agent
   **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md --min-disposed 16
   **Micro-tasks:** (max 6)
   1. [ ] **M1** — cite group 1 of half 2
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md --min-disposed 12
      - [ ] extract the negative cases for this group from OBLIGATIONS.md
      - [ ] locate one mirror test per ID; read both sides
      - [ ] apply cite edits only at verified mirrors
      - [ ] run this micro's Verify until green
   2. [ ] **M2** — cite group 2 of half 2
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md --min-disposed 16
      - [ ] extract the negative cases for this group from OBLIGATIONS.md
      - [ ] locate one mirror test per ID; read both sides
      - [ ] apply cite edits only at verified mirrors
      - [ ] run this micro's Verify until green
   3. [ ] **M3** — close the batch
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md
      - [ ] run the batch Verify command
      - [ ] record any goal candidates under Candidate goals
      - [ ] flip Status to done and tick the acceptance boxes
      - [ ] run cargo fmt --check and make ticket-status


## Comments

Grill: `.scratch/t066-at-citation-parity/grill.md`. Split 2026-10-02
(ticket 04 of the four-batch split).
