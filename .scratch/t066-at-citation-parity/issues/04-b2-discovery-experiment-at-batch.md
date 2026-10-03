# 04: Triage batch B2 — discovery, experiment, release ATs (16)

**What to build:** The same read-both-sides audit for this batch.

**ATs:** AT-028 AT-029 AT-030 AT-033 AT-037 AT-038 AT-039 AT-040
AT-041 AT-042 AT-043 AT-044 AT-046 AT-047 AT-048 AT-051

**Blocked by:** None (01 and 02 are done; run after 03 for a clean
sequential frontier).

**Status:** done
**Covers:** 3

**Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md

- [x] All 16 dispositioned: cited or goal candidate
- [x] Re-scan: batch absent; gates green

**Candidate goals:** (none — all 16 cited at verified mirrors)

Residual facets (recorded at close; not goal candidates):
- AT-029: the report's missing-access field is never asserted, and no
  failing-adapter scenario exists in code (the prior-art path has no
  adapter layer).
- AT-037: no test changes an endpoint after sealed data are opened and
  no test mints a new exploratory version — immutability is enforced by
  rejection (digest mismatch).
- AT-046: the lifecycle four-status struct has no test and no single
  load asserts all four statuses together; the two mirror sites cover
  science/engineering/execution and novelty separately.
- AT-039: the dataset and simulator Blocker variants are never
  constructed in tests; exact-variant assertions cover scope and budget
  blockers.


**Small tasks:** (max 8)

1. [x] **S1** — cite the first half of this batch
   **Status:** done
   **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md --min-disposed 8
   **Micro-tasks:** (max 6)
   1. [x] **M1** — cite group 1 of half 1
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md --min-disposed 4
      - [x] extract the negative cases for this group from OBLIGATIONS.md
      - [x] locate one mirror test per ID; read both sides
      - [x] apply cite edits only at verified mirrors
      - [x] run this micro's Verify until green
   2. [x] **M2** — cite group 2 of half 1
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md --min-disposed 8
      - [x] extract the negative cases for this group from OBLIGATIONS.md
      - [x] locate one mirror test per ID; read both sides
      - [x] apply cite edits only at verified mirrors
      - [x] run this micro's Verify until green

2. [x] **S2** — cite the second half of this batch and close it
   **Status:** done
   **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md --min-disposed 16
   **Micro-tasks:** (max 6)
   1. [x] **M1** — cite group 1 of half 2
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md --min-disposed 12
      - [x] extract the negative cases for this group from OBLIGATIONS.md
      - [x] locate one mirror test per ID; read both sides
      - [x] apply cite edits only at verified mirrors
      - [x] run this micro's Verify until green
   2. [x] **M2** — cite group 2 of half 2
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md --min-disposed 16
      - [x] extract the negative cases for this group from OBLIGATIONS.md
      - [x] locate one mirror test per ID; read both sides
      - [x] apply cite edits only at verified mirrors
      - [x] run this micro's Verify until green
   3. [x] **M3** — close the batch
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/04-b2-discovery-experiment-at-batch.md
      - [x] run the batch Verify command
      - [x] record any goal candidates under Candidate goals
      - [x] flip Status to done and tick the acceptance boxes
      - [x] run cargo fmt --check and make ticket-status


## Comments

Grill: `.scratch/t066-at-citation-parity/grill.md`. Split 2026-10-02
(ticket 04 of the four-batch split).

**Batch B2 dispositions (2026-10-03):** 16 ATs cited at verified
mirrors after reading both sides — prior_art (AT-028, AT-029,
AT-030), multi_objective_archive (AT-033), tests/test_qualification.py
(AT-037), experiment_compiler (AT-038, AT-039), semantic_parity
(AT-040, AT-046, AT-048), amendment_gates (AT-041),
result_interpreter (AT-042, AT-046), prototype_worker (AT-043),
dossier_export (AT-044), evidence_lifecycle (AT-047), backend_adapter
(AT-051). No goal candidates. Residual facets recorded above.
