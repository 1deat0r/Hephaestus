# T-063 grill - crash reconciliation + wired canary (M3 clause 3.5b)

Goal: the last assessment NOT-RUN = "Crash-mid-deployment reconciliation
(interrupted deployments) + live canary/drift services" (R-118: restart
and crash recovery MUST reconstruct the champion and reconcile
interrupted deployments; violating guardrails MUST stop the rollout).

## Facts (probes)

- Zero crash/reconcile coverage in selfimprove (only the limitations
  comment); canary exists only as Deployment.observed_scope string.
- `ImprovementLedger::from_json` = `serde unwrap_or_default` — a
  CORRUPT/TRUNCATED ledger silently becomes an EMPTY one: a genuine
  R-116/R-118 violation (silent loss of learned decisions) hiding in
  the restart path. 3 call sites (self_improvement:214,
  champion_reuse:194/236).
- promote() is pure; append-at-commit-point = the only persistence —
  crash between decision and append currently loses the DECISION with
  no record to reconcile (nothing durable to detect).
- champion_reuse already demos restart-from-stable + direct rollback;
  the canary CHECK (indicator → violation → rollback) is not a fn.

## Q1 - Crash model + reconciliation contract?
**A:** Write-ahead deployment protocol, all typed:
- `begin_deployment(dir, &LedgerEntry)` → writes `pending.json`
  (the exact entry to append) — the decision becomes durable FIRST;
- `commit_deployment(dir, &mut ledger, entry)` → appends (dedup:
  identical challenger+target+outcome already present → no second
  append), persists atomically (`persist` = temp+rename), removes
  pending;
- `recover(dir) -> Result<(ImprovementLedger, Vec<RecoveryAction>),
  RecoverError>`: corrupt main → `Err(CorruptLedger)` (FAIL CLOSED —
  fixes the silent-empty hole; `from_json` likewise becomes
  `Result<Self, RecoverError>`, 3 callers updated); stale
  `pending.json` → replayable action (well-formed) or loud discard
  (malformed); stale temp file → ignored with action note; main
  missing → empty ledger + note.
Reconcile = recovery REPORTS the interrupted deployment as a
replayable action; the caller (demo) completes it exactly once.
(agent-default)

## Q2 - Wired canary?
**A:** `check_deployment_guardrails(&Deployment, &GuardrailIndicators)
-> Result<(), CanaryViolation>`: predeclared indicators (name, observed,
limit) — breach names the indicator. The demo's regression path goes
THROUGH this fn (observation → violation → rollback) instead of a bare
rollback call; `champion_reuse`'s rollback step is refactored to use it
(one-line change + comment) so the monitor step is genuinely wired.
A continuous polling daemon stays out (typed-records limitation keeps
its honest line in the exit doc — "wired per-mission check", not a
live service). (agent-default)

## Q3 - Where do the demo tests live?
**A:** New `tests/crash_reconciliation.rs`: (i) corrupt-ledger recovery
refuses (no silent empty) + from_json Result callers green;
(ii) atomic persist: stale tmp ignored, main intact; (iii) WAL crash
windows: pending-without-append → recover reports replayable → commit
exactly-once (no dup), append-without-pending-cleanup → dedup;
(iv) canary: guardrail breach names the indicator and drives rollback.
champion_reuse gets the canary-wiring tweak (Q2). (agent-default)

## Q4 - Scope?
**A:** Triggers recording (R-115 "triggers are recorded") NOT in this
ticket — noted for the next assessment pass; no live polling daemon;
no commit (rule 5). Exit assessment 3.5b updates to PASS only for the
crash half after receipts exist — live-daemon wording stays honest.
(agent-default)
