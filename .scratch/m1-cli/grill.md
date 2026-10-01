# Grill — M1 exit CLI: run & recover a deterministic fixture DAG

Self-interview (Phase 2, perpetual-directive cycle 9). Frontier emptied in
one round.

Goal: *a `hephaestus` CLI binary that runs a deterministic fixture DAG
through the recorder/scheduler/budget stack and recovers it from its
ledger, making IMPLEMENTATION_PLAN's M1 exit line (line 44) true end-to-end.*

## Evidence

- IMPLEMENTATION_PLAN:44 (M1 Exit): four evidence pillars + "A CLI can run
  and recover a deterministic fixture DAG" — pillars exist in test suites;
  **no binary exists** (`src/main.rs` absent, no `[[bin]]`).
- MASTER_SPEC:42 promises "a local CLI"; :431 modular monolith (binary
  belongs in the existing `hephaestus` crate, not a new crate).
- README is in `MANIFEST.sha256` (81-file frozen envelope) → editing it
  needs a versioned reseal; `check_readme_fences` guards README shell
  fences. CLI docs therefore live in `--help` + code docs (decision row).
- Stack to exercise: `Scheduler` + `RecordingExecutor` + `OperationRecorder`
  + `BudgetLedger` + `TaskDag::validate` — all built in T-006..T-011.
- House rules: zero new dependencies (std arg parsing), `cargo test` builds
  bins so a `[[bin]]` is covered by existing gates; new source files are
  staged + allowlisted at landing (standing procedure).

## Round 1

❓ **Q1 — Binary placement**: `crates/hephaestus/src/main.rs` (+ a
`fixture` module) → binary named `hephaestus` (package name). No new crate
(modular monolith, MASTER_SPEC:431).
*source: plan + Cargo layout.*

❓ **Q2 — Subcommands**: `hephaestus fixture run --state-dir <DIR>` builds
the fixture DAG (constants: 4 tasks — succeeds/free, succeeds/priced,
fails (retryable:false), ambiguous-effect (retryable:false) — plus a budget
limit deliberately smaller than the priced total so one reserve is
refused), runs it through the **real** stack (TaskDag::validate →
Scheduler → RecordingExecutor → DeterministicExecutor → OperationRecorder),
prints a **stable JSON summary** (states, budget totals, refusal counts —
no timestamps), exits 0. `hephaestus fixture recover --state-dir <DIR>`
reopens the ledger, replays, prints the same-shaped summary plus the
`RecoveryPlan` (requeue/unresolved/terminal/cancelled/corrupt), exits 0.
`--help`/bad args → usage on stderr, exit 2. No state dir → creates it;
recover on missing dir → error exit 1.
*source: exit criterion wording + determinism discipline.*

❓ **Q3 — Determinism definition**: same fixture into two fresh state dirs
⇒ **byte-identical summaries** (decision layer only; ledger `created_at`
timestamps are excluded from the summary by construction). In-run
determinism: no clock/randomness in decisions (fixture outcomes are
constants); the DeterministicExecutor returns scripted outcomes per task
id. "Denied stays denied": the failing task's terminal `Failed` appears in
recover with **no** requeue entry. Budget pillar: refused reservation shows
as `BUDGET_UNAVAILABLE`-style failure count in the summary; ambiguous
non-idempotent pillar: recover lists it under `unresolved`, never requeue.
*source: plan exit line + R-051 spirit.*

❓ **Q4 — DeterministicExecutor**: in-binary `TaskExecutor` impl keyed by
task id → constant outcomes (scripted Succeeded{cost}/Failed/Ambiguous).
No I/O → the *fixture* is deterministic while still exercising the real
scheduler/recorder/budget path. Sandbox stays out (T-010 owns real
isolation; a fixture must not depend on bwrap presence).
*source: "deterministic" in the exit line + portability.*

❓ **Q5 — Tests**: `tests/cli.rs` integration tests spawn the built binary
via `env!("CARGO_BIN_EXE_hephaestus")`: (1) `fixture run` exit 0 + summary
parses + expected states; (2) two fresh runs → identical summary bytes;
(3) `fixture recover` on that state → exit 0 + plan shows terminal-failed
not requeued + ambiguous in unresolved + priced refusal accounted;
(4) `--help` exit 0 mentions both subcommands; (5) unknown subcommand →
exit 2; (6) recover on nonexistent dir → exit 1. Deny-first for error paths.
*source: repo test conventions.*

❓ **Q6 — Output shape**: stdout = one JSON object
`{fixture, tasks:[{id,state}], budget:{limit,spent,refused}, plan?}`;
stderr = human errors. serde_json already a crate dep (workspace
serialization), zero new dependencies. Field order fixed by struct
declaration (golden-byte comparisons in tests rely on it).
*source: determinism + house style.*

❓ **Q7 — State dir layout**: `<state-dir>/ops-ledger.jsonl` +
`<state-dir>/store/` (OperationRecorder::open's two paths). `run` refuses
to overwrite a non-empty state dir? Choose: `run` **requires empty/absent**
dir (fail-closed — reruns must be explicit by picking a new dir or
`--force`); `--force` allowed? Keep: no force flag (YAGNI) — error exit 1
with a clear message. Simplest honest.
*source: best reversibility.*

❓ **Q8 — Docs**: `--help` strings + module docs carry usage; README NOT
touched (manifest); GLOSSARY: no new domain terms this cycle (fixture =
plan vocabulary already) → no glossary edit, decision row notes it.
*source: manifest evidence + grill Q10 of prior runs.*

❓ **Q9 — Gates**: no Makefile/workflow changes (no seal churn); binary is
covered by `cargo test`/clippy automatically; new files staged+allowlisted
at landing (standing). Evidence: CLI tests green + `make ci` + doc-check.
*source: house gate mechanics.*

❓ **Q10 — Ticket shape**: **2 tickets** — 01: binary + `fixture run`
(fixture module, DeterministicExecutor, JSON summary, arg handling) +
tests 1/2/4/5; 02: `fixture recover` (replay+plan printing, empty-dir
rules, tests 3/6) + full gates/evidence. Edge 1→2.
*source: vertical slice sizing; run is verifiable without recover.*

## Frontier status

Empty. No refusal-category item (local file state only, no credentials,
no external publication).
