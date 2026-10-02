# 01: Advisory AT-citation report (worklist with negative cases)

**What to build:** `tools/report_at_citations.py` + `make at-coverage`
(advisory, ADR-024 tier): every uncited AT with its R, negative case,
required outcome, and the R's cite sites — the triage worklist in one
command. R-gate untouched.

**Blocked by:** None (can start immediately).

**Status:** done
**Covers:** 1

- [x] `make at-coverage` prints the full worklist (77 ATs) with
      negative cases parsed from OBLIGATIONS
- [x] Advisory exit 0; `make ci` unchanged-green; fmt on the new tool

## Comments

Grill: `.scratch/t066-at-citation-parity/grill.md`.
