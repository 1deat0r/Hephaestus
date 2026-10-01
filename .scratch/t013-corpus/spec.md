# T-013 spec — corpus ingestion, spans, retrieval, coverage, evidence edges

Status: ready-for-agent

## Problem Statement

M2 discovery cannot ground opportunities without a Knowledge service: today
there is no way to ingest authorized files into captured bytes, cite exact
source spans, run deterministic retrieval, report what was (and was not)
covered, or link evidence with types and provenance (IMPLEMENTATION_PLAN
T-013, R-015/R-016/R-017/R-018/R-098/R-106/R-107).

## Solution

A pure `knowledge` module: a `Corpus` owning captured bytes plus `ingest`,
`verify_span`, `search`, `coverage`, edge linking with shared-origin tallies,
and quarantine/corrections. A `SourceAdapter` trait with a local-file adapter
supplies real bytes through the real parser path. Everything rebuilds
deterministically from captured bytes.

## User Stories

1. As a knowledge caller, I want to ingest an authorized local file into
   captured bytes with sha, locator, validity/ingestion times, and parser
   id/version, so that provenance is exact (R-017).
2. As a knowledge caller, I want byte-offset spans verified against captured
   bytes (not live files), so that tampering and drift fail closed (R-107).
3. As a discovery caller, I want deterministic full-text search with ranked
   spans, so that the same query always yields byte-identical results.
4. As a discovery caller, I want empty results returned as empty with honest
   coverage — never as evidence of anything (R-009 boundary).
5. As a readiness caller, I want per-query and per-corpus coverage reports
   (searched/matched/spans/inaccessible/quarantined), so that policy can
   decide what blocks qualification (R-106).
6. As a genesis caller, I want typed evidence edges (Supports, Contradicts,
   Mentions, SharesOrigin) bound to spans with provenance (R-016).
7. As a genesis caller, I want support tallies counted by distinct origin,
   so that two spans from one source never double-count (R-018).
8. As an auditor, I want quarantine with reason to invalidate incident edges
   without deleting bytes, and corrections to supersede (never mutate), so
   that history stays auditable (R-098/R-017).
9. As an auditor, I want every view (search, coverage, tallies) rebuildable
   from captured bytes alone (R-015).

## Implementation Decisions

- New module `crates/hephaestus/src/knowledge/` with `record` (Corpus,
  CapturedSource, Span, Edge, Coverage types) and `service` (`ingest_bytes`,
  `verify_span`, `search`, `coverage`, `link_edge`, `tally`, `quarantine`,
  `correct`) submodules; registered in `lib.rs`.
- Tokenization: lowercase alphanumeric splitting, fixed and documented;
  score = matched query-token count, ties broken by source id (u64 order).
- `SourceAdapter` trait (`ingest_file(path) -> CapturedSource`); local-file
  adapter only — no network (rule-6(d) consent absent).
- Corrections mint versions (`supersedes`); quarantine flips edge validity;
  originals retained. All transitions pure and twin-run deterministic.
- No ADR (direct plan implementation); no requirements.json change
  (Knowledge Rs exist).

## Testing Decisions

- A good test asserts service behavior at the public seam with exact
  tallies/codes/counts, never internals.
- Red-first at `ingest_bytes`/`verify_span` (01), `search`/`coverage` (02),
  `link_edge`/`tally`/`quarantine` (03).
- Prior art: deny-matrix style (`scheduler_deny.rs`), twin-run determinism
  (mission tests), ledger binding test (mission envelope → BudgetLedger).
- Fixtures: small inline byte strings plus real repo files through the local
  adapter (e.g. GLOSSARY excerpts) — real bytes, no network.

## Out of Scope

Network adapters, embeddings/vector search, graph servers, LLM summarization,
discovery operators (T-014+), Context Compiler packets, novelty claims from
retrieval, persistence into ledger/artifact store (caller owns it).

## Further Notes

- GLOSSARY gains corpus terms (row-first, with ticket 03 or earlier).
- New source files staged + allowlisted with seal regeneration at landing
  (ADR-024 standing procedure).
