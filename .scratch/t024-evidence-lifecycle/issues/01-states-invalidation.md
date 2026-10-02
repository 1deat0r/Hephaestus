# 01: State machines + invalidation traversal

**Status:** done

**What to build:** `lifecycle` module: OpportunityState + HypothesisState
enums with `advance` transition tables (named illegal transitions),
orthogonal CandidateFields, EvidenceGraph with append-only corrections,
`invalidate` (dependent traversal, stale dossier marks, re-evaluation
queue entries awaiting budget+permission), snapshot ID binding.

**Acceptance:**
- [x] Legal/illegal transitions named (spec AC 1)
- [x] Orthogonal fields (spec AC 2)
- [x] Invalidation traversal + append-only (spec AC 3/4)
- [x] Snapshot binding (spec AC 6)

## Comments
