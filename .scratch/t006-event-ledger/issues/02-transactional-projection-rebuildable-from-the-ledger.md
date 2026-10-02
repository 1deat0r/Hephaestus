# 02: Transactional projection rebuildable from the ledger

**What to build:** A derived per-mission timeline view that is written only after its source event is durable, and that can be deleted outright and rebuilt from the ledger with no loss — proving retrieval views are never authoritative.

**Blocked by:** 01 (append-only event ledger).

**Status:** done

- [x] Projection updates happen strictly after the event append is durable (append-then-project)
- [x] Deleting the projection file and rebuilding from the ledger reproduces an identical view (AT-015 / R-015)
- [x] Simulated death between event append and projection write leaves a recoverable state: rebuild restores the view (AT-015 / R-015)
- [x] A projection is never readable as truth ahead of the ledger — views are constructed only by rebuilding from the ledger or by loading a previously persisted view; `write` does not take a ledger, so this holds by API shape plus documented convention, not by the type system (limitation recorded in the spec)
- [x] Integration tests assert only external behavior at the public ledger-module seam; small in-module unit tests cover internal edge helpers (per spec Testing Decisions)

## Comments

2026-09-30 — Done. Verified: projection rebuild byte-identical after deletion from a disk reopen (AT-015/R-015), append-then-project stale-view recovery, in-module unit tests. `cargo test -p hephaestus --test event_ledger` green (19 cases); `make ci` exit 0.

Closed 2026-10-02 — work landed earlier; verified green: event ledger, projection, and artifact-store tests green in make ci.
