# 06: Triage batch B4 — requirements, release, and self-improvement ATs (9)

**What to build:** The same read-both-sides audit for this batch.

**ATs:** AT-081 AT-085 AT-086 AT-087 AT-088 AT-089 AT-090 AT-093
AT-097 (AT-097 is already a recorded goal candidate from ticket 02 —
confirm and keep, do not silently cite)

**Blocked by:** None (run after 05).

**Status:** done
**Covers:** 3, 4

**Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/06-b4-requirements-release-at-batch.md

- [ ] All 9 dispositioned: cited or goal candidate (AT-097 keeps its
      recorded candidate status unless a true mirror appears)
- [ ] Final re-scan: the worklist equals the recorded goal candidates
      exactly; gates green

**Candidate goals:**
- AT-097 — recorded goal candidate from ticket 02; confirm and keep.
- AT-086 — no review-report exporter exists in code scope: nothing exports a package review report, so no test can exercise the single-authoring-assistant plus check-scope scenario (nearest is tools/check_audit_age.py, which names R-086 for the human-led audit process but asserts cadence only) -> goal candidate, never a silent skip.
- AT-087 — the commit-msg hook (.githooks/commit-msg) enforces a recorded ADR decision for gate changes, but no test executes the hook and no build path asserts decision-plus-traceability for a backend-to-mandatory change -> goal candidate, never a silent skip.


**Small tasks:** (max 8)

1. [x] **S1** — cite the first half of this batch
   **Status:** ready-for-agent
   **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/06-b4-requirements-release-at-batch.md --min-disposed 5
   **Micro-tasks:** (max 6)
   1. [x] **M1** — cite group 1 of half 1
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/06-b4-requirements-release-at-batch.md --min-disposed 3
      - [x] extract the negative cases for this group from OBLIGATIONS.md
      - [x] locate one mirror test per ID; read both sides
      - [x] apply cite edits only at verified mirrors
      - [x] run this micro's Verify until green
   2. [x] **M2** — cite group 2 of half 1
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/06-b4-requirements-release-at-batch.md --min-disposed 5
      - [x] extract the negative cases for this group from OBLIGATIONS.md
      - [x] locate one mirror test per ID; read both sides
      - [x] apply cite edits only at verified mirrors
      - [x] run this micro's Verify until green

2. [ ] **S2** — cite the second half of this batch and close it
   **Status:** ready-for-agent
   **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/06-b4-requirements-release-at-batch.md --min-disposed 9
   **Micro-tasks:** (max 6)
   1. [ ] **M1** — cite group 1 of half 2
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/06-b4-requirements-release-at-batch.md --min-disposed 7
      - [ ] extract the negative cases for this group from OBLIGATIONS.md
      - [ ] locate one mirror test per ID; read both sides
      - [ ] apply cite edits only at verified mirrors
      - [ ] run this micro's Verify until green
   2. [ ] **M2** — cite group 2 of half 2
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/06-b4-requirements-release-at-batch.md --min-disposed 9
      - [ ] extract the negative cases for this group from OBLIGATIONS.md
      - [ ] locate one mirror test per ID; read both sides
      - [ ] apply cite edits only at verified mirrors
      - [ ] run this micro's Verify until green
   3. [ ] **M3** — close the batch
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/06-b4-requirements-release-at-batch.md
      - [ ] run the batch Verify command
      - [ ] record any goal candidates under Candidate goals
      - [ ] flip Status to done and tick the acceptance boxes
      - [ ] run cargo fmt --check and make ticket-status


## Comments

Grill: `.scratch/t066-at-citation-parity/grill.md`. Split 2026-10-02
(ticket 06 of the four-batch split; closes the AT-parity goal).
