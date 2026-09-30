# Spec — T-006: append-only event ledger, transactional projections, content-addressed artifact storage

Status: ready-for-agent
Goal source: derived:roadmap (IMPLEMENTATION_PLAN.md M1 opener; untrusted derivation recorded in decisions.md)
Grill record: `.scratch/t006-event-ledger/grill.md` (10 questions, all self-answered)

## Problem Statement

The control plane can type and validate contract records but has nowhere durable
to write them: there is no authoritative history of what the system did, no
place to keep bytes that must outlive a process, and no way to reconstruct a
view after state is lost. Until that exists, every later milestone (capability
grants, budget reservations, the task scheduler, recovery) has nothing to
record operations against, and any "the system did X" claim would live only in
an ephemeral conversation — exactly what R-014 forbids.

## Solution

A Rust module in the control-plane crate that owns three primitives:

1. **Event ledger** — an append-only JSONL file where each line is a validated
   contract `Event`, hash-chained (`previous_event_sha256`), flushed and
   synced on append, verified end-to-end whenever the ledger is opened.
2. **Transactional projection** — derived views applied only *after* an event
   is durable, so a crash can never show a view of an event that did not
   persist; any projection can be deleted and rebuilt from the ledger alone.
3. **Content-addressed artifact store** — bytes staged to a temp file, then
   atomically renamed into `objects/<prefix>/<sha256>`; a referencing event is
   appended only after the commit rename. Staging entries that were never
   committed can be garbage-collected; committed objects are immutable and
   never collected.

Process death between any two persistence boundaries leaves a state that the
next `open` recovers from: history intact, projection rebuildable, staging
sweepable.

## User Stories

1. As a control-plane operation, I want every authoritative state change
   recorded as a validated `Event` line, so that R-013 is satisfied at the
   storage layer rather than by convention.
2. As a future milestone (grants, budgets, scheduler), I want a single append
   API with a defined durability guarantee, so that I do not invent a second,
   divergent persistence mechanism.
3. As a recovery routine, I want the hash chain checked on open, so that a
   tampered or corrupted history is detected instead of silently replayed.
4. As a recovery routine, I want a truncated final line (crash mid-append) to
   be detected and trimmed to the last complete event, so that audit history
   before the crash survives a torn write.
5. As a projection consumer, I want views built only from durable events, so
   that what I read never describes work that did not happen.
6. As a projection consumer, I want to delete a projection file and rebuild it
   from the ledger with no loss, so that AT-015-style index loss is a non-event.
7. As an artifact producer, I want bytes staged out of the object namespace
   first, so that a half-written file can never be addressed by its digest.
8. As an artifact consumer, I want the committed path to be a single atomic
   rename, so that either the whole object exists under its digest or it does
   not exist at all.
9. As an artifact consumer, I want every read to re-hash the bytes against the
   requested digest, so that silent corruption fails loudly at the seam.
10. As an operator, I want `gc` to touch only never-committed staging entries,
    so that the collector cannot become the data-loss bug.
11. As a test, I want crashes simulated by reopening from disk at each
    persistence boundary, so that AT-057 behavior is exercised without
    instrumentation in production code.
12. As a reviewer, I want the requirement IDs (R-013, R-014, R-015, R-057) and
    their acceptance IDs named in the tests, so that traceability claims are
    checkable rather than asserted.
13. As a maintainer, I want zero new dependencies, so that the offline build
    and the gate suite keep working exactly as they do today.
14. As a developer, I want the ledger module behind one small public surface,
    so that the number of seams the whole test suite reasons about stays at one.
15. As a future T-011 (event-cursor recovery) implementer, I want ordered
    `sequence` values and a cursor-able read API now, so that replay does not
    need a format change later.

## Implementation Decisions

- **Language/location:** Rust, control-plane crate, new `ledger` module
  (grill Q2). No new workspace member, no schema change: the `event` contract
  already exists in `schemas/contracts.schema.json` and generates the typed
  `Event` record.
- **On-disk format:** one JSON `Event` per line (JSONL). Chain rule: line *n*'s
  `previous_event_sha256` is the sha256 of the exact bytes of line *n-1*;
  first event is `null`. `sequence` must be contiguous from 1 within a
  mission's stream. Append = write line + `flush` + `sync_data`.
- **Validation gate:** `Event::validate()` (and the ledger's own chain/sequence
  checks) run *before* a line is written; a rejected event changes nothing.
- **Open = verify:** opening a ledger walks the whole chain; any break,
  unparseable complete line, or sequence gap is an error naming the 1-based
line number.
  Two EOF conditions are recovered rather than rejected: a *torn trailing
  line* (incomplete JSON at EOF) is truncated to the last complete event and
  reported as recovered; a complete final line missing its newline is
  normalized (newline appended) so a later append cannot fuse lines.
- **Projection contract:** append-then-project (grill Q5). Projections are
  derived files; they carry no authority and must be rebuildable from the
  ledger alone. One concrete projection ships: a per-mission timeline index.
- **Artifact store layout:** `staging/` for in-flight writes,
  `objects/<first-2-hex>/<sha256>` for committed content. Commit order:
  stage → hash-verify → atomic rename → append referencing event. Objects are
  immutable; reads re-verify digest. Orphan objects (committed but never
  referenced by an event) are retained — content addressing makes them safe,
  and the repo's unreferenced-file advisory already covers advisory orphans.
- **GC:** explicit call, staging only, age cutoff; committed objects are never
  candidates. Age alone is sufficient (amends grill Q9's extra "digest never in
  the ledger" condition): a successful commit *moves* the file out of staging,
  so anything still there was never committed and cannot be data-loss-bearing.
- **Durability claim scope:** durability is "survives process death and
  machine-appropriate `sync_data`", not "survives kernel panic / power loss
  across all filesystems" — that limitation is stated in the module docs.
- **Dependencies:** none added. Test isolation uses `std::env::temp_dir()` with
  a unique subdir and teardown (grill Q7).
- **Glossary:** add *event ledger*, *projection*, *staged artifact*,
  *content addressing* to `GLOSSARY.md` (verified not gated by MANIFEST, gate
  seal, or generated-document drift).

## Testing Decisions

- Integration tests assert external behavior at the module's public seam only:
  given bytes/events in, what is durable on disk after reopen. No *integration*
  test reaches inside the module. Small in-module unit tests may cover internal
  edge helpers (line splitting, sequence bookkeeping) — that is what "unit
  tests inside the module" below means; they complement, never replace, the
  seam tests.
- **Single primary seam:** the `ledger` module's public API
  (`EventLedger::open/append/events`, projection rebuild, `ArtifactStore
  stage/commit/gc/read`). Unit tests inside the module cover edge validation;
  integration tests in the crate's `tests/` directory cover cross-boundary
  behavior against real temp-dir files (deny-first convention).
- **Bound tests to obligations:** AT-013 (invalid event refused, nothing
  written), AT-014 (after simulated process death, history reconstructable with
  no ephemeral state), AT-015 (delete projection → rebuild → identical view),
  AT-057 (crash injected at each boundary: after event append, between
  projection and its source, after staging, between rename and referencing
  event — reopen shows a recoverable state each time).
- **Prior art:** `crates/hephaestus/tests/artifact_identity.rs` (NIST sha256
  vector, digest parsing), `grant_deny.rs` / `receipt_deny.rs` (deny-first
  structure), `crates/hephaestus/tests/migrate.rs` (reopen/round-trip style).
- **Suite:** `cargo test --workspace`, `cargo clippy --workspace --all-targets
  -- -D warnings`, `cargo fmt --check`, then `make ci` (M0 exit gates per
  grill Q8). Evidence recorded as command + exit code in the state LOG.

## Out of Scope

- Budget reservations and the budget ledger (T-008, R-055).
- Task scheduling, dispatch, retries (T-009/T-011).
- Event-cursor pause/resume/steering UX (R-067/R-068) — the ordered sequence
  is groundwork only.
- Dossier export (R-044), prior-art search, hypothesis machinery.
- Graph/retrieval index rebuild beyond the shipped projection (AT-015 is
  demonstrated, not generalized).
- Any change to `schemas/contracts.schema.json`, `requirements.json`,
  generated contract code, or manifest/gate files.
- AT-057's "retries only permitted operations" clause — retry policy belongs
  to the operation/retry machinery (T-011); T-006 covers the persistence-
  boundary half of AT-057 (recoverable state after crashes at commit points).
- ADR: none — this executes the approved plan; no architectural departure.

## Further Notes

- M0 prerequisite: T-006 depends on M0; verification re-runs the M0 exit gates
  and treats a red gate as a blocker (grill Q8).
- Deliberate omissions are listed in grill Q4 so that "not covered here" reads
  as a decision, not an oversight.
- Known limitations to carry into the report: `sync_data` semantics are
  filesystem-dependent; the tests demonstrate crash-boundary recovery via
  reopen, not power-failure simulation. The store/ledger ordering rule
  (reference event only after commit rename) and the projection's derivation
  rule are API conventions documented on the entry points, not type-system
  guarantees — `Timeline::load` will read a hand-fabricated view file too;
  nothing treats a view as authority, so the worst case is a stale or bogus
  *derived* value that a rebuild from the ledger overwrites.
