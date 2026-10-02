# 01: Add parallel docs job to CI (ADR-024 step 10)

**What to build:** Every push runs rustdoc with warnings denied as its own
CI job, parallel to the existing gates job, so doc-rot turns the check red
without slowing local `make ci` or CI wall-time.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] CI workflow gains a `docs` job running `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`, same triggers, parallel to `gates`
- [x] Gate seal regenerated so the in-tree `gate-seal` check verifies after the workflow edit
- [x] Exact command exits 0 locally (evidence captured)
- [x] Full `make ci` green after all changes
- [x] Workflow YAML still parses (with an available parser, or structural check if none)
- [x] No changes to Makefile, hooks, or `make ci` composition

## Comments

Closed 2026-10-02 — work landed earlier; verified green: the docs CI job runs green on every push this session (gh run: gates + docs).
