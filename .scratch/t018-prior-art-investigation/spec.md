# T-018 spec - two-stage prior-art investigation and claim charts

Status: ready-for-agent

## Problem Statement

The genesis chain produces hypotheses with claims, but nothing checks them
against prior art. MASTER_SPEC §10 requires a two-stage search (fast
rediscovery pass, then claim-level comparison), five allowed conclusions
(no global novelty), verified captured-byte span references, claim charts
with assessed differences, and confidentiality-aware (redacted) queries.
R-106/R-107/R-112 pin these; AT-029-adjacent reporting duties apply.

## Requirements trace

- R-106: deep/wide/counterevidence/near-match qualification on adjudicated
  scoped fixtures (in-repo corpora; no external retrieval).
- R-107: span verification against captured bytes + recorded parser
  transformations, never mutable locators.
- R-112: `validated_solution` vs `validated_candidate` labeling; scoped
  invention candidates keep assessed differences.
- §10:203: conclusions exactly `KNOWN | NEAR_MATCH |
  NO_MATCH_WITHIN_SEARCH_SCOPE | CONFLICTING | UNRESOLVED`.
- §10:205: redacted queries by default; unredacted external submission
  requires explicit disclosure approval.

## Acceptance Criteria

1. Stage-1 fast pass flags clear rediscoveries as `KNOWN` (fixture: known
   mechanism corpus); zero hits never yields novelty language.
2. Stage-2 claim-level comparison produces `NEAR_MATCH` with a difference
   account, `CONFLICTING`, `NO_MATCH_WITHIN_SEARCH_SCOPE`, or `UNRESOLVED`
   per claim; charts carry per-claim rows (component, mechanism, use
   context, regime, result) with passage references.
3. Span verification: passage references carry source+byte-range+captured
   text; charts refuse to verify when captured text does not match the
   fixture corpus bytes (R-107).
4. Redacted queries: mechanism-specific tokens stripped by default;
   unredacted path gated behind explicit `full_disclosure_approved`
   (default false).
5. Report carries search dates, scopes covered, query families,
   near-matches, missing access (R-029).
6. Labels: rediscovery → `validated_solution`; `validated_candidate`
   requires verified chart + assessed differences (R-112).
7. Twin-run byte-identical reports (determinism).

## Risks

- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
- Fabricated novelty: the five-conclusion enum is closed; no score field
  exists on the report.
