# 02: `fixture recover` and exit-pillar proof

**What to build:** The second half of IMPLEMENTATION_PLAN:44 — reopen the
state dir, replay the hash-chained ledger, print the summary plus the
RecoveryPlan, and assert all four M1 exit pillars from real CLI output.

**Blocked by:** 01 (`hephaestus` binary with `fixture run`).

**Status:** done

- [x] `fixture recover --state-dir <DIR>`: replay + plan JSON (requeue/
      unresolved/terminal/cancelled/corrupt in ledger order), exit 0;
      missing dir → exit 1
- [x] Pillar assertions from recover output: terminal task stays terminal
      (denied stays denied); ambiguous non-idempotent task under
      `unresolved`, absent from `requeue`; budget refusal count matches
      run; `replayed_ops` equals fixture size (recorded replay)
- [x] `run` → `recover` on the same dir round-trips (states survive)
- [x] Full gates: fmt, clippy `-D warnings`, `cargo test --workspace`,
      `make doc-check`, `make ci` — evidence lines in the state LOG; new
      files staged + allowlisted (standing procedure)

## Comments

2026-10-01T00:56:21Z — Done; 7/7 CLI tests green against the real binary (run, twin determinism, recover pillar proof, usage/exit codes, empty-dir refusal, missing-state exit 1).

Closed 2026-10-02 — work landed earlier; verified green: cli tests 9/9 green; fixture run and fixture recover verified in CI.
