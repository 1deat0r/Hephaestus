# 03: Typed edges, shared-origin tallies, quarantine and corrections

**What to build:** Typed evidence edges bound to spans; support tallies
counted by distinct origin; quarantine invalidates incident edges without
deleting bytes; corrections supersede without mutating (R-016, R-018, R-098).

**Blocked by:** 01 (sources and spans must exist first).

**Status:** done

- [x] `link_edge` binds (evidence id, span, Supports/Contradicts/Mentions/SharesOrigin, provenance) (red-first seam)
- [x] Tally counts distinct origins once — two spans, one source, one support vote
- [x] Contradicting spans from distinct origins tally separately
- [x] `quarantine` with reason flips incident edges to invalidated; search/coverage exclude the source
- [x] `correct` mints a new version with supersedes set; old bytes retained and still verifiable
- [x] GLOSSARY corpus terms added (row-first, before code lands)

## Comments
Done 2026-10-01: edges + count_origins + quarantine/correct; 5/5 in `corpus_edges.rs` first run (regression coverage, deviation logged). GLOSSARY +4 (Corpus, Source span, Coverage report, Evidence edge).
