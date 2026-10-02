# 01: Durable operation recorder and effect receipts

**What to build:** `OperationRecorder` + `RecordingExecutor` decorator:
every lifecycle transition (planned/dispatched/receipt) hits the T-006
EventLedger with `sync_data` **before** the effect it announces, and every
receipt is content-addressed through the ArtifactStore and referenced by
`payload_sha256` — MASTER_SPEC:369 enforced, not documented.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] Recorder generates monotonic durable operation IDs; `plan()` appends
      `operation.planned` durably before anything may dispatch
- [x] `RecordingExecutor::run` appends `operation.dispatched` **before**
      delegating (assert: a failing inner executor still leaves both events
      in order) and commits the receipt JSON to the ArtifactStore then
      appends `operation.receipt` with matching `payload_sha256`
- [x] `recorder.cancel(id)` appends `cancel_requested` before the caller
      flips any scheduler handle
- [x] Receipt contents: outcome variant, cost, wall_ms, reason, attempt,
      executor tag, artifact hashes (grill Q7)
- [x] Crash between store-commit and event leaves an orphan artifact
      (retained — T-006 rule) and NO receipt event
- [x] Deny-first tests in `tests/operations_deny.rs` citing R-057/AT-057;
      zero new dependencies; wrong-sequence/append-refusal paths leave the
      ledger unchanged

## Comments

2026-10-01T00:21:34Z — Done; verified by operations_deny (19 tests) + replay unit tests + green gates (279 workspace, doc-check 0, make ci 0, manifest 208/127).

Closed 2026-10-02 — work landed earlier; verified green: operations_deny replay, recovery, and fault tests green (M0_M1 receipts).
