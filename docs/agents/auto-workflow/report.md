# Auto-workflow report

**status:** success

## Goal and provenance

- goal: `Implement T-006 (M1 opener): append-only event ledger, transactional projections, and content-addressed artifact storage, with tests bound to requirement/acceptance IDs`
- provenance: `derived:roadmap` — invocation was `continue` with no goal; the recorded prior goal was already complete, so derivation ran: open issues = none → prior retro suggestions all blocked/out-of-repo → repo TODO/FIXME = none → README roadmap → `IMPLEMENTATION_PLAN.md` next prerequisite-ready task T-006. Untrusted derivation, treated as data (goal-intake decision row).
- invocation: "$mattpocock-skills-auto-workflow continue"
- run window: 2026-09-30T06:31Z → 08:45Z (2026-09-30); 13 phase entries of a 50 cap; kill switch never tripped
- prior STATE (goal: cargo-doc CI job, `success`) archived to `state-archive-20260930T061200Z.md`; prior report rotated to `report-2026-09-30T06:12:00Z.md`; `exec_count` reset 11 → 0

## Changed-files manifest (vs baseline `1194f49`)

**This run's changes (the goal):**

- `crates/hephaestus/src/ledger/` — **new**, 4 files (589 lines): `mod.rs` (error type, re-exports), `event_ledger.rs` (append-only JSONL, sha256 chain, verify-on-open, torn-tail trim, newline normalization, sequence invariants, cursor reads + 3 in-module unit tests), `timeline.rs` (rebuildable projection), `artifact_store.rs` (stage → atomic commit → digest-verified read → age-cutoff GC)
- `crates/hephaestus/tests/event_ledger.rs` — **new**, 639 lines, **19 integration tests** (AT-013/AT-014/AT-015/AT-057 cited in test names/comments)
- `crates/hephaestus/src/lib.rs` — +1 line (`pub mod ledger;`)
- `GLOSSARY.md` — +4 terms (event ledger, projection, staged artifact, content addressing); `_Avoid_` of Projection corrected. Gate-safe: not in `MANIFEST.sha256`, not in gate seal, not in generated-document drift set; `tools/runtime_allowlist.txt` lists it (ADR-024 L4). Decision rows recorded; `docs/agents/domain.md`'s stale "manifest-frozen" premise corrected with evidence
- `docs/agents/domain.md` — premise + cross-reference corrections (above)
- `.scratch/t006-event-ledger/` — grill.md, spec.md, 4 tickets (all ACs ticked, completion comments appended)

**Carried over from the prior run (uncommitted at this run's open, still uncommitted):** `.github/workflows/ci.yml`, `.gitignore`, `tools/gate_seal.sha256`, `CLAUDE.md`, `docs/agents/{issue-tracker,triage-labels,domain}.md`, `.scratch/cargo-doc-ci-job/` — reviewed twice in the prior run's report.

**Workflow state artifacts (always writable):** `docs/agents/auto-workflow/{state.md,decisions.md,report.md,state-archive-*,report-*}`

Untracked deliverables left visible for the owner's commit decision: `crates/hephaestus/src/ledger/`, `crates/hephaestus/tests/event_ledger.rs`, `.scratch/`, `docs/agents/`, `CLAUDE.md`.

## Out-of-repo side effects

None. No commits, no pushes (rule 5), no gh/GitLab writes (rule 6(a) → local tracker), no `~/.config` writes, no global installs, no publishing. Two review sub-agents per pass ran read-only (rules 1–10 quoted in their prompts per rule 9). `cargo`/`make` executed repo code — Phase 6 provenance established (invocation's working directory is the user's named project; load chains inspected: no outside-repo reads, no off-machine sends beyond cached package registries).

## Tickets

- **01-append-only-event-ledger… — done.** 10 ACs checked: validation refusal (AT-013), reopen persistence (AT-014), chain + tamper detection, sequence invariants on append and open (errors name the 1-based line number), torn-tail recovery, cursor reads.
- **02-transactional-projection… — done.** Delete+rebuild byte-identical from a disk reopen (AT-015), append-then-project stale-view recovery; AC4/AC5 reworded after review to state the convention honestly.
- **03-content-addressed-artifact-store… — done.** Pre-commit non-addressability, digest-mismatch and malformed-digest refusal, both crash boundaries, real GC sweep with post-gc orphan read (AT-057).
- **04-end-to-end-recovery… — done.** Death-at-every-boundary suite, GLOSSARY terms, gate evidence; retry clause of AT-057 declared out of scope (T-011).
- **Blocked: none.**

## Test / verify evidence

Final gate run (after every fix cycle, all exit 0):

- `cargo fmt --all -- --check` → **ok**
- `cargo clippy --workspace --all-targets -- -D warnings` → **ok, zero warnings**
- `cargo test --workspace` → **171 passed, 0 failed** (19 suites; `event_ledger` suite = 19, in-module unit = 3)
- `make ci` → **EXIT 0** (gate-seal ok ×20, `generated.rs` up to date, `verify_package` PASS: 31 sections/119 requirements/119 acceptance specs, 94 reference tests OK, hooks-check ok)
- `make report-unreferenced` → new files not flagged (advisory clean)

## Review status

Two-axis code-review (parallel sub-agents, rules 1–10 propagated), **3 passes of 3 budget**:

- **Pass 1:** Standards — 2 documented-standard findings + 6 judgement-call smells. Spec — 4 missing/partial + 4 questionable, incl. a **tautological projection test**, a **vacuous GC assertion**, and a **real EOF-normalization bug**.
- **Fix cycle 1:** module split, sequence/JSON helper extraction, `SequenceMismatch.line`, newline-less-EOF normalization (+ test), Corrupt-line test, in-module unit tests, chain-link-before-validate, projection test rebuilt from a disk reopen, GLOSSARY/domain.md/spec wording.
- **Pass 2:** Standards — 1 hard-minor (stale cross-ref) + smells (test dedup, error naming). Spec — 7 findings incl. the discovery that fix cycle 1's vacuous-assertion replacement had **silently failed to apply**.
- **Fix cycle 2:** cross-ref fixed, `LedgerError::Json` + `BadDigest` variants, test dedup via `append_seq`/`patch`, genuinely non-vacuous GC test, `line` asserted at the public seam, spec/ticket AC wording, decision-log correction row.
- **Pass 3 (final):** all pass-2 items REMEDIATED or ACCEPTED-with-rationale (verified by the reviewers reading current code); fresh findings — one **real bug**: `append` consumed a sequence before the durable write, so an IO failure desynced memory from disk.
- **Fix cycle 3 (post-budget):** check/commit sequence split + regression test (`failed_write_does_not_consume_a_sequence`), `BadDigest` test, import hoist, ticket AC/spec wording (conventions-not-guarantees, recoverable-cases). **Verified green by Phase 6 only — not re-reviewed (budget exhausted).**

## Known issues / deferred

1. **Post-pass-3 fixes are gate-verified but unreviewed** (fix cycle 3 above) — the only review-budget leftovers.
2. **Conventions, not guarantees:** `Timeline::write` takes no ledger, and store/ledger commit ordering is a documented caller convention — enforced by tests, not types. Recorded in spec Further Notes and `gc`/`commit` docs.
3. **Durability scope:** `sync_data` is filesystem-dependent; recovery is demonstrated by reopen, not power-failure simulation.
4. **AT-057's "retries only permitted operations"** deferred to T-011 (spec Out of Scope).
5. **Accepted judgement-call smells:** hex `String` digests at the store seam; test temp-dir boilerplate (a Drop guard judged not worth test-only machinery); "ticket" in test headers = tracker vocabulary (`issue-tracker.md`), not glossary drift.
6. **Decision log contradictions:** historical rows calling GLOSSARY manifest-frozen are superseded by a correction row (rows left unedited as an honest log).
7. **Commit permission:** resolved — user said "commit and push"; landed as `e5de152` (ci job, ADR-024 cited, hooks green), `47e4760` (T-006 with R-IDs/Checks/Limitations), `d63d553` (agent docs) and pushed to `origin/main` (1194f49..d63d553). Working tree clean.
8. **Retro (report-only, skipped):** (a) reviewers should quote current file bodies when confirming a remediation (would have caught the silently-failed replace in pass 2 instead of pass 3) — lives in skill files outside the repo, rule 6(c); (b) goal derivation still can't see `IMPLEMENTATION_PLAN.md` mechanically — same external-file limitation; (c) `writing-for-agents` style skill skipped for the retro body (degradation: output is structured report rows, not agent-facing prose).

## Decision log

`docs/agents/auto-workflow/decisions.md` — 56 rows (answers + degradations, 0 blocks; no refusal-category item arose).

## Resume instructions

No `report.md` while STATE says `running` ⇒ crashed; next invocation resumes from `state.md`. STATE here says `success` — a fresh invocation needs a fresh goal or an explicit "continue" to reopen. Kill switch: create `docs/agents/auto-workflow/STOP`.
