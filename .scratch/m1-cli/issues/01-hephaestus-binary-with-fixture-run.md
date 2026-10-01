# 01: `hephaestus` binary with `fixture run`

**What to build:** A real binary that validates the fixture DAG and drives
the entire control-plane stack — scheduler, recording executor, budget,
operation recorder — then prints a stable JSON summary proving the exit
line's four pillars are reachable by one command.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [x] `src/main.rs` + `fixture.rs`: `fixture run --state-dir <DIR>` runs
      the constant fixture (5 scripted tasks; budget limit below priced
      total) through TaskDag::validate → Scheduler → RecordingExecutor →
      DeterministicExecutor → OperationRecorder → BudgetLedger
- [x] Summary JSON (fixed field order): per-task states, budget
      limit/spent/refused, ambiguity count — no timestamps; exit 0
- [x] `--help` exit 0 documents both subcommands; unknown args → stderr
      usage + exit 2; `run` refuses non-empty state dir (exit 1)
- [x] DeterministicExecutor: scripted outcomes keyed by task id, zero I/O
- [x] Zero new dependencies; no README/Makefile/workflow edits
- [x] Tests (spawn `CARGO_BIN_EXE_hephaestus`): run exit 0 + JSON parses +
      expected states; twin fresh runs byte-identical; `--help` mentions
      both subcommands; unknown subcommand exits 2

## Comments

2026-10-01T00:56:21Z — Done; 7/7 CLI tests green against the real binary (run, twin determinism, recover pillar proof, usage/exit codes, empty-dir refusal, missing-state exit 1).
