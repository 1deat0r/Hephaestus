# 01: Append-only event ledger with hash chain and recovery

**What to build:** A control-plane ledger that turns every authoritative state change into a durable, tamper-evident history: a validated contract event is appended as one JSON line, chained by sha256 to the previous line, flushed and synced; opening the ledger verifies the whole chain and recovers from a torn trailing line without losing prior history.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] Append refuses any event failing contract validation or the ledger's own sequence/chain checks, and writes nothing when it refuses (AT-013 / R-013)
- [x] Appended events persist across reopen with the chain intact (AT-014 / R-014)
- [x] `open` detects a broken chain or sequence gap and reports the 1-based line number instead of replaying past it
- [x] A torn final line (crash mid-append) is detected, trimmed to the last complete event, and reported as recovered — prior history intact (AT-057 groundwork / R-057)
- [x] Reads are ordered by `sequence` and support cursor-style iteration from a position
- [x] Zero new dependencies; deny-first tests in the crate's tests directory

## Comments

2026-09-30 — Done. Verified: validation refusal writes nothing (AT-013), reopen persistence (AT-014), sha256 chain with tamper detection, sequence invariants on append and on open (errors name the 1-based line number), torn-tail recovery plus newline-less-EOF normalization, cursor reads. Suite green; `make ci` exit 0.

Closed 2026-10-02 — work landed earlier; verified green: event ledger, projection, and artifact-store tests green in make ci.
