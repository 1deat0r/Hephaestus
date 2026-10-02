# 01: WAL deployment protocol + fail-closed recovery + wired canary check

**What to build:** write-ahead deployment (begin/commit with dedup),
atomic ledger persist, fail-closed `recover`/`from_json` (corrupt
ledger refuses instead of silently emptying), the guardrail-indicator
canary check, and the crash-reconciliation demo test covering both
crash windows plus the canary-driven rollback — M3 clause 3.5b's crash
half with receipts.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] Corrupt ledger → `Err(CorruptLedger)` (silent-empty hole fixed;
      all `from_json` callers on `Result`)
- [x] Atomic persist + stale-tmp reconciliation note
- [x] WAL windows: pending→replay exactly-once; append-without-cleanup
      → dedup (no duplicate entries)
- [x] Canary breach names the indicator; champion_reuse drives its
      regression through the check
- [x] New test green; existing suites green; allowlist +4 + reseal;
      gates green; assessment 3.5b updated honestly

## Comments

Grill: `.scratch/t063-crash-reconciliation-demo/grill.md`.
Spec: `.scratch/t063-crash-reconciliation-demo/spec.md`.
