# Spec — M1 exit CLI: run & recover a deterministic fixture DAG

Status: ready-for-agent
Goal source: derived:roadmap (perpetual directive, cycle 9)
Grill record: `.scratch/m1-cli/grill.md` (10 questions, all self-answered)

## Problem Statement

M1's implementation tasks (T-006…T-011) are complete and green in tests,
but the plan's M1 exit line ends with a claim that is simply false today:
"A CLI can run and recover a deterministic fixture DAG." There is no
binary in the workspace, so none of the exit pillars — recorded replay,
denials that stay denied, bounded parallel budgets, non-duplicating
ambiguous effects — can be demonstrated by an operator with one command;
they exist only as test names.

## Solution

A `hephaestus` binary (inside the existing crate, modular monolith) with
two subcommands over a caller-chosen state directory:

1. **`fixture run --state-dir <DIR>`** — validates a fixed five-task fixture
   DAG and drives the *real* stack (TaskDag → Scheduler →
   RecordingExecutor → DeterministicExecutor → OperationRecorder →
   BudgetLedger): one task succeeds free, one succeeds priced, one fails
   terminally (non-retryable), one reports an ambiguous effect; the budget
   limit is deliberately below the priced total so a reservation is
   refused. Prints a stable JSON summary (task states, budget totals,
   refusal/ambiguity counts) — no timestamps — and exits 0.
2. **`fixture recover --state-dir <DIR>`** — reopens the hash-chained
   ledger, replays, and prints the same-shaped summary plus the
   `RecoveryPlan`: terminal stays terminal (denied stays denied), the
   ambiguous non-idempotent task lands in `unresolved` (never requeue),
   budget refusals accounted. Exit 0 on successful replay.

Error discipline: `--help` → exit 0 with usage; unknown/missing args →
stderr usage, exit 2; recover on a missing/foreign state dir → exit 1.
`run` refuses a non-empty state directory (no silent overwrite).

## User Stories

1. As an operator, I want one command to execute the M1 fixture through
   the real control-plane stack, so that the exit criterion is
   demonstrable, not asserted.
2. As an auditor, I want the summary as stable JSON, so that two runs of
   the same fixture are byte-comparable.
3. As an auditor, I want recover to show a terminal failure staying
   terminal, so that "denied operations stay denied" is visible.
4. As an auditor, I want the ambiguous non-idempotent task under
   `unresolved` in recover, so that "never blindly duplicated" is visible.
5. As an operator, I want the budget-limited refusal counted in the
   summary, so that "parallel budgets remain bounded" is visible.
6. As an operator, I want `recover` to read exactly what `run` wrote, so
   that recorded replay is the same ledger the stack produced.
7. As a user, I want usage errors on stderr with exit 2, so that scripts
   can tell bad invocations from operational failures (exit 1).
8. As a user, I want `--help` to document both subcommands, so that the
   CLI needs no README (which is manifest-frozen).
9. As a developer, I want zero new dependencies and no gate-file edits,
   so that every existing gate keeps passing unchanged.
10. As a reviewer, I want the fixture outcomes scripted as constants, so
    that the fixture is deterministic by construction (no I/O, no clock,
    no randomness in decisions).
11. As an operator, I want `run` to refuse a non-empty state dir, so
    that an accidental rerun cannot silently mix histories.
12. As a reviewer, I want tests spawning the real built binary, so that
    argv handling and exit codes are covered, not simulated.
13. As a future maintainer, I want the fixture definition in its own
    module, so that extending the exit demo does not bloat main.
14. As a reviewer, I want the four exit pillars asserted from CLI output,
    so that IMPLEMENTATION_PLAN:44 is checked by tests, not prose.
15. As a user of the offline build, I want the binary built by the same
    `cargo test`/clippy invocations gates already run, so that it cannot
    rot silently.

## Implementation Decisions

- **Binary:** `crates/hephaestus/src/main.rs` + `fixture.rs` module;
  binary name = package name `hephaestus` (grill Q1).
- **Subcommands/exit codes** per grill Q2; **JSON summaries** with fixed
  struct field order per Q6; **determinism** per Q3; **DeterministicExecutor**
  per Q4; **state dir** layout + empty-dir rule per Q7; **docs** per Q8
  (README untouched — manifest); **no glossary/ADR/Makefile changes**.
- **Fixture constants** live in `fixture.rs`: task table, scripted
  outcomes, budget limit — the single source for run *and* the tests'
  expectations.
- Exit pillars map to summary facts: `plan.terminal` contains (and
  `plan.requeue` excludes) the terminally-denied task; `plan.unresolved`
  contains the ambiguous non-idempotent task (absent from `requeue`);
  `budget.refused` counts bounded-budget refusals; `replayed_ops` is the
  history size; `ambiguous` is the run-time ambiguity count.

## Testing Decisions

- Seam: the built binary (`CARGO_BIN_EXE_hephaestus`) + its JSON output;
  no internal reach-ins (grill Q5's six cases, deny-first on error paths).
- Determinism proof: two fresh-state runs ⇒ byte-identical stdout.
- Pillar proof: recover JSON asserts the four exit-line claims.
- Prior art: `tests/cli` conventions absent (first binary — noted);
  deny-first style from `operations_deny.rs`.
- Suite: fmt, clippy `-D warnings`, `cargo test --workspace`,
  `make doc-check`, `make ci` — evidence to the LOG; new files staged +
  allowlisted at landing (standing).

## Out of Scope

- Mission intake/TUI/gateway (later milestones); sandbox execution inside
  the fixture (deterministic executor by design, grill Q4); budget
  persistence across restarts (recover's plan is printed, not auto-applied
  to a restored budget — budgets are in-process by T-008 design);
  `--force` overwrites; README documentation (manifest-frozen).

## Further Notes

- Limitation: "deterministic" covers the decision layer — ledger bytes
  contain wall-clock `created_at` values by design; the summary excludes
  them, which is what the byte-equality test pins.
- Limitation: recover prints the plan; applying budget effects needs a
  live budget and is a caller decision (documented on the subcommand).
