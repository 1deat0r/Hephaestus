# 01: Gated requirement-citation coverage with a sealed reasoned allowlist

**What to build:** `make req-coverage` wired into `ci`: a code-scope
scan of every R-id against requirements.json that fails on an
untriaged citation gap and on a stale allowlist entry, with the
allowlist sealed and its two entries reasoned (R-054 N/A-until-cache,
R-079 build-order process); five honest cite retrofits clear the
rest; ADR-027 records the decision — future derivations read this
report instead of re-grepping.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] `make req-coverage` green with exactly 2 allowlisted entries
- [x] Stale detection demonstrated (cite an allowlisted id → FAIL →
      revert → green)
- [x] R-081/R-085/R-087/R-088/R-089 retrofits landed
- [x] `ci` includes req-coverage; gate seal covers the allowlist;
      ADR-027 recorded; `make ci` + `make doc-check` green

## Comments

Grill: `.scratch/t059-requirement-coverage-report/grill.md`.
Spec: `.scratch/t059-requirement-coverage-report/spec.md`.
