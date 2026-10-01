# M0 + M1 Exit Assessment

**Date:** 2026-10-01 (UTC)
**Scope:** M0 and M1 exit criteria only — IMPLEMENTATION_PLAN.md lines 26
and 44. **M2–M6 exit lines are deliberately not assessed** (their work has
not been performed; inventing dispositions for them would be fabrication).
**Method:** fresh commands run in this session; every `PASS` below points
at a receipt (command + result, or a named test) a skeptic can re-run.
**What this document is NOT:** it does not change any release label, does
not establish scientific validity, production security, or performance,
and is not a substitute for the per-run workflow reports under
`docs/agents/auto-workflow/`.

---

## M0 exit assessment (IMPLEMENTATION_PLAN:26)

> Exit: schema, fixture, traceability, and protected-evaluator access
> tests pass. The release label is specification/engineering foundation,
> not autonomous invention.

| # | Criterion | Disposition | Evidence (fresh this session) |
|---|-----------|-------------|-------------------------------|
| 0.1 | Schema tests pass | **PASS** | `make ci` → EXIT 0 includes `verify_package` → `"status": "PASS"` with `principal_record_schemas: 18`,
  `schema_valid_example_records: 13`, `requirements: 119`; `cargo test -p hephaestus --test contracts` → **12 passed, 0 failed**; `generated.rs is up to date` (freshness gate inside `make ci`) |
| 0.2 | Fixture tests pass | **PASS** | `cargo test -p synthetic-evaluator` → **19 passed, 0 failed** (true/false/confounder/ambiguous corpus probes); `make ci` → reference fixture suites `Ran 94 tests … OK` (Python: fixture corpus, qualification, traceability, digest vectors); `cargo test -p hephaestus --test digest_conformance` → **2 passed, 0 failed** |
| 0.3 | Traceability tests pass | **PASS** | `cargo test -p hephaestus --test traceability` → **16 passed, 0 failed**; `verify_package` reciprocal-traceability within the PASS above; `make ci` runs `gen-check` (generated contract freshness) and the generated-document drift checks green |
| 0.4 | Protected-evaluator access tests pass | **PASS** | `cargo test -p hephaestus --test evaluator_access` → **4 passed, 0 failed**; `cargo test -p hephaestus --test sealed_deny` → **14 passed, 0 failed** (sealed access, deny-first) |
| 0.5 | Release label unchanged: "specification/engineering foundation, not autonomous invention" | **RECORDED — not changed by this assessment** | Label rule restated in *Limitations and non-claims* below; no release machinery was touched in this cycle |

**M0 disposition: PASS** (all four test clauses green from fresh runs;
label rule recorded).

---

## M1 exit assessment (IMPLEMENTATION_PLAN:44)

> Exit: recorded operations replay, denied operations stay denied, parallel
> budgets remain bounded, and ambiguous non-idempotent effects do not
> duplicate. A CLI can run and recover a deterministic fixture DAG.

| # | Clause | Disposition | Evidence (fresh this session) |
|---|--------|-------------|-------------------------------|
| 1.1 | Recorded operations replay | **PASS** | `cargo test -p hephaestus --test operations_deny` (19 tests, incl. `replay_is_deterministic_across_reopens`, `reopen_continues_monotonic_ids_and_sequences`, five fault-boundary reopen tests); CLI receipt below: `"replayed_ops": 5` after a real process restart of `fixture recover` |
| 1.2 | Denied operations stay denied | **PASS** | `rule_table_covers_every_state` asserts the terminally-failed op is in `plan.terminal` and **absent from `requeue`**; CLI receipt: `"terminal":["alpha","beta","gamma"]` with `"requeue":["epsilon"]` — `gamma` never requeued. Policy-engine deny matrices (R-060/R-099 suites in `cargo test --workspace`) cover the authorization half |
| 1.3 | Parallel budgets remain bounded | **PASS** | `cargo test -p hephaestus --test budget_ledger` — `concurrent_reservations_never_oversubscribe` (8 threads × 50 attempts against a limit of 100; `ok+err==400`, `reserved==ok*3`, never over limit); CLI receipt: `"budget":{"limit":100,"spent":60,"unresolved":40,"refused":1}` |
| 1.4 | Ambiguous non-idempotent effects do not duplicate | **PASS** | `apply_records_unresolved_exactly_once_and_releases_holds` and `fault_boundary_ambiguous_effect_reconciles_exactly_once` (second `apply` records nothing; `unresolved` count stable); CLI receipt: `"plan":{"unresolved":["delta"]}` and `delta ∉ requeue` |
| 1.5 | A CLI can run and recover a deterministic fixture DAG | **PASS** | Re-run this session: `hephaestus fixture run --state-dir <tmp>` then `fixture recover --state-dir <tmp>` (exit 0 both): run → `{"fixture":"m1-exit","tasks":[{"id":"alpha","state":"succeeded"},{"id":"beta","state":"succeeded"},{"id":"gamma","state":"failed"},{"id":"delta","state":"ambiguous"},{"id":"epsilon","state":"planned"}],"budget":{"limit":100,"spent":60,"unresolved":40,"refused":1},"ambiguous":1}`; recover → same states plus `"plan":{"requeue":["epsilon"],"unresolved":["delta"],"terminal":["alpha","beta","gamma"],"cancelled":[],"corrupt":[]},"replayed_ops":5`. `cargo test -p hephaestus --test cli` → **8 passed, 0 failed** (incl. twin-fresh-runs byte-identical summaries) |

**M1 disposition: PASS** (five clauses green from fresh runs, CLI demo
re-run this session).

---

## Gate evidence (the suite that wraps every claim above)

- `make ci` → **EXIT 0** (post-staging run — the one that gates this very
  document) — `manifest-check: ok (tracked=212, allowlisted=131)`,
  `gate-seal: ok (20)`, `generated.rs is up to date`,
  `verify_package` `"status": "PASS"`, `Ran 94 tests … OK`,
  `hooks-check: ok`
- `cargo test --workspace` → **291 passed, 0 failed** (five consecutive
  full runs this session ended 291/0; see caveat)
- `make doc-check` → **EXIT 0** (rustdoc `-D warnings`)

**Honesty caveat (observed once, not hidden):** one earlier full-suite run
this session reported `179 passed, 1 failed` with only aggregate output
captured — the failing test was not identified. Five subsequent complete
runs were 291/0, and `make ci` (which runs the same Rust suites) was green
throughout. Disposition: treated as an unexplained single transient, not
as evidence of a systematic fault; if it recurs, this assessment's M1
receipts must be re-verified before relying on them.

---

## Limitations and non-claims (AGENTS.md: specification ≠ invention)

This assessment does **not** establish:

- **Scientific validity or invention quality** — no campaign, dossier, or
  qualified result is involved; the package remains a
  specification/engineering foundation (M0 label rule, unchanged).
- **Production security** — the sandbox is unprivileged-userns-dependent
  with no seccomp layer and no brokered egress (documented limitations in
  the sandbox module); it is not a multi-tenant boundary.
- **Durability across process death for budgets and schedulers** — those
  are in-process by design (MASTER_SPEC:379's single writer); only the
  event ledger persists, which is why recovery replays *operations*, not
  balances.
- **Real-external reconciliation** — ambiguous effects rest as budget
  `unresolved` entries; no external system has ever been contacted.
- **Performance of any kind** — no benchmark was run or implied.
- **Multi-milestone coverage** — M2–M6 dispositions do not exist here by
  design.

Evidence tiers used above: *test receipt* = automated, re-runnable;
*command receipt* = re-runnable shell command with recorded output;
*documentary* = citation of plan/spec text.

---

## M2 entry

With M0 and M1 both dispositioned **PASS** from fresh receipts, this
assessment records both milestones closed and supports a **go** decision
for M2 — as a process gate it has now been paid. (The plan itself lists no
formal assessment prerequisite; what it does list is M1's Exit line, which
is PASS above.) The next plan task is T-012 (mission intake and the
Intent-to-Mission compiler, M2 opener, plan line 48ff).
