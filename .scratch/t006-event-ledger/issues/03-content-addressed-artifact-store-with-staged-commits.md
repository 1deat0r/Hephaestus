# 03: Content-addressed artifact store with staged commits

**What to build:** Bytes can be written out of band, committed atomically under their own sha256 address, referenced by a ledger event only after the commit succeeds, read back with digest verification, and abandoned staging entries garbage-collected — without ever exposing a half-written object.

**Blocked by:** 01 (event ledger).

**Status:** ready-for-agent

- [x] Staged files live outside the object namespace and become addressable only via one atomic rename (AT-057 / R-057)
- [x] The referencing ledger event is appended only after the commit rename succeeds — enforced by API shape (commit takes no ledger) and exercised by the crash-boundary tests; the ordering is a documented caller convention, not a type-system guarantee (limitation recorded in the spec)
- [x] Reads re-hash the bytes and fail loudly on digest mismatch
- [x] Crash between staging and rename: reopen shows no object; the stale staging entry is sweepable (AT-057 / R-057)
- [x] Crash between rename and referencing event: object retained (content-addressed orphan, safe), state recoverable (AT-057 / R-057)
- [x] `gc` removes only never-committed staging entries past the cutoff; committed objects are never candidates
- [x] Reuse of the existing digest primitive; zero new dependencies

## Comments

2026-09-30 — Done. Verified: pre-commit non-addressability, digest-mismatch and malformed-digest refusal, both crash boundaries with a real GC sweep and post-gc orphan read (AT-057/R-057). Suite green; `make ci` exit 0.
