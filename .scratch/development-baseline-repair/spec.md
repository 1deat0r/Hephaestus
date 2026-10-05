# Spec — Development baseline repair: restore verification gates (ADR-024)

Status: ready-for-agent
Goal source: human authorization 2026-10-05 (dependency-ordered backlog, first small task)
Baseline HEAD: 8b5278ffa18f083740394dca237e033814559ba4 (main, already pushed)

## Problem Statement

Three local gates fail at the baseline HEAD, so `make ci` stops at its
first target and the remote `ci` workflow has failed on the last five
pushes.

1. `cargo fmt --all -- --check` reports rustfmt layout differences in
   `crates/hephaestus/src/main.rs`,
   `crates/hephaestus/tests/architecture_resource_admission.rs`, and
   `crates/hephaestus/tests/cli.rs` (8 diff hunks, exit 1).
2. `make req-coverage` fails with
   `requirement-citations: stale allowlist entry R-079 — now cited`.
   The real citation sits in
   `crates/hephaestus/tests/traceability.rs:141` (test
   `m5_milestone_not_required_for_m0_through_m4_completion`), added by
   commit 8e9ce81.
3. `make gate-seal` fails with
   `seal mismatch: tools/runtime_allowlist.txt`. Commit 8b5278f added
   two intentional allowlist lines and did not regenerate the seal.

Formatting must change no behavior. The stale entry must be checked
against its real citation before removal. The seal must be regenerated
deliberately after the intentional allowlist change. No gate may weaken.

## Solution

1. Apply `cargo fmt --all` (layout only) to the three named files.
2. Delete only the `R-079` line in `tools/requirement_citations.txt`
   after confirming the test citation.
3. Allowlist this feature's own tracked files, then run `make seal`
   exactly once (ADR-024, ADR-027).
4. Keep compact receipts in this feature directory, run `make ci` and
   `make doc-check`, and land the repair in one commit.

## User Stories

1. As a developer, I want `make ci` green at the baseline, so that the
   next task starts from a trusted state.
2. As a reviewer, I want the stale citation entry removed only after its
   real citation is confirmed, so that coverage stays honest.
3. As a maintainer, I want one deliberate seal regeneration with an ADR
   citation, so that gate changes stay traceable.

## Acceptance Criteria

1. `cargo fmt --all -- --check` exits 0, and the staged diff limited to
   `crates/**` contains only the three named Rust files, each with
   rustfmt layout changes only (no behavior change). The full commit diff
   also touches gate and ticket files, which AC 2, AC 3 and AC 5 cover.
2. `make req-coverage` exits 0 with the stale `R-079` line removed and
   the real citation at `crates/hephaestus/tests/traceability.rs:141`
   confirmed by inspection of commit 8e9ce81.
3. `make gate-seal` exits 0 after one deliberate `make seal`
   regeneration, with the intentional allowlist change verified from git
   history first.
4. `make ci` and `make doc-check` both exit 0 locally after the repair.
5. Every new tracked file of this feature is allowlisted in
   `tools/runtime_allowlist.txt`, and no pre-existing untracked file is
   staged.
6. Commit `baseline-repair.1: restore verification gates (ADR-024)` is
   pushed to origin/main and its `ci` workflow run finishes with all jobs
   green.

## Out of Scope

- Architecture runtime work (arch-eff tickets stay untouched).
- Any change to protected evaluators, gate definitions, or test semantics.
- Staging `.pi/`, `.scratch/architecture-efficiency/` planning files, or
  `docs/PI_AGENTS_SETUP_RESEARCH_2026-10.md`.
- Installing or downgrading Pi (observed installed version: 1.0.2, not the
  preparation protocol's 1.0.0; recorded here as an observed difference).
