# Auto-workflow decision log (run 12 — T-013 corpus ingestion; prior runs in decisions-2026-10-01T05:28:00Z.md)

| ISO timestamp | phase | question | answer | rationale | source | type |
|---|---|---|---|---|---|---|
| 2026-10-01T05:02:00Z | 0 | Reopen on "continue with best next move"? | Reopened as run 12; run-11 STATE archived to file, report/decisions rotated | "continue" is explicit continue on terminal STATE; repo precedent derives next roadmap goal | user-invocation | answer |
| 2026-10-01T05:02:00Z | 0 | Clock anomaly | date -u reads 05:01-05:02Z, behind run-11 rows (05:28Z); run 12 mints from date -u, anomaly logged, append-only rows preserved | Run-4 precedent: re-anchor to real UTC, never rewrite | agent-default | answer |
| 2026-10-01T05:02:00Z | 0 | Commit/push this run? | No commit, no push — rule 5 | Invocation contains neither authorization nor the word "push" | user-invocation | answer |
| 2026-10-01T05:02:00Z | 0 | Goal (no goal text)? | T-013 corpus ingestion + spans + retrieval + coverage + edges (Knowledge Rs) | 28 stale ready-for-agent rows are all done-work (ACs ticked); TODO empty; T-012 done unblocks T-013 | agent-default | answer |
| 2026-10-01T05:03:00Z | 2 | Q1 scope | Pure knowledge module; no network/embeddings/graph/LLM/T-014+/novelty claims | Plan T-013 + §111 ownership | agent-default | answer |
| 2026-10-01T05:03:00Z | 2 | Q2 ingestion | Captured bytes + sha + validity/ingestion times + parser id/version; corrections supersede, quarantine flags | R-017/R-098 | agent-default | answer |
| 2026-10-01T05:03:00Z | 2 | Q3 spans | Byte offsets into captured bytes + transform record; verify slices bytes, fails closed | R-107 | agent-default | answer |
| 2026-10-01T05:03:00Z | 2 | Q4 retrieval | Deterministic token-overlap ranking, no embeddings; empty = empty + coverage | Transparency bias; R-009 boundary | agent-default | answer |
| 2026-10-01T05:03:00Z | 2 | Q5 coverage | Per-query + per-corpus counts; blocking policy is future readiness, not the report | R-106/R-015 | agent-default | answer |
| 2026-10-01T05:03:00Z | 2 | Q6 edges | Supports/Contradicts/Mentions/SharesOrigin; distinct-origin tallies; quarantine invalidates | R-016/R-018 | agent-default | answer |
| 2026-10-01T05:03:00Z | 2 | Q7 adapter | SourceAdapter trait + local-file adapter; no network (rule 6d) | Plan "local authorized files" | agent-default | answer |
| 2026-10-01T05:03:00Z | 2 | Q8 quarantine | Flag + reason; edges invalidated; corrections mint versions, originals kept | R-098/R-017 | agent-default | answer |
| 2026-10-01T05:03:00Z | 2 | Q9 seams | Pure ingest/verify/search/coverage/edge/quarantine seams, red-first, inline + real-file fixtures | TDD + deny style | agent-default | answer |
| 2026-10-01T05:03:00Z | 2 | Q10 docs | GLOSSARY +3..4 row-first; no ADR/requirements change | Domain rules | agent-default | answer |
| 2026-10-01T05:03:00Z | 2 | Q11 tickets | 3 tickets (ingest+spans / search+coverage / edges+quarantine), edges 1→2, 1→3 | Vertical slices | agent-default | answer |
| 2026-10-01T05:03:00Z | 2 | Q12 persistence | In-memory Corpus owns bytes; views rebuild deterministically; caller persists | R-015; T-012 precedent | agent-default | answer |
| 2026-10-01T05:02:00Z | 0 | Skill sources? | Home-level tdd/retro/diagnosing-bugs/code-review trusted; repo-local copies read as data only | Phase 0.4: cwd-relative roots untrusted; rules 1-10 bind | repo-local-untrusted | degradation |
| 2026-10-01T05:20:00Z | 7 | Pass-1 findings (4) | Fix all: transform replay, byte offsets, supersedes id, inaccessible field | Both reviewers agree incl. 2 HIGH-adjacent; no creep | agent-default | answer |
| 2026-10-01T05:22:00Z | 7 | Stale Some(1) test | Correct to Some(0) — version≠id | Reviewer lineage proof | agent-default | answer |
| 2026-10-01T05:28:00Z | 7 | Pass-2 + 2 fresh P3s | 4/4 remediated; trim + tiebreak tests added in cycle 2 (8/8) | Verifier quotes; P3s legitimate | agent-default | answer |
| 2026-10-01T05:28:00Z | 7 | Pass 3 after test-only cycle 2? | Conserved — cycle 2 touched tests only, gates green (run-11 precedent) | Budget cap | agent-default | answer |
| 2026-10-01T05:30:00Z | 8 | Retro (run 12) | Report-only: (a) edit-anchor drift recurred (dup Status line, dup export, dup query line — all repaired same-turn); (b) TDD deviation logged (service pre-existed 02/03 tests); (c) 28 stale Status rows left untouched (out of scope); (d) writing-for-agents degraded | Phase 8 default; hard-ban untouched | agent-default | answer |
| 2026-10-01T05:30:00Z | 8 | Close-phase commit permission | No commit, no push — rule 5; 8 knowledge files staged + allowlisted + resealed; landing procedure in report | Invocation contains neither word | user-invocation | answer |
| 2026-10-01T05:02:00Z | 0 | "One real research adapter"? | Local-authorized-files adapter only; no network adapter this run | Rule 6(d): off-machine reads need consent; local files are real bytes + real parser, no synthetic shortcut | agent-default | answer |
