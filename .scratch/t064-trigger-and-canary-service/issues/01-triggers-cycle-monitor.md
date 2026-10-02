# 01: Recorded triggers, bounded cycles, and the monitor that stops rollouts

**What to build:** `TriggerKind`/`TriggerRecord` persisted on the
ledger, `run_improvement_cycle` (typed refusals + Proposed /
NoJustifiedChange over the single propose path), `monitor_deployment`
(bounded indicator loop → wired check → real rollback with
caller-proven incumbent digest), the champion_reuse monitor-driven
regression path, `tests/trigger_canary.rs`, and the honest 3.5c/M3
re-disposition — the last gap before an M3 completion claim.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] Triggers persist; empty subject refused (typed)
- [x] Cycle: 5 typed refusals + Proposed + NoJustifiedChange
- [x] Monitor: Completed (incl. over-long stream bound), Stopped with
      rollback receipt naming the indicator, UnverifiedIncumbent early
- [x] champion_reuse 3/3 monitor-driven; trigger_canary green; suites
      green
- [x] Gates green; allowlist +4 + reseal; assessment 3.5c + M3 table
      re-dispositioned from fresh receipts only

## Comments

Grill: `.scratch/t064-trigger-and-canary-service/grill.md`.
Spec: `.scratch/t064-trigger-and-canary-service/spec.md`.
