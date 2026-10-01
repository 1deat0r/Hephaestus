# Spec — T-011: durable operations, receipts, replay, fault injection

Status: ready-for-agent
Goal source: derived:roadmap (perpetual directive, cycle 8)
Grill record: `.scratch/t011-recovery/grill.md` (12 questions, all self-answered)

## Problem Statement

Everything that *executes* is now durable-adjacent (scheduler dispatches,
sandbox runs, budget reserves) but nothing is durable *about execution*:
an operation that crashes between dispatch and completion leaves only
inference — no record that it started, no receipt that it finished, no
policy for what may be retried. R-057/AT-057 ("recovery preserves audit
history and retries only permitted operations") has no runtime home,
MASTER_SPEC:369's "recorded before the effect is dispatched" is unenforced,
and the T-006/T-007/T-008 integrations were explicitly deferred to this
task.

## Solution

An `operations` module built on the existing substrates:

1. **`OperationRecorder`** — durable operation IDs and lifecycle events
   (`planned`, `dispatched`, `receipt`, `cancel_requested`,
   `output_committed`, `reconciled`) appended to the T-006 `EventLedger`
   with `sync_data` **before** the corresponding effect; effect receipts
   serialize to JSON, commit through the T-006 `ArtifactStore`, and land as
   events referencing `payload_sha256`.
2. **`RecordingExecutor`** — a `TaskExecutor` decorator (zero scheduler
   changes) that appends `dispatched` before delegating and commits+records
   the receipt after; `recorder.cancel` records before flipping the
   scheduler handle.
3. **Replay** — a pure fold over the hash-chained ledger producing per-op
   states (`Planned/Succeeded/Failed/TimedOut/Cancelled/Ambiguous/
   Corrupt` — `Dispatched` is transient and rests as `Ambiguous`) in
   ledger order; corrupt orders fail closed. Receipt-event presence drives
   the Corrupt rule even when a payload is unreadable (pass-1 fix).
4. **`recover()` → `RecoveryPlan`** — AT-057's rule table: Planned ⇒
   requeue; Ambiguous ⇒ release any held reservation and record
   `unresolved` exactly once, **unless** `retryable && attempts <
   max_attempts` ⇒ requeue (never a blind retry, MASTER_SPEC:371);
   exhausted ambiguity ⇒ `unresolved` too — a possibly-completed effect
   must be reconciled, never relabelled a settled failure (:371; the
   earlier "terminal Failed" wording here was wrong and is corrected);
   `cancel_requested` ⇒ cancel, not requeue;
   Corrupt ⇒ surfaced, never auto-requeued.

## User Stories

1. As a scheduler caller, I want an operation recorded durably *before* it
   dispatches, so that a crash mid-effect still proves the effect was
   authorized to start.
2. As an auditor, I want every receipt (outcome, cost, wall-time, reason,
   attempt, artifact hashes) content-addressed and hash-referenced by its
   event, so that receipts are tamper-evident and crash-safe as a pair.
3. As a recovery routine, I want replay to be a pure function of the
   verified chain, so that the same ledger always yields the same states
   (R-051's determinism spirit).
4. As a recovery routine, I want a receipt without a dispatch labelled
   `Corrupt` and excluded from requeue, so that impossible histories never
   silently repair themselves.
5. As recovery, I want dispatch-without-receipt labelled `Ambiguous`, so
   that a possible external effect is reconciled, never re-run blindly.
6. As an auditor, I want ambiguous non-idempotent work to end as a budget
   `unresolved` entry exactly once (idempotent guard), so that AT-056's
   never-duplicated rule survives recovery itself.
7. As an operator, I want retryable-with-budget work requeued only while
   `attempts < max_attempts`, so that AT-057's "retries only permitted"
   is data, not folklore.
8. As an operator, I want a crash after `cancel_requested` to recover as
   cancelled — not requeued — so that cancellation intent is durable.
9. As a developer, I want a decorator instead of scheduler edits, so that
   capability contracts stay untouched.
10. As a test, I want crashes simulated by reopening the ledger from disk
    (the T-006 pattern), so that fault injection needs no instrumentation.
11. As a reviewer, I want R-057/AT-057 (and the four plan boundaries)
    named in test headers, so traceability is checkable.
12. As a user of the offline build, I want zero new dependencies, so every
    gate keeps working.
13. As a maintainer, I want one writer and derived-only state, so there is
    exactly one source of truth (the ledger).
14. As an auditor, I want an artifact committed without its receipt to
    remain an orphan (retained) while the operation stays `Ambiguous`, so
    the T-006 orphan rule and T-011 recovery agree.
15. As a future gateway, I want operation IDs generated monotonically and
    queryable from the replay view, so external clients can poll status.

## Implementation Decisions

- **Module:** `operations/` beside the other core modules (grill Q2).
- **Recorder** over `EventLedger` path: events use the `event` contract
  with mission id `OPS`, `event_type` lifecycle strings, `operation_id`
  field; `sync_data` before return (grill Q3); receipt payload via
  `ArtifactStore` + `payload_sha256`.
- **IDs:** recorder-generated monotonic `OP-<n>` + caller tag (grill Q4).
- **States derived by replay only** (grill Q4/Q5); `RecordingExecutor`
  decorator, no scheduler/sandbox edits (grill Q10).
- **RecoveryPlan** rule table exactly per grill Q6; applying a plan is the
  caller's loop (module provides the plan).
- **Single writer** `&mut` (grill Q9); **fault injection** = reopen-from-
  disk at the four plan boundaries + cancellation (grill Q8).
- **Glossary:** operation, effect receipt, recovery plan — decision row
  **before** the edit. No ADR; no schema/requirements changes; zero deps.

## Testing Decisions

- Seam-level: recorder APIs, `RecordingExecutor::run`, `replay()`,
  `recover()` — public behavior only; in-module unit tests for fold rules.
- Prior art: `event_ledger.rs` (reopen/torn-tail pattern), `budget_ledger.rs`
  (deny matrices), `scheduler_run.rs` (outcomes).
- **Bound tests cite R-057/AT-057:** append-before-effect ordering (effect
  never observed without its event — decorator asserts ledger tip first);
  receipt store+event pairing; replay determinism (same ledger ⇒ same view,
  byte-identical JSON); Corrupt on receipt-without-dispatch; the four
  fault boundaries; cancellation durability; recovery-plan table (each
  row); budget release + exactly-once `unresolved`; zero-dependency claim.
- Suite: fmt, clippy `-D warnings`, `cargo test --workspace`,
  `make doc-check`, `make ci` — evidence to the LOG; new files staged +
  allowlisted.

## Out of Scope

- Real external-system reconciliation calls (hooks only — no externals
  exist yet); idempotency-key negotiation; distributed/multi-writer
  journals; checkpoint files beyond the ledger; automatic rescheduling
  loops (the Scheduler owns dispatch; recovery produces a plan);
  gateway-facing status APIs (view is queryable, no service).

## Further Notes

- Limitation: durability is in-process `sync_data` (same class as T-006),
  single authoritative writer (MASTER_SPEC:379); no power-failure
  simulation.
- Limitation: `Ambiguous` resolution against real externals needs the
  provider/reconciliation layer — until then unresolved entries are the
  honest resting state.
- The ledger-integration deferrals from T-007/T-008 close here for the
  operation path; those modules' own optional event feeds remain future
  work unless a ticket asks.
