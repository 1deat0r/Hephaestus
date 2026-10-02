# T-059 spec - requirement coverage report

Status: ready-for-agent

## Problem Statement

Goal derivation runs a coverage scan of R-ids against implementation
code. The ad-hoc scope was wrong in both directions: too narrow
(crates+python → phantom gaps re-audited every run) or tautological
(requirements data cites itself). Nothing gates NEW requirements from
landing uncited, and nothing catches a stale "known gap".

## Requirements trace

- Directly serves R-081's spirit (every requirement's citation
  signal stays honest) and the workflow's derivation quality; no new
  R obligation. Enforcement precedent: ADR-024 tier registry.
- ADR-027 records the decision (RUNTIME_DECISIONS.md, verified
  outside MANIFEST).

## Design

- `tools/check_requirement_citations.py`: code-scope scan
  (crates/**.rs, python/**.py, tests/**, tools/*.py,
  validation/*.py, .githooks/*) vs requirements.json; allowlist
  `tools/requirement_citations.txt` (`R-XXX # reason`); fail on
  untriaged gaps AND stale allowlist entries; counts on PASS.
- `gate_seal.py` EXTRA_DATA gains the allowlist (sealed).
- Makefile: `req-coverage` target; `ci` list gains `req-coverage`
  (ADR-027).
- Five honest cite retrofits: R-081 (verify_package), R-087
  (commit-msg hook), R-085 (release record doc), R-088 (corpus
  ingest/review headers), R-089 (method_registry header).
- Allowlist final content: R-054 (N/A-until-cache, reasons), R-079
  (build-order process obligation, reasons).

## Acceptance Criteria

1. `make req-coverage` green with exactly 2 allowlisted gaps.
2. Stale detection: citing an allowlisted id makes the tool FAIL
   until the entry is removed (demonstrated once, reverted).
3. Five retrofits landed; scan shows no other uncited ids.
4. `make ci` (now including req-coverage) + `make doc-check` green;
   gate seal covers the new allowlist.
5. ADR-027 recorded in RUNTIME_DECISIONS.md.

## Out of Scope

Requirements-data scanning (tautology), glossary rows, commit (rule
5).

## Further Notes

Grill: `.scratch/t059-requirement-coverage-report/grill.md`.
