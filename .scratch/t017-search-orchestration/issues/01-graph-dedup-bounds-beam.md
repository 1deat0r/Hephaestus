# 01: Search graph + dedup + bounds + beam (tracer bullet)

**What to build:** The `orchestrator` module: `SearchNode` (parents,
operator, payload, evidence snapshot, cost estimate, rejection reason),
normalized-key deduplication (paraphrase → same key → expansion stops with
`duplicate_generation`), `SearchBounds` (max_depth/max_candidates/
max_operator_calls — each exceeded stops with a recorded reason), and the
transparent best-first beam (≤ width per wave, sorted by named
deterministic priority: has-discriminator then evidence count). `run(seed,
sources, bounds, beam_width) -> SearchOutcome`.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] `SearchNode` with parents/operator/evidence snapshot/cost/rejection (spec AC 4)
- [x] Dedup: paraphrase stream stops with `duplicate_generation` (AT-031)
- [x] Bounds enforced with per-bound stop reasons (spec AC 2)
- [x] Beam keeps ≤ width per wave; non-kept → archive with reasons (spec AC 3)
- [x] Red-first at the orchestrator seam; twin-run byte-identical (spec AC 7)

## Comments

## Comments
Done 2026-10-01T06:56Z: 7/7 in `search_orchestrator.rs`. Fixture bugs fixed honestly: (1) children restating the seed collide with the seed dedup key — correct behavior, reworded fixture; (2) duplicate bound needs 3 strikes — fixture supplies 3 paraphrase children. Archive is a view: all children are nodes; non-kept are archived with reasons.
