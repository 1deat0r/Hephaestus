# M5 + M6 Exit Assessment

**Date:** 2026-10-02 (UTC)
**Scope:** M5 and M6 exit criteria — IMPLEMENTATION_PLAN.md lines 106
and 116. Successor to `docs/M0_M1_EXIT_ASSESSMENT.md` and
`docs/M2_M3_M4_EXIT_ASSESSMENT.md` in method and honesty rules; with
this document every milestone exit line has a receipt bundle.
**Method:** fresh commands run in this session; every `PASS` points at a
named receipt a skeptic can re-run. NOT-RUN means not run — never
softened into a PASS.
**What this document is NOT:** it does not change any release label,
does not establish scientific validity or performance, and does not
substitute for the per-run workflow reports under
`docs/agents/auto-workflow/`.

---

## M5 exit assessment (IMPLEMENTATION_PLAN:106)

> Exit: each optional feature can be disabled. The core path stays
> functional. More parallelism is not automatically a better result.

| # | Clause | Disposition | Evidence (fresh this session) |
|---|--------|-------------|-------------------------------|
| 5.1 | Each optional feature can be disabled | **PASS** | Advisory: `advisory_provider` → 8 passed (NullProvider is the zero-provider core path — R-005); backend: `backend_adapter` → 7 passed incl. `tachyon_blocked_no_compatibility_claim` (optional backend stays OFF with no compatibility claim) and `contract_check_covers_all_clauses`; acceleration: `acceleration_eval` → 7 passed incl. `six_mechanism_kinds_typed` and `promoted_requires_value_guardrails_and_disablability` (disablability is a promotion precondition) |
| 5.2 | The core path stays functional | **PASS** | This session's full gates: `make ci` EXIT=0 with **602 passed / 0 failed** (NullProvider + LocalProcess + deterministic control plane all inside); `workspace_core` → 6 passed (control-plane views read persisted state) |
| 5.3 | More parallelism is not automatically a better result | **PASS** | `acceleration_eval::absent_infrastructure_is_not_evaluable_not_failure` (absent infra → honest `NotEvaluable`, never fabricated measurement), `matched_ablation_enforces_identical_envelopes` (value must be measured within guardrails to promote), `promoted_requires_value_guardrails_and_disablability` — promotion requires measured value, not parallelism |

**M5 disposition: PASS** (three clauses, fresh named receipts).

---

## M6 exit assessment (IMPLEMENTATION_PLAN:116)

> Exit: new capabilities are qualified separately rather than inheriting
> proof from the software domain.

| # | Clause | Disposition | Evidence (fresh this session) |
|---|--------|-------------|-------------------------------|
| 6.1 | New capabilities qualified separately, never inheriting proof | **PASS (machinery)** | `domain_packs` → 4 passed: `qualification_refuses_self_and_unqualified_oracle` (no self-qualification, oracle must be adjudicated), `measurement_units_and_simulator_validity`, `execution_class_separation`; module contract states proof is never inherited (domainpack/mod.rs:3) |
| 6.2 | T-034: external/genuinely independent reproduction of the first warranted candidate | **NOT-RUN** | The reproduction GATES are tested (`reproduction_record` → 4 passed: `warrant_requires_both_claims_and_first_candidate`, `recording_enforces_independence_and_endpoint_registration`, `negative_and_inconclusive_outcomes_are_first_class`) but NO candidate has warranted external reproduction — no such run exists. NOT-RUN, not PASS |
| 6.3 | Advanced T-033 extensions (fine-tuning etc.) | **N/A (optional by plan)** | The plan marks these optional and states M6 cannot waive the M3 core (now claimed — see M2_M3_M4 assessment); nothing to run for the exit line |

**M6 disposition: NOT CLAIMED COMPLETE** — clause 6.1 machinery PASS;
clause 6.2 (the actual external reproduction) is explicitly NOT-RUN
until a candidate's novelty/utility claims warrant it.

---

## Gate evidence (the suite that wraps every claim above)

- `cargo test` (fresh, this session): `advisory_provider` 8 passed,
  `backend_adapter` 7, `acceleration_eval` 7, `domain_packs` 4,
  `reproduction_record` 4, `workspace_core` 6 — all green.
- `make ci` → **EXIT 0, 602 passed / 0 failed** (manifest 624/543,
  `requirement-citations: ok (119/117/2)`, md-links 336/0, seal 22);
  `make doc-check` → EXIT 0.

## Limitations and non-claims

- **Release label unchanged**: specification/engineering foundation,
  not autonomous invention. M3 is claimed complete with its recorded
  limitations (see the M2–M4 assessment); **M4 and M6 are NOT
  claimed**, and M5/M6 machinery PASS does not imply measured
  acceleration value or any external reproduction.
- Optional-provider integrations (real Jev/Tachyon endpoints) remain
  contract-tested stubs/blocked statuses — no live provider was used
  (consent-gated; rules 6/8).
- No campaign, no baseline measurements, no shipped release.

## Resume pointer

Milestone-exit receipt bundles now exist for M0–M6:
`M0_M1_EXIT_ASSESSMENT.md`, `M2_M3_M4_EXIT_ASSESSMENT.md` (M3
claimed), this document (M5 PASS, M6 not claimed).
