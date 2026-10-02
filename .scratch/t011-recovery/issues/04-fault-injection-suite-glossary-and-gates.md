# 04: Fault-injection suite, glossary, and gate evidence

**What to build:** The plan's four crash boundaries proven with the
T-006 reopen-from-disk pattern, plus the cancellation boundary, the domain
vocabulary, and green gates — closing the run and the M1 recovery story.

**Blocked by:** 03 (recovery plan with budget reconciliation).

**Status:** done

- [x] Boundary tests: (1) planned-only crash ⇒ requeue, budget untouched;
      (2) dispatched-no-receipt ⇒ Ambiguous ⇒ release+unresolved (or
      permitted requeue); (3) artifact-committed-no-receipt ⇒ still
      Ambiguous, orphan retained; (4) AmbiguousEffect crash before
      `reconciled` ⇒ `mark_unresolved` exactly once on apply (double-apply
      idempotent); (5) cancel_requested crash ⇒ plan cancels, never
      requeues — each with reopen-from-disk
- [x] End-to-end: RecordingExecutor over a mock (or SandboxExecutor)
      drives a scheduler DAG while recorder+ledger+budget stay consistent
      after simulated death (AT-057 audit history preserved)
- [x] GLOSSARY gains: operation, effect receipt, recovery plan (decision
      row written **first**)
- [x] fmt / clippy `-D warnings` / `cargo test --workspace` /
      `make doc-check` / `make ci` green with evidence lines; new files
      staged + allowlisted; limitations (single writer, sync_data scope,
      no real-external reconciliation) recorded in spec/report

## Comments

2026-10-01T00:21:34Z — Done; verified by operations_deny (19 tests) + replay unit tests + green gates (279 workspace, doc-check 0, make ci 0, manifest 208/127).

Closed 2026-10-02 — work landed earlier; verified green: operations_deny replay, recovery, and fault tests green (M0_M1 receipts).
