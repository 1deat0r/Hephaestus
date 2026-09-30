# 04: End-to-end recovery story, glossary, and gate evidence

**What to build:** The full T-006 story proven in one suite: simulated process death at every persistence boundary leaves a state that reopen recovers — history intact, projection rebuildable, staging sweepable, objects immutable — with the domain vocabulary recorded and the M0 gate evidence captured.

**Blocked by:** 02 (transactional projection), 03 (artifact store).

**Status:** ready-for-agent

- [x] Crash-injection suite covers each boundary: after event append, between append and projection, after staging, between rename and referencing event — reopen recovers each time (AT-057 / R-057)
- [x] After simulated death, evidence and artifacts reconstruct from persisted records alone, with no ephemeral state (AT-014 / R-014)
- [x] GLOSSARY gains: event ledger, projection, staged artifact, content addressing
- [x] `cargo test --workspace`, clippy `-D warnings`, fmt-check green
- [x] `make ci` green (M0 exit gates re-run; a red M0 gate blocks this ticket)
- [x] Test names/comments cite R-013, R-014, R-015, R-057 and their AT IDs; limitations recorded (sync_data filesystem scope; orphan retention)

## Comments

2026-09-30 — Done. Verified: end-to-end death-at-every-boundary suite (AT-014/AT-015/AT-057), GLOSSARY +4 terms (decision rows recorded; domain.md premise corrected), fmt/clippy -D warnings/`cargo test --workspace`/`make ci` all green after each fix cycle. Limitations recorded in spec Further Notes and the workflow report (sync_data scope; conventions-vs-guarantees; retry clause deferred to T-011).
