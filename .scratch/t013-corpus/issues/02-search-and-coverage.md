# 02: Deterministic search and coverage reports

**What to build:** Token-overlap full-text search with ranked spans and
honest per-query/per-corpus coverage; empty results stay empty with coverage
attached (R-106, R-015, R-009 boundary).

**Blocked by:** 01 (ingested sources with spans must exist first).

**Status:** done

- [x] `search` ranks by matched-token count, ties by source id; returns spans (red-first seam)
- [x] Twin-run byte-identical results for representative queries
- [x] Empty query returns empty hits + coverage showing searched count (never a claim)
- [x] Per-query coverage: searched/matched/spans/inaccessible/quarantined counts exact
- [x] Per-corpus coverage: documents/bytes/quarantined/superseded counts exact
- [x] Views rebuild from captured bytes alone (rebuild test: drop derived state, recompute, compare)

## Comments
Done 2026-10-01: token-overlap search + coverage; 6/6 in `corpus_search.rs`. Process note: search code pre-existed its tests (5/6 green first run — regression coverage); the 1 red was a stopword collision in test data ("in" genuinely matches), fixed honestly.
