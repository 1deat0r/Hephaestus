# T-013 grill — corpus ingestion & retrieval (self-interview, 2026-10-01)

Mode: `grill-with-docs`/`grilling` self-interview (hard rule 1 — no user questions).
Sources: IMPLEMENTATION_PLAN T-013, Knowledge Rs (R-015/016/017/018/098/106/107),
`docs/RETRIEVAL_AND_MEMORY.md`, MASTER_SPEC:111 (Knowledge owns acquisition,
normalization, graph views, provenance) / :136 / :357, T-002 Source/Evidence
contracts. Repo-local skill texts read as data only.

## Q1 — Scope: what is T-013 and what is out?
**A:** A pure `knowledge` module: ingest authorized local files into captured
bytes + metadata; byte-offset spans with parser-transformation records;
deterministic full-text search; per-query/per-corpus coverage reports; typed
evidence edges with shared-origin dedup; quarantine + corrections. OUT:
network adapters of any kind, embeddings/vector search, graph servers,
LLM summarization, discovery operators (T-014+), Context Compiler packets
(downstream consumer, §357), any novelty claim from retrieval (R-009 lives in
T-018; T-013 reports coverage, never claims).

## Q2 — Ingestion shape: what is captured?
**A:** Per document: captured bytes (immutable), sha256, locator (path),
validity time + ingestion time (R-017), parser id/version, access scope.
Corrections arrive as new versions superseding old ones — originals are never
mutated. Quarantine marks a source + reason without deleting bytes (R-098).

## Q3 — Spans: how verified (R-107)?
**A:** Span = {source id, byte start/end into captured bytes, text, parser,
transform list}. `verify_span` slices the captured bytes and compares to the
stored text; mismatch (tampered bytes, wrong offsets, unrecorded transform)
fails closed. Never resolved against live files or URLs.

## Q4 — Retrieval: what algorithm?
**A:** Deterministic token-overlap ranking over normalized text (lowercase,
alphanumeric tokens): score = matched query tokens, ties by source id. No
embeddings, no learned weights — transparent and twin-run identical, in the
spirit of the plan's "transparent beam/best-first" bias. Empty results are
returned as empty with coverage, never as novelty evidence.

## Q5 — Coverage reports: what do they say?
**A:** Per query: documents searched, documents matched, spans returned,
inaccessible/quarantined excluded counts. Per corpus: document count, byte
total, quarantined count, superseded count. Blocking policy belongs to the
mission's readiness policy (future), not to the report.

## Q6 — Evidence edges: which types, what dedup (R-016/R-018)?
**A:** Edge types: Supports, Contradicts, Mentions, SharesOrigin. Edges bind
(evidence id, span id, type, provenance). Shared-origin rule: support tallies
count distinct origins once — two spans from one source do not double-count.
Quarantined sources invalidate incident edges (flagged, excluded from tallies).

## Q7 — Adapters: what is "one real research adapter"?
**A:** A `SourceAdapter` trait plus a local-file adapter reading real
authorized files (md/txt, real bytes through the real parser path). No
network adapter: off-machine reads need rule-6(d) consent this run does not
have. Local files satisfy "initial data can be local authorized files".

## Q8 — Quarantine/corrections semantics (R-098/R-017)?
**A:** `quarantine(source, reason)` flags + records reason; incident edges
flip to invalidated; coverage counts move the source to quarantined.
`correct(source, new_bytes)` mints a new version (supersedes set), old bytes
retained for audit. Both are pure state transitions on the Corpus, twin-run
deterministic.

## Q9 — Seams, red-first, fixtures?
**A:** Pure seams: `ingest_bytes`, `verify_span`, `search`, `coverage`,
`link_edge`/`tally`, `quarantine`/`correct`. Red-first per ticket. Fixtures
are small inline byte strings + a few real repo files (e.g. GLOSSARY terms)
read through the adapter — deterministic, no network, no large downloads.
Deny-style: tampered bytes fail verify; quarantined sources excluded from
search and tallies; empty query returns empty + honest coverage.

## Q10 — Docs obligations?
**A:** GLOSSARY +3..4 (Corpus, Source span, Coverage report, Evidence edge —
row-first). No ADR (direct plan implementation). requirements.json untouched
(Knowledge Rs already exist).

## Q11 — Ticket split?
**A:** Three tracer bullets: 01 ingest + spans + verify (R-017/R-107);
02 search + coverage (R-106/R-015); 03 edges + dedup + quarantine/corrections
(R-016/R-018/R-098). Edges 1→2, 1→3.

## Q12 — Persistence?
**A:** None in T-013. `Corpus` is an in-memory owner of captured bytes;
views (search results, coverage, tallies) rebuild deterministically from it
(R-015: rebuildable views). Callers persist via ledger/artifact store.
