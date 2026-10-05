# M2 + M3 + M4 Exit Assessment

**Date:** 2026-10-02 (UTC)
**Scope:** M2, M3, M4 exit criteria — IMPLEMENTATION_PLAN.md lines 64, 82,
96 (plus the T-033 mandatory-core clause at line 98). **M5/M6 exits are
deliberately not assessed** (not assessed → not claimed; inventing
dispositions would be fabrication).
**Method:** fresh commands run in this session; every `PASS` points at a
receipt (a named test or a CLI/e2e run) a skeptic can re-run. This is the
successor to `docs/M0_M1_EXIT_ASSESSMENT.md` in method and honesty rules.
**What this document is NOT:** it does not change any release label, does
not establish scientific validity, production security, or performance,
and does not substitute for the per-run workflow reports under
`docs/agents/auto-workflow/`.

---

## M2 exit assessment (IMPLEMENTATION_PLAN:64)

> Exit: broad domain-only goals create grounded opportunities and
> testable hypotheses on hidden synthetic worlds, without target
> mechanisms in prompts or fixtures visible to workers. Generation cost
> and failure modes are recorded. This is a hypothesis-generation
> milestone, not proof of invention quality in the wild.

| # | Clause | Disposition | Evidence (fresh this session) |
|---|--------|-------------|-------------------------------|
| 2.1 | Broad domain-only goal → grounded opportunities → testable hypotheses | **PASS** | `cargo test --test e2e_m2` → 1 passed: goal → mission compile → ingested world-derived trace → span-grounded opportunity(s) → operator mechanisms → `validate` → `Readiness::TestReady`, with a byte-identical twin-run receipt (no seed hypothesis supplied by the caller) |
| 2.2 | On hidden synthetic worlds; no target mechanisms in prompts/fixtures visible to workers | **PASS** | `cargo test -p synthetic-evaluator --test e2e_trace_fixture` → 1 passed (the control-plane fixture file regenerates byte-identically from `load_corpus()` worlds, inside the evaluator's own crate); `cargo test --test evaluator_access` → 4 passed (manifest AND resolved-lockfile AND source-name guards prove the control plane never links the hidden evaluator — data only) |
| 2.3 | Generation cost and failure modes are recorded | **PASS (bounded-generation sense)** | `e2e_m2`: every emitted mechanism asserts non-empty `failure_modes`; generation is bounded (`max_per_operator = 8`) with `rejected_applicability`/`heuristic_rejection_audit` recorded in the serialized sweep receipt. Currency/provider cost is N/A in the fixture chain (no provider spend exists to record — noted, not implied) |
| 2.4 | "Hypothesis-generation milestone, not proof of invention quality in the wild" | **RECORDED — unchanged** | Label restated in *Limitations* below; nothing here contradicts it |

**M2 disposition: PASS** (four clauses dispositioned from fresh receipts).

---

## M3 exit assessment (IMPLEMENTATION_PLAN:82 + the T-033 clause at 98)

> Exit: end-to-end scenarios cover supported, contradicted, inconclusive,
> invalid, and blocked outcomes. A negative result is exported honestly.
> Candidate workers cannot inspect the sealed evaluation manifest. The
> same artifact can be independently reproduced under the declared
> environment.
>
> T-033 (mandatory core): … Demonstrate a subsequent mission using the
> improved persisted champion … A functioning invention loop without
> autonomous bounded self-improvement cannot claim M3 completion.

| # | Clause | Disposition | Evidence (fresh this session) |
|---|--------|-------------|-------------------------------|
| 3.1a | Supported outcome, end-to-end | **PASS** | `cargo test --test e2e_m3` (1/2): trace → qualified `mean-difference-z-interval` (hand-computed in `method_registry`) → frozen plan → `interpret` → `Valid`+`Supported` → dossier export with `Measured` label + fixture receipt |
| 3.1b | Contradicted outcome | **PASS** | `e2e_m3` (2/2): theta above interval → `Contradicted`, exported with `negative_kind: UnsupportedMechanism` attached; plus `result_interpreter::interval_entirely_below_contradicts_overlap_inconclusive` |
| 3.1c | Inconclusive outcome | **PASS (named unit)** | `result_interpreter::interval_entirely_below_contradicts_overlap_inconclusive` (overlap case), `reproduction_record::negative_and_inconclusive_outcomes_are_first_class` |
| 3.1d | Invalid outcome | **PASS (named unit)** | `result_interpreter::invalid_execution_short_circuits_to_not_assessed`; `experiment_compiler::digest_mismatch_rejected` |
| 3.1e | Blocked outcome | **PASS (named unit)** | `experiment_compiler::blockers_are_precise_never_imagined` (§13:262 precise blockers, never imagined outcomes) |
| 3.2 | A negative result is exported honestly | **PASS** | `e2e_m3` (2/2): contradicted result exported with its named negative kind and the verdict in the payload — nothing smoothed |
| 3.3 | Candidate workers cannot inspect the sealed evaluation manifest | **PASS** | `cargo test --test sealed_deny` + `cargo test --test evaluator_access` (both inside this session's green `make ci`) |
| 3.4 | The same artifact can be independently reproduced | **PASS (named unit)** | `reproduction_record::recording_enforces_independence_and_endpoint_registration`, `…::negative_and_inconclusive_outcomes_are_first_class`, `…::twin_run_byte_identical` |
| 3.5a | Demonstration loop: non-seeded opportunity → bounded proposal → protected evaluation → persisted champion → later mission uses it → restart preserves state → injected regression → rollback, disqualified challengers undeployed | **PASS** | `cargo test --test champion_reuse` → 3 passed (`r119_full_loop_later_missions_use_the_persisted_champion`, `r119_disqualified_challengers_stay_undeployed`, `proposal_edge_refuses_unseeded_and_self_budgeted_candidates`): bound 8 → championed 16 (digest-verified payloads) → byte-identical post-restart receipt → rollback restores 8; false/confounded/over-budget/permission-expanding each refused by name. Proposal edge refuses unseeded/self-budgeted candidates (a vacuous-`any` bug found by this very test was fixed during construction) |
| 3.5b | Crash-mid-deployment reconciliation (interrupted deployments) | **PASS** | `cargo test --test crash_reconciliation` → 4 passed: corrupt ledger REFUSES (the silent-empty `from_json` hole found and fixed — fail-closed `Result`, all callers updated); atomic persist notes stale temps; BOTH write-ahead crash windows reconcile exactly-once (pending→replay, append-without-cleanup→dedup); torn pending discarded loudly with its reason retained |
| 3.5c | Recorded triggers + bounded continuous canary/monitor (R-115 triggers; R-118 monitor stops the rollout) | **PASS** | `cargo test --test trigger_canary` → 3 passed: triggers persist on the ledger across restart (`record_trigger`, typed `EmptySubject` refusal); cycles are bound-typed (stop rule / candidate cap / budget cap / deadline refusals) with the recorded `NoJustifiedChange` outcome and the single proposal path; the monitor runs its declared bound, stops a breached rollout with a real rollback receipt naming the indicator, and refuses an unproven incumbent BEFORE checking. `champion_reuse` (3/3) now drives its injected regression THROUGH this monitor. Honest scope: a bounded per-deployment monitor loop with declared stop rules — R-118/115 require recorded triggers and a stopping monitor, not an out-of-process forever-daemon; the daemon question is recorded as scope, not as a gap |

**M3 disposition: CLAIMED (complete)** — every clause PASSes from
fresh receipts: 3.1 (five outcome classes), 3.2 (honest negative
export), 3.3 (sealed manifest), 3.4 (independent reproduction), 3.5a
(R-119 demonstration loop), 3.5b (crash reconciliation), 3.5c
(recorded triggers + bounded monitor). Limitations that travel with
the claim: the monitor is a bounded per-deployment loop with declared
stop rules (scope decision recorded in 3.5c — not an out-of-process
daemon); self-improvement gates function on fixture evidence, which
does NOT establish general benefit (R-119 permits shipping functioning
gates before benefit is established); the release label is unchanged —
M4 remains NOT claimed.

---

## M4 exit assessment (IMPLEMENTATION_PLAN:96)

> Exit: ship as EXPERIMENTAL unless evidence supports
> QUALIFIED_FOR_DECLARED_SCOPE. "Faster," "more novel," or "better"
> claims require the relevant measured baseline comparison and uncertainty.

| # | Clause | Disposition | Evidence (fresh this session) |
|---|--------|-------------|-------------------------------|
| 4.1 | EXPERIMENTAL-unless-qualified label machinery | **PASS (machinery only)** | `release_packet::scope_label_defaults_experimental_requires_all_inputs` (green inside `make ci`); `assemble_release` gates (R-077/078/076) green |
| 4.2 | Measured baseline comparisons for speed/novelty claims | **NOT-RUN** | Eval-suite arm *definitions* exist (`evalsuite`, R-074/R-075 tests green) but no campaign has measured them; model arms additionally require a provider behind a contract — a consent-gated decision (rules 6/8), not an agent default |
| 4.3 | Pilot/confirmatory campaign executed | **NOT-RUN** | `pilot::plan_pilot`/`ready_for_confirmation` machinery is tested; zero missions have been flown |
| 4.4 | An actual ship/release | **NOT-RUN** | Nothing has been shipped; no release label is being changed |

**M4 disposition: NOT CLAIMED** — only the label machinery is PASS; the
comparison, campaign, and ship clauses are NOT-RUN by design (consent and
campaign execution pending).

---

## Gate evidence (the suite that wraps every claim above)

- `make ci` → **EXIT 0** at assessment time: **592 passed / 0 failed**,
  `manifest-check: ok (tracked=604, allowlisted=523)`,
  `requirement-citations: ok (119 requirements, 117 cited, 2 reasoned
  allowlist entries)`, `md-links: ok (319 md files, 0 broken)`,
  `gate-seal: ok (22 gate files sealed)`.
- `make doc-check` → **EXIT 0**.
- Two incidents during this assessment, recorded not hidden:
  1. `subprocess_children_stay_inside_the_network_namespace` failed once
     under parallel load (host-level spawn EAGAIN — same class as
     run-14's local flake); standalone re-run → 1 passed, full gate green.
  2. `fork_bomb_stops_at_the_nproc_allowance` was ROOT-CAUSED as a latent
     TEST BUG, not a flake: its python payload used `{{ok}}/{{fail}}`
     (double braces → literal braces printed, so the completed-run parse
     could never succeed) and spawned serially (so the nproc bound was
     never reached) — the test had only ever passed via the wall-kill
     branch. Fixed with a concurrent 4-worker payload and real counts,
     plus a failure message that prints stdout; standalone 2× green and
     `sandbox_deny` 17/17.

## Campaign receipts (dev-roadmap tickets 01-04)

Fresh campaign receipts land here (tickets 01-04).

- Ticket 01 (fixture campaign driver): 20-mission loop with R-103
  denominator and budget receipts (commits a9b7698, 18169f8).
- Ticket 02 (real-corpus mission): `cargo test --test e2e_real_corpus`
  3 passed over repo docs (commit adf8a71, closed 06a58e7).
- Ticket 03 (docs domain pack): `cargo test --test docs_domain_pack`
  3 passed (commit b88fea4, closed c3a693f).
- Ticket 04 (canary-watch): `cargo test --test cli canary_watch`
  1 passed; twin runs byte-identical, stopped outcome with the
  rollback receipt restoring incumbent d8 (commit 7fc2649, closed
  4cb24e2).
- Full workspace suite at ticket-04 close: 82 suites ok, exit 0;
  `cargo doc` exit 0.

## Limitations and non-claims

- **Release label unchanged**: specification/engineering foundation, not
  autonomous invention. What is new is a fixture-chain E2E receipt
  (M2 PASS; M3 claimed complete with the limitations above) — not invention quality, not
  in-the-wild evidence, not statistical claims.
- **M3 claimed complete** as of this revision (all clauses PASS from
  fresh receipts — see the M3 table; scope/limitation lines travel
  with the claim); **M4 still not claimed** (no campaign, no baselines
  measured, nothing shipped).
- The hidden evaluator stays in its own permission scope: the control
  plane consumes checked-in data only (evaluator_access guards —
  never widened this session; one dev-dep attempt was caught and
  reverted by exactly that guard, recorded in the run-31 decisions).
- Model-baseline arms remain consent-gated (rules 6/8); no credentials
  were sought or used.
- Host sandbox spawn flake is environmental (evidence above), not a
  product regression; it has bitten CI/local runs intermittently this
  session and is recorded wherever it occurred.

## Resume pointer

The E2E chain lives in `crates/hephaestus/src/missionrun.rs` (shared by
`tests/e2e_m2.rs`, `tests/e2e_m3.rs`, and `hephaestus fixture mission-run
--trace <FILE>`); fixture provenance is enforced by
`synthetic-evaluator/tests/e2e_trace_fixture.rs`.
