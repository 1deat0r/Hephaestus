# Grill — T-006: event ledger, projections, artifact storage

Self-interview mode (mattpocock-skills-auto-workflow Phase 2): every question the
`grilling` skill would put to the human, answered immediately from codebase
evidence + goal via the auto-answer policy. Round 1 below is the whole frontier
(goal fixed, no upstream decisions open); no round 2 was needed because each
answer was decidable from evidence.

Goal under grill: *Implement T-006 (M1 opener): append-only event ledger,
transactional projections, and content-addressed artifact storage, with tests
bound to requirement/acceptance IDs.*

## Round 1

❓ **Q1 — Scope cut**: Full T-006 (ledger + projections + artifact store + GC +
crash-boundary tests) or a smaller tracer bullet first?

➡️ **Full T-006 triad, bounded.** The wire format already exists (the `event`
contract), the hashing primitive already exists
(`hephaestus::security::artifact::ArtifactDigest`, NIST-vector-tested), and the
workspace already links serde/serde_json/sha2 — so each of the three limbs is
small. What is explicitly *out* of scope: budget reservations (T-008 / R-055),
event-cursor mission control (T-011 / R-068), dossier export (T-018+), any
scheduler (T-009). Runner-up — ledger-only tracer bullet — rejected because it
would defer the riskiest assumption (crash boundaries across commit) to a later
ticket; the plan's own text ties the three limbs together.
*source: agent-default · policy "keep the tracer bullet that proves the riskiest
assumption".*

❓ **Q2 — Language and module location**: Where does T-006 live?

➡️ **Rust, `crates/hephaestus/src/ledger/` (control-plane layer), exposed as
`pub mod ledger`.** IMPLEMENTATION_PLAN: "Use Rust for the authoritative local
runtime, Python for isolated experiment workers." `crates/hephaestus/src/lib.rs`
declares `LAYER = "control-plane"` and says "No scheduler, authorization
service, or scientific analysis runtime yet" — a ledger is control-plane
plumbing, not science. No new crate, no workspace change.
*source: agent-default · plan + lib.rs evidence.*

❓ **Q3 — Storage backend / dependencies**: Files, SQLite, in-memory?

➡️ **Append-only JSONL file, one `contracts::Event` per line, sha256 hash chain;
zero new dependencies.** `Event` already carries `sequence`, `previous_event_sha256`
(`Option<String>`, first event `null`), `payload_sha256`, `operation_id`,
`policy_version` — the ledger's chain fields are contract fields, so the on-disk
format *is* the contract. SQLite would add a native dep (not in `Cargo.lock`,
offline-availability unknown); in-memory alone fails R-014 (durability).
Runner-up: `tempfile` dev-dep — rejected, absent from `Cargo.lock`; tests use
`std::env::temp_dir()` + unique subdir instead.
*source: agent-default · evidence: contracts schema + Cargo.toml/Cargo.lock.*

❓ **Q4 — Requirement / acceptance IDs in scope**: Which obligations do the
tests bind to?

➡️ **In scope: R-013/AT-013** (authoritative state changes via validated
control-plane operations — every ledger append runs `Event::validate()` first),
**R-014/AT-014** (evidence/artifacts survive the loss of ephemeral state —
rebuild/recover from ledger + store after "process death"), **R-015/AT-015**
(retrieval views rebuildable from authoritative records — delete the
projection, rebuild from the ledger), **R-057/AT-057** (recover safely across
dispatch and artifact commit boundaries — crash injected at each persistence
boundary). **Out of scope, recorded so the omission is deliberate:** R-055
(transactional budget reservation → T-008), R-067/R-068 (UI grounding, event
cursors → later tasks), R-044 (dossier export), R-056 (external-effect
reconciliation → T-011).
*source: agent-default · requirements.json + acceptance-tests.json scan.*

❓ **Q5 — Meaning of "transactional projections"**: What is the consistency
contract?

➡️ **Append-then-project; projections are derived and never authoritative.**
Ordering: (1) append event to ledger, flush + `sync_data()`; (2) apply to
projection, persist projection. A crash between (1) and (2) loses at most the
projection, which is rebuildable from the ledger (AT-015) — so no projection
write is ever able to lose primary evidence, and no projection may be read as
truth. Artifact commit ordering: stage bytes → atomic `rename` to
`objects/<aa>/<sha256>` → *then* append the referencing event. Crash before
rename: temp file remains, GC-able. Crash after rename before event: orphan
object — content-addressed and valid, left in place (dedup-safe; the repo's own
unreferenced-file advisory already treats orphans as advisory, not failure).
*source: agent-default · minimal-rework choice; runner-up "projection-first"
rejected because a crash could then show a view of an event that never
durably existed.*

❓ **Q6 — TDD seams**: Where does `tdd` drive?

➡️ **Public entry points of the new module, per policy default:**
`EventLedger::append` / `EventLedger::open` (verify-on-open) / `EventLedger::events`;
`Projection::rebuild` + apply path; `ArtifactStore::stage` / `commit` / `gc`.
Red-green at each seam; the crash-boundary tests are written as deny-first
tests alongside (`crates/hephaestus/tests/` already follows that convention:
`artifact_identity.rs`, `grant_deny.rs`, `receipt_deny.rs`).
*source: agent-default · policy "test seams" row.*

❓ **Q7 — Test infrastructure**: How do tests get isolated workspaces without
new dependencies?

➡️ **`std::env::temp_dir()` + unique subdir (`t006-<pid>-<nanos>-<n>`), removed
in the test's teardown; no new crates.** Mirrors the acceptance-tests "Use an
isolated test workspace" setup line. Runner-up: `tempfile` — rejected (absent
from `Cargo.lock`, would need the registry index).
*source: agent-default · degradation row: dev-dep unavailable offline → std
substitute.*

❓ **Q8 — M0 prerequisite**: T-006 depends on M0 — verify before claiming done?

➡️ **Yes — run the M0 exit gates as part of Phase 6 verification, and treat a
red M0 gate as a blocker for T-006, not something to work around.**
IMPLEMENTATION_PLAN M0 exit: "schema, fixture, traceability, and
protected-evaluator access tests pass." Concretely: `cargo test --workspace`,
`make ci` (which runs fmt, clippy, rust+py tests, verify_package, traceability,
drift, seal).
*source: agent-default · plan dependency line.*

❓ **Q9 — Garbage collection semantics**: What may `gc` delete, and when?

➡️ **Staging area only, on an explicit cutoff age (default: everything never
committed); committed objects are never GC'd.** A staged file whose digest
never appears in the ledger *and* is older than the cutoff is removed; anything
under `objects/` is immutable and retained. This keeps GC itself from becoming
a data-loss risk — the failure mode of an over-eager GC in a content-addressed
store is unrecoverable, so the conservative side is chosen.
*source: agent-default · best-reversibility.*

❓ **Q10 — Documentation obligations**: Does T-006 need an ADR, GLOSSARY
edits, or traceability changes?

➡️ **No ADR — this executes IMPLEMENTATION_PLAN (not an architectural
departure; AGENTS.md requires decision records only for departures). No
`requirements.json`/`TRACEABILITY.md` edits — no obligation is added or
changed, and those documents are generated/manifest-gated. GLOSSARY.md gains
four terms** (event ledger, projection, staged artifact, content addressing):
it is 25 lines, is *not* in `MANIFEST.sha256`, not in the gate seal, not in the
generated-document drift set — so the edit is gate-safe (checked
`grep GLOSSARY tools/*.py` = no hits). Spec records the mapping R-013/R-014/
R-015/R-057 → tests.
*source: agent-default · gate inventory evidence.*

## Frontier status

Empty. Every branch settled from codebase evidence; nothing silently assumed;
no question deferred to a later round; no refusal-category item encountered
(no secrets, money, legal, auth, or external publication in this design).
