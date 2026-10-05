# 05: Triage batch B3 — advisory, backend, and evaluation ATs (16)

**What to build:** The same read-both-sides audit for this batch.

**ATs:** AT-053 AT-054 AT-064 AT-065 AT-066 AT-067 AT-068 AT-069
AT-070 AT-071 AT-072 AT-074 AT-075 AT-077 AT-078 AT-079

**Blocked by:** None (run after 04).

**Status:** done
**Covers:** 3

**Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/05-b3-advisory-backend-eval-at-batch.md

- [x] All 16 dispositioned: cited or goal candidate
- [x] Re-scan: batch absent; gates green

**Candidate goals:** (none -- all 16 cited at verified mirrors; record closed — an entry with no honest mirror
is disposed as a goal candidate only at close, because an open ticket
must stay inside the 16 scope-ID cap)


**Small tasks:** (max 8)

1. [x] **S1** — cite the first half of this batch
   **Status:** done
   **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/05-b3-advisory-backend-eval-at-batch.md --min-disposed 8
   **Micro-tasks:** (max 6)
   1. [x] **M1** — cite group 1 of half 1
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/05-b3-advisory-backend-eval-at-batch.md --min-disposed 4
      - [x] extract the negative cases for this group from OBLIGATIONS.md
      - [x] locate one mirror test per ID; read both sides
      - [x] apply cite edits only at verified mirrors
      - [x] run this micro's Verify until green
   2. [x] **M2** — cite group 2 of half 1
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/05-b3-advisory-backend-eval-at-batch.md --min-disposed 8
      - [x] extract the negative cases for this group from OBLIGATIONS.md
      - [x] locate one mirror test per ID; read both sides
      - [x] apply cite edits only at verified mirrors
      - [x] run this micro's Verify until green

2. [x] **S2** — cite the second half of this batch and close it
   **Status:** done
   **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/05-b3-advisory-backend-eval-at-batch.md --min-disposed 16
   **Micro-tasks:** (max 6)
   1. [x] **M1** — cite group 1 of half 2
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/05-b3-advisory-backend-eval-at-batch.md --min-disposed 12
      - [x] extract the negative cases for this group from OBLIGATIONS.md
      - [x] locate one mirror test per ID; read both sides
      - [x] apply cite edits only at verified mirrors
      - [x] run this micro's Verify until green
   2. [x] **M2** — cite group 2 of half 2
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/05-b3-advisory-backend-eval-at-batch.md --min-disposed 16
      - [x] extract the negative cases for this group from OBLIGATIONS.md
      - [x] locate one mirror test per ID; read both sides
      - [x] apply cite edits only at verified mirrors
      - [x] run this micro's Verify until green
   3. [x] **M3** — close the batch
      **Verify:** python3 tools/report_at_citations.py --batch .scratch/t066-at-citation-parity/issues/05-b3-advisory-backend-eval-at-batch.md
      - [x] run the batch Verify command
      - [x] record any goal candidates under Candidate goals
      - [x] flip Status to done and tick the acceptance boxes
      - [x] run cargo fmt --check and make ticket-status


## Comments

Grill: `.scratch/t066-at-citation-parity/grill.md`. Split 2026-10-02
(ticket 05 of the four-batch split).

**S1 dispositions (half 1):** eight entries disposed, all at verified
mirrors read on both sides — advisory_provider (calibration record
absent, advisory authority limited, unknowns abstain), backend_adapter
(local slice with no distributed service, contract clauses, no
compatibility claim), sandbox_run (local workflow reports the actual
budget spent), dossier_export (tool-version change → environment
mismatch), workspace_core (persisted progress, cursor recovery,
identical local and gateway authorization), learning_reuse_gates
(altered source hash invalidates reuse — the reuse-key half of entry
054).

**S2 dispositions (half 2):** eight entries cited at verified mirrors read on both sides -- champion/challenger evaluation (self-favorable evaluation refused, permission-expanding promotion denied, guardrail-regression rollback with provenance), campaign baselines (unequal model/tool access flagged invalid, judge-score/novelty/replication fields kept separate), release gates (critical-unresolved blocks scope, finite-suite uncertainty with no universal field), milestone scope (M5+ requirements absent from M0-M4 runtime sets).
