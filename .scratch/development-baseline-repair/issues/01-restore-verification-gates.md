# BR-01: Restore verification gates

**Status:** done
**Verify:** make ci
**Covers:** 1, 2, 3, 4, 5, 6, 7
**Requirements:** R-079, R-081, R-087
**Read first:** spec at `../spec.md`.

## Scope

This TASK restores the development baseline at HEAD 8b5278f. It touches
gate-adjacent files only: three rustfmt layout fixes, one stale citation
allowlist entry, allowlist lines for this feature's tracked files, and
gate-seal regenerations (ADR-024, ADR-027). After commit
`baseline-repair.1` (14b947f) failed remote `md-links` in the clean
checkout, the human authorized tracking seven existing
architecture-efficiency planning documents with content preserved (trailing blank EOF lines
   trimmed where the conflict-staged gate required) and registering
them in the runtime allowlist. No runtime behavior changes. No AE-01
work.

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

1. [x] **S1** Restore the verification gates and land the repair
   **Status:** done
   **Verify:** make ci
   Fix fmt, the stale citation entry, and the seal; run full gates; save receipts; stage selected files; push; confirm remote CI.
   **Micro-tasks:**
   1. [x] **M1** Apply rustfmt layout changes
      **Verify:** cargo fmt --all -- --check
      - [x] Run `cargo fmt --all`; confirm only the three baseline Rust files change.
      - [x] Confirm `cargo fmt --all -- --check` exits 0.
   2. [x] **M2** Remove the stale citation entry
      **Verify:** make req-coverage
      - [x] Confirm the citation at `crates/hephaestus/tests/traceability.rs:141` (commit 8e9ce81).
      - [x] Delete only the R-079 line in `tools/requirement_citations.txt`; re-run `make req-coverage`.
   3. [x] **M3** Reseal the runtime gates
      **Verify:** make gate-seal
      - [x] Allowlist this feature's tracked files in `tools/runtime_allowlist.txt`.
      - [x] Run `make seal` once; confirm `make gate-seal` exits 0.
   4. [x] **M4** Run full gates and save receipts
      **Verify:** make ci
      - [x] Run `make ci` and `make doc-check`; save results in `../receipts/checks-green.md`.
      - [x] Confirm no untracked file outside M6's authorized seven is staged.
   5. [x] **M5** Commit, push, wait for remote CI
      **Verify:** make ci
      atomic
   6. [x] **M6** Integrate planning documents and land the correction
      **Verify:** make ci
      - [x] Track the seven approved architecture-efficiency planning documents; trim trailing blank EOF lines only.
      - [x] Register the seven paths in `tools/runtime_allowlist.txt` and run `make seal`.
      - [x] Verify a tracked-only export with `make ci` and `make doc-check`.
      - [x] Stage the completed ticket, document the omission in the receipt, commit `baseline-repair.2`, push, wait for green CI.

## Comments

Spec: `../spec.md`. Receipts: `../receipts/`.
External CI failure blocks the next task; repair forward, never rewrite.
