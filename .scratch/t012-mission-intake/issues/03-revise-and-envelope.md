# 03: Revise/versioning, impact report, budget-envelope binding

**What to build:** Changes to goal, cost limit, dataset policy, or quality
margin mint a new Mission version with a changed-sections impact report, and
the resource envelope binds to T-008 minor-unit money with explicit unknown
prices (R-012).

**Blocked by:** 01 (record + compile seam must exist first).

**Status:** done

- [x] `revise` on each of the four change classes → version n+1, supersedes n (red-first at `revise` seam)
- [x] ImpactReport lists exactly the changed sections, no more
- [x] Override of an inferred requirement flows through `revise` (new version, provenance kept)
- [x] Resource envelope uses T-008 minor-unit money + currency; unknown price → explicit unknown, never zero
- [x] Same revise applied twice → byte-identical Mission (determinism)
- [x] GLOSSARY mission-compiler terms added (row-first, before code lands)

## Comments
Done 2026-10-01: `revise` + `MissionChange`/`ImpactReport`/`ReviseError`; clippy large-enum-variant fixed via `Box<Mission>`; 8/8 in `mission_revise.rs` incl. ledger binding. GLOSSARY +3 (Mission, Value frame, Authorization request).
