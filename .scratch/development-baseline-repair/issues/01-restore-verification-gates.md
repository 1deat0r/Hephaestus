# BR-01: Restore verification gates

**Status:** ready-for-agent
**Verify:** make ci
**Covers:** 1, 2, 3, 4, 5, 6
**Requirements:** R-079, R-081, R-087
**Read first:** spec at `../spec.md`.

## Scope

This TASK restores the development baseline at HEAD 8b5278f. It touches
gate-adjacent files only: three rustfmt layout fixes, one stale citation
allowlist entry, allowlist lines for this feature's tracked files, and one
gate-seal regeneration (ADR-024, ADR-027). No runtime behavior changes.

One small task. Verification, receipts, the commit, and the remote CI
check are acceptance steps of that small task.

Requirement IDs served:
- R-079: stale allowlist entry removed after its real test citation was
  verified in `crates/hephaestus/tests/traceability.rs:141`.
- R-081: owner, milestone and acceptance coverage recorded in the spec and
  in this ticket's Covers list.
- R-087: gate change carries recorded decisions ADR-024 and ADR-027.

Original failure evidence: `../receipts/baseline-failures.md`.

**Small tasks:**

1. [ ] **S1** Restore the verification gates and land the repair
   **Status:** ready-for-agent
   **Verify:** make ci
   Fix fmt, the stale citation entry, and the seal; run full gates; save receipts; stage selected files; push; confirm remote CI.
   **Micro-tasks:**
   1. [ ] **M1** Apply rustfmt layout changes
      **Verify:** cargo fmt --all -- --check
      - [ ] Run `cargo fmt --all`; confirm only the three baseline Rust files change.
      - [ ] Confirm `cargo fmt --all -- --check` exits 0.
   2. [ ] **M2** Remove the stale citation entry
      **Verify:** make req-coverage
      - [ ] Confirm the citation at `crates/hephaestus/tests/traceability.rs:141` (commit 8e9ce81).
      - [ ] Delete only the R-079 line in `tools/requirement_citations.txt`; re-run `make req-coverage`.
   3. [ ] **M3** Reseal the runtime gates
      **Verify:** make gate-seal
      - [ ] Allowlist this feature's tracked files in `tools/runtime_allowlist.txt`.
      - [ ] Run `make seal` once; confirm `make gate-seal` exits 0.
   4. [ ] **M4** Run full gates and save receipts
      **Verify:** make ci
      - [ ] Run `make ci` and `make doc-check`; save results in `../receipts/checks-green.md`.
      - [ ] Confirm no pre-existing untracked file is staged.
   5. [ ] **M5** Commit, push, wait for remote CI
      **Verify:** make ci
      atomic

## Comments

Spec: `../spec.md`. Receipts: `../receipts/`.
External CI failure blocks the next task; repair forward, never rewrite.
