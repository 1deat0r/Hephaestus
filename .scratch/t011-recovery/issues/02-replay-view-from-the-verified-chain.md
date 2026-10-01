# 02: Replay view from the verified chain

**What to build:** A pure `replay(ledger) -> OperationView` fold: chain
verified by T-006 open, states derived only from events, ledger-order
determinism, impossible histories labelled `Corrupt` and never repaired —
so recovery has a trustworthy input.

**Blocked by:** 01 (durable operation recorder and effect receipts).

**Status:** ready-for-agent

- [x] States derived per grill Q4/Q5 (Planned/Succeeded/Failed/
      TimedOut/Cancelled/Ambiguous/Corrupt + cancel_requested flag)
- [x] Receipt-without-dispatch ⇒ that op `Corrupt` (fail-closed, reported);
      dispatch-without-receipt ⇒ `Ambiguous`; unknown event types ignored
      with a counted warning field, not a panic
- [x] Determinism: same ledger bytes ⇒ byte-identical serialized view;
      iteration order = ledger order (R-051 spirit cited)
- [x] Corrupt chain (tampered event) refuses replay entirely (T-006 open)
- [x] Unit tests for fold edges; integration tests cite R-057/AT-057

## Comments

2026-10-01T00:21:34Z — Done; verified by operations_deny (19 tests) + replay unit tests + green gates (279 workspace, doc-check 0, make ci 0, manifest 208/127).
