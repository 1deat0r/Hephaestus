# T-066 grill - AT-scope citation parity (extend the ADR-027 gate's view)

Goal: the R-signal became true (t059); the AT-signal did not — 77 ATs
are cited nowhere in code scope (fresh scan: AT-001..AT-119 with the
session's AT-cites already excluded). The scan is ad-hoc again = the
exact disease ADR-026/027 cured for R-ids.

## Facts

- Fresh scan: 77 gaps, identical to the run-30 list (no new AT cites
  landed since; test NAMES like `at_034_...` do not match the
  case-sensitive `AT-\d{3}` pattern — only literal citations count).
- OBLIGATIONS.md holds, per AT, a negative case + required outcome —
  the raw material for honest triage (is there a test that MIRRORS it?).
- Precedents: R-citation tool (fail-closed, sealed allowlist,
  stale-detection); advisory reports exist (ADR-024 tier:
  report-unreferenced); run-24's15 R retrofits show the honest cite
  pattern (site where the obligation is actually exercised).
- ATs split roughly: (a) mirrored by existing tests (cite at the
  mirror), (b) doc/process-flavoured positives with no code mirror
  (reasoned entry or future goal), (c) genuinely untested negatives
  (SURFACE AS GOALS — never allowlisted silently).

## Q1 - Tool shape?
**A:** `tools/report_at_citations.py` + `make at-coverage` target,
**advisory (NOT in `ci`)** per ADR-024's advisory tier: pure report —
for each uncited AT prints its R, the OBLIGATIONS negative case,
required outcome, and the R's cite sites (triage hint). Exit 0 always;
the gate discipline that matters (no silent gap-hiding) arrives when
the open list drains and the target graduates into `ci` (recorded as
the follow-up, not done speculatively). R-gate untouched. (agent-default)

## Q2 - Allowlist for ATs?
**A:** None in v1 — an allowlist without per-AT audits would invite
the fabrication this whole line of work exists to prevent. Triage
lands as CODE CITES (mirrored) or as GOAL TICKETS (gaps); the report
is the worklist. (agent-default)

## Q3 - Triage batches this run?
**A:** Tickets split by obligation family so each batch is auditable in
one context: **02 = amendment ATs (AT-094..AT-119)** — their tests
exist from T-035..T-046-era work (audit which mirror which; cite
honestly, remainder → goal list); **03 = core ATs (the rest)** — same
method. Each ticket ends with a re-scan showing its batch drained to
cites or explicitly recorded goal candidates (goals recorded in the
ticket + decisions rows, not in a pass/fail gate). (agent-default)

## Q4 - Honest edge: ATs whose negative case tests DO exist but under
different wording?
**A:** Cite only after reading BOTH the OBLIGATIONS negative case and
the candidate test (the run-24 method); when the test exercises a
NEIGHBOURING property, it is NOT a mirror — record as goal candidate.
No name-matching shortcuts. (agent-default)

## Q5 - Scope?
**A:** no commit (rule 5); `at-coverage` stays advisory until the open
list is drained (graduation = future decision row); R-gate semantics
untouched. (agent-default)
