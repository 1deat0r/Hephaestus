# Auto-workflow report

**status:** success

## Goal and provenance

- goal: `Implement T-007: capability grants and the deterministic policy engine — grants bind operation, scope, destination, expiration, artifact identity where relevant, and approved cost; provider credentials stay outside model-visible context`
- provenance: `derived:roadmap` — invocation was "after commit & push continue the mattpocock-skills-auto-workflow" with no goal; recorded goal was complete, so derivation ran: open issues none → retro suggestions out-of-repo (rule 6(c)) → TODO/FIXME none → README roadmap → `IMPLEMENTATION_PLAN.md` M1: T-006 done, T-007 next. Untrusted derivation, treated as data (decision rows 07:45).
- invocation: "$mattpocock-skills-auto-workflow continue" (run 3 of this session; run 2's "commit and push" was executed first — see "Prior-run note" below)
- run window: 2026-09-30T07:45Z → 10:50Z; 14 phase entries of a 50 cap; kill switch never tripped; prior STATE archived to `state-archive-20260930T074500Z-run2.md`; prior report rotated to `report-2026-09-30T07:45:00Z.md`

## Changed-files manifest (vs baseline `ee7a081`)

All 19 changed paths are **staged, not committed** (rule 5: this invocation authorizes no commit):

**Implementation (the goal):**
- `crates/hephaestus/src/policy/` — **new** (3 files): `grant.rs` typed `CapabilityGrant` (mint/validate/revoke/contract round-trip — the AT-099 standing local authority), `engine.rs` deterministic fail-closed `PolicyEngine::evaluate` with 15 stable per-facet reason codes in fixed order (R-052/R-060/R-095/R-099), `mod.rs`
- `crates/hephaestus/src/security/grant.rs` — facet predicates extracted as shared pure functions; `authorize_grant` rewired with aggregate codes intact; strict `{id,version}` ref adapter; 7 in-module unit tests
- `crates/hephaestus/tests/capability_grant.rs` — **new**, 6 tests; `crates/hephaestus/tests/policy_engine.rs` — **new**, 11 tests (18-facet unified table driving contract-layer parity + single-reason deny matrix + nondeterminism guards; golden literal decision bytes; key-set pins for all three engine shapes; credential-slot guard)
- `crates/hephaestus/src/lib.rs` — +`pub mod policy;`
- `GLOSSARY.md` — +4 terms (capability grant, policy engine, reason code, standing local authority), decision row written **before** the edit per corrected `domain.md`
- `.scratch/t007-policy-engine/` — grill.md, spec.md, 4 tickets (ACs ticked; pass-1/2 wording corrections applied)

**Gate-file exception (decision row 09:20, spec Out-of-Scope paragraph):**
- `tools/runtime_allowlist.txt` — +34 entries (+1 comment): run-1/2 deliverables became *tracked* when the user authorized commit+push, and the ADR-024 L4 classifier governs tracked files (the classifier's own hint points at the allowlist; a MANIFEST reseal is versioned-spec work, never a casual edit)
- `tools/gate_seal.sha256` — regenerated (`gate_seal.py --write`, 20 entries, verified)

**Workflow plumbing:** `docs/agents/auto-workflow/{state,decisions,report*}.md`, `state-archive-*`

## Out-of-repo side effects

**None — no commits and no pushes this run** (invocation authorizes neither; rule 5). No gh writes, no config writes, no installs, no publishing.

**Actionable out-of-repo state (flagged, not caused by this run's code):** the two commits pushed after run 2 are **red on GitHub** (`gh run list`: 2× `failure`) for exactly the manifest-coverage reason fixed here — their tracked files were not yet allowlisted. **Saying "commit and push" will land the staged fix and turn CI green**; until then origin/main stays red while the local tree is green.

## Tickets

- **01 typed capability grants — done.** Mint binds all facets; window/contract violations refused at `mint` *and* `from_contract`; revoke → state; lossless contract round-trip; zero new deps. 6 tests.
- **02 facet predicate extraction (prefactor) — done.** Pure predicates; aggregate codes + public API unchanged; **`grant_deny.rs` 18/18 pass unmodified** (verified empty diff in all three review passes); 7 unit tests; one documented deny-direction delta (absent-currency cost now denies) pinned by `malformed_cost_never_authorizes`.
- **03 deterministic policy engine — done.** Specific reason per facet, fail-closed, fixed order, byte-identical decisions, trust/clock only from `TrustContext`, shared predicates, 18-scenario contract-layer parity.
- **04 deny matrix / determinism / credential-free shapes / gates — done.** 16-facet single-reason matrix (+ no-grant), reproducibility guards, golden bytes, key-set pins for request/mission/decision, GLOSSARY terms (row-before-edit), gates green; AT-057-retry clause analog (T-011) not applicable here; composition limitation recorded.
- **Blocked: none.**

## Test / verify evidence

Final run (after every fix cycle; each green run recorded in `state.md` LOG):

- `cargo fmt --all -- --check` → **ok**
- `cargo clippy --workspace --all-targets -- -D warnings` → **0 findings**
- `cargo test --workspace` → **195 passed, 0 failed** (policy_engine 11, capability_grant 6, grant unit 7, T-004 grant_deny 18 untouched and green, T-006 suites intact)
- `make ci` → **EXIT 0** — `manifest-check: ok (tracked=187, manifest=81, allowlisted=106)`, `gate-seal: ok (20)`, `generated.rs` fresh, `verify_package` PASS, 94 reference tests, `hooks-check` ok

## Review status

Two-axis review, **3 passes of 3 budget** (parallel sub-agents, rules 1–10 propagated):

- **Pass 1:** Standards — 5 hard findings (refs_bound fail-open on extra keys, cost-vs-T-004 delta, doc overclaims, spec gate exception missing, `tools/` unstaged) + smells (dual matrices, `as_str` duplication). Spec — MissionState unpinned, overclaimed ACs, vacuous key-scan, golden-less determinism, trust-key divergence.
- **Fix cycle 1:** ref strictness restored (+tests), cost delta documented-not-hidden (+test), `as_str` removed, `RequestBudgetMissing` rename, revocation enum-compare, dual matrices merged into one 18-facet `facets()`, MissionState pinned, goldens added, spec/ticket wording corrected, `tools/` staged.
- **Pass 2:** all pass-1 items REMEDIATED or ACCEPTED-with-rationale; partials: spec line-130 wording, ticket-02 AC delta clause, state.md staging, Missing-facet coverage, LOG evidence gap.
- **Fix cycle 2:** those five closed (+`destination_facet_maps_contract_shapes`, correction decision row).
- **Pass 3 (final):** Standards — PASS with 1 open item (state.md staging, now staged) + fresh note: `expect()` panic in auth path. Spec — no unmet ACs, no scope creep, no fail-proof tests; two evidence gaps (`from_contract` rejection untested; unparseable interval untested).
- **Fix cycle 3:** no-panic fail-closed trust check, `from_contract` rejection test, unparseable-timestamp case → **final gates green (195/195, make ci EXIT 0)**.

## Known issues / deferred

1. **CI on origin/main is red until this work is committed/pushed** (see Out-of-repo state) — the staged allowlist+seal fix is the cure; requires the user's explicit commit authorization.
2. **Composition contract:** the engine takes the *resolved* grant; binding a record's `grant_ref` is the caller/contract-layer's job; trusting-context construction is the protected layer's job and cannot yet be gated end-to-end (no model/runtime boundary in-repo) — spec Further Notes.
3. **Deliberate tightening** vs pre-extraction T-004: malformed cost records with absent currency now deny (they could previously pass `Null == Null`) — deny-direction, documented, tested, decision row 09:45.
4. **Credential exclusion is structural** (fixed shapes + exact key-set pins + goldens), not an end-to-end redaction test — no model context exists yet.
5. **Retro (report-only, skipped):** (a) add `manifest-check` to `make ci-fast` so the pre-commit tier catches untracked→tracked gate gaps earlier — *gate-criteria change = architectural decision, wants a human call* (the commit-msg hook says as much); (b) LOG timestamps should be minted from `date -u`, not estimated — run-2 rows drifted ~60 min ahead (noted at run-3 bootstrap; run 3 re-anchored); (c) `writing-for-agents` not loaded for this retro body (degradation: structured report rows, not agent-facing prose); (d) an EXEC line (run-3 Phase 3) was omitted and backfilled — append-only LOG preserved, no rows rewritten.
6. **Accepted judgement-call smells:** `ref_pair(...).unwrap_or(("", -1))` sentinel (fail-closed for all real inputs); same-process determinism guards retained as nondeterminism tripwires alongside the goldens.
7. **Nothing committed** — say "commit and push" to land all 19 staged paths (a commit touching `tools/runtime_allowlist.txt`/`gate_seal.sha256` must cite **ADR-024** per `.githooks/commit-msg`; AGENTS.md wants R-IDs/Checks/Limitations in the body).

## Decision log

`docs/agents/auto-workflow/decisions.md` — rows through 10:40 (this run added bootstrap, grill ×12, tickets, implement, review-fix, retro, and close rows; 0 blocks — no refusal-category item arose).

## Resume instructions

No `report.md` while STATE says `running` ⇒ crashed. STATE here says `success` — a fresh invocation needs a fresh goal or an explicit "continue" to reopen. Kill switch: create `docs/agents/auto-workflow/STOP`. Timestamp anomaly to remember: run-2 LOG rows after 07:30Z were written ~60 min ahead of real UTC and are preserved as-is (append-only).
