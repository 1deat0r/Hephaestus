# Spec — Gate parity: pre-commit manifest-check + local doc-check

Status: ready-for-agent
Goal source: derived:retro (run-3 retro item (a); untrusted derivation recorded in decisions.md)
Grill record: `.scratch/gate-parity/grill.md` (10 questions, all self-answered)

## Problem Statement

Two checks that GitHub Actions runs were unreachable from any local tier.
The manifest-coverage classifier only ran inside `make ci`, so nothing a
developer does *before pushing* could catch a tracked-but-uncovered file —
run 3 pushed twice red for exactly that. The rustdoc `-D warnings` check
existed only as a remote-only workflow job, so no local command exercised
the one check that caught the run-3 intra-doc link; an agent following
`make ci` alone had no way to know the gap existed. The gate list itself
pointed at neither: `ci-fast` omitted the classifier, and the docs job
re-stated its command inline instead of calling the registry.

## Solution

Close both gaps inside the existing tier architecture, without touching
`make ci`'s latency budget:

1. **Pre-commit tier gains the classifier** — `manifest-check` joins
   `ci-fast`, so every commit (the hook runs exactly `make ci-fast`) fails
   locally, with the classifier's own hint, before anything is pushed.
2. **`doc-check` becomes a named target** running exactly the remote
   command, and the remote docs job calls `make doc-check` — one definition,
   two runners (single-gate-registry doctrine).
3. **ADR-025** records the tier change, its measurements, and the rejected
   alternatives; ADR-024's record is not edited (append-only register).

## User Stories

1. As a developer, I want `git commit` to fail when I stage a file that is
   neither in `MANIFEST.sha256` nor allowlisted, so that the failure happens
   at my machine with a hint, not as a red run after pushing.
2. As a developer, I want the classifier still listed explicitly in `make
   ci`'s target list, so that registry inventory stays readable at a glance.
3. As an agent, I want `make doc-check` to exist, so that the exact check
   the remote docs job runs is one command away instead of tribal knowledge.
4. As a maintainer, I want the docs workflow to call `make doc-check`, so
   that the command cannot drift between the Makefile and the workflow.
5. As a reviewer, I want the tier change recorded as ADR-025 with rejected
   alternatives and measured overhead, so that a gate-criteria change is a
   decision, not an accident.
6. As a reviewer, I want ADR-024's text left untouched, so that the register
   stays an append-only history with supersession stated in the new record.
7. As a user of the hot path, I want cargo doc *absent* from `ci-fast` and
   `ci`, so that the ADR-024 step-10 latency decision keeps holding.
8. As a maintainer, I want the gate seal regenerated after these edits, so
   that tamper detection keeps covering the files that define "green".
9. As a future committer, I want ADR-025's number to satisfy the commit-msg
   hook, so that the citation requirement is already answered when the change
   lands.
10. As an auditor, I want the negative probe and timings recorded in the
    state LOG, so that "the gate now catches this" is demonstrated, not
    asserted.

## Implementation Decisions

- **Wiring point:** the `ci-fast` target in the Makefile — the pre-commit
  hook already delegates to it and forbids duplicating the gate list.
- **Registry inventory:** `manifest-check` stays explicitly on the `ci`
  target list as well (double-run accepted; measured overhead recorded).
- **New target:** `doc-check` runs `RUSTDOCFLAGS="-D warnings" cargo doc
  --workspace --no-deps`, wired into **no** tier; a Makefile comment records
  why (ADR-024 step-10 latency budget).
- **Workflow single-sourcing:** the docs job's `run:` becomes
  `make doc-check`; everything else in the workflow is untouched.
- **Decision record:** ADR-025 appended to `docs/RUNTIME_DECISIONS.md`
  (runtime register — allowlisted, unsealed, not manifest-frozen), naming
  the two ADR-024 appendix rows it supersedes; ADR-024 not edited.
- **Gate seal:** regenerated once after all sealed-file edits
  (`tools/gate_seal.py --write`), covering `Makefile` and
  `.github/workflows/ci.yml`.
- **No mapped R-ID:** this is runtime gate hygiene; the eventual commit body
  says "no requirement ID — ADR-024/025 gate machinery" rather than
  inventing an obligation.
- **Dependencies/lockfiles:** none touched.
- **No commit/push this run** (rule 5) — work lands when authorized; a
  later commit must cite an ADR (ADR-025's number qualifies).

## Testing Decisions

- Config change: no unit-test seam. A good test here is *behavioral
  evidence at the make boundary*: the gate fails when it should and passes
  when it should, with the timing and output recorded.
- **Evidence set (state LOG, command + rc):**
  1. `make ci-fast` green with the classifier listed.
  2. **Negative probe:** `git add` a throwaway unlisted file →
     `make ci-fast` fails citing that file → probe removed from index and
     disk → green again. The probe is the only file staged/reverted by the
     test; no user file is ever staged or reverted.
  3. `make doc-check` rc=0 (the check that caught the run-3 rustdoc bug).
  4. `make ci` EXIT 0 (full tier, incl. seal check with the regenerated
     seal).
  5. `make ci-fast` timing before/after the change, recorded next to the
     ~50 ms estimate from grill Q6.
- **Prior art:** the run-3 red pushes themselves (CI logs recorded in
  `report-2026-09-30T10:50:00Z.md` and the fix commit messages) are the
  real-world failure this spec's probe reproduces deliberately.

## Out of Scope

- cargo doc in any tier (`ci`, `ci-fast`) — ADR-024 step-10 decision stands.
- `md-links` (or any network/transient check) in `ci-fast` — ADR-024's
  rejected-alternative rationale applies.
- Pre-commit hook edits (it already delegates), `periodic.yml`, the
  scheduled audit workflow, the external GitHub ruleset (owner-owned).
- Editing ADR-024's appendix in place (append-only register).
- Any `requirements.json`/traceability/glossary/schema change.
- ADR: none *beyond* ADR-025 — this implements the register's own tier
  doctrine; the record documents the criteria change, not a departure.

## Further Notes

- Intended friction (grill Q10): staging a new file without allowlisting it
  now blocks the commit — that is the gap closing. The failure message is
  the classifier's existing hint (allowlist vs versioned MANIFEST reseal),
  so the remedy is on screen when the gate fires.
- Limitation: `doc-check` stays out of all tiers, so a rustdoc regression
  is still caught *locally only when someone runs it* — the remote docs job
  remains the automatic backstop. Encoding `doc-check` into a tier would
  violate the latency decision this spec explicitly preserves.
- Limitation: the negative probe manipulates the git index (add/rm --cached
  of a throwaway file); it never touches committed history or user files.
