# Grill — Gate parity: pre-commit manifest-check + local doc-check

Self-interview (auto-workflow Phase 2). Every question the `grilling` skill
would put to the human, answered from codebase evidence + goal via the
auto-answer policy. Frontier emptied in one round.

Goal under grill: *Close the local/remote gate gap behind run 3's red push —
add manifest-check to the pre-commit tier (`make ci-fast`) and a local
doc-check target mirroring the remote docs job, without changing `make ci`'s
latency budget (ADR-024 step 10 stands); record the gate-criteria change as a
runtime decision.*

## Evidence gathered before answering

- `.githooks/pre-commit` runs **exactly `make ci-fast`** and says "Gate list
  lives in the Makefile (ci-fast target) — do not duplicate it here": wiring
  `ci-fast` is sufficient; the hook needs no edit.
- `Makefile`: `ci: … manifest-check … ci-fast hooks-check` (the classifier is
  already in the full tier) and `ci-fast: gen-check verify test-package
  conflict-staged` (the pre-commit tier — **no** manifest-check: the exact
  gap that let run 3 push red twice before any local tier noticed).
- `.github/workflows/ci.yml` docs job runs `RUSTDOCFLAGS: -D warnings cargo
  doc --workspace --no-deps` — **remote only**; `make ci` never exercised it
  (run 3's unresolved intra-doc link slipped past every local run).
- ADR-024 (in `docs/RUNTIME_DECISIONS.md`) owns the tier architecture; its
  appendix tier table lists "manifest-classifier → `make ci`" and "cargo doc
  → separate CI job, later" — explicitly labelled *non-normative*.
  ADR-025 is the next free number.
- Prior decision (run 1 goal, ADR-024 step 10): cargo doc is
  **deliberately not** part of `make ci` — "local latency budget stays
  untouched". That decision stands (goal constraint).
- `Makefile` and `.github/workflows/ci.yml` are sealed gate files
  (`tools/gate_seal.sha256`); a later commit touching them needs an
  ADR-NNN citation per `.githooks/commit-msg`.
- ADR-024 rejected alternative, verbatim: "running every check on every
  commit (mid-task transient link failures create retry pressure; tiering
  keeps commits fast…)" — that rationale was about *link* checks; the
  manifest classifier is deterministic (no network, no mid-task churn
  beyond staged-but-unallowlisted files, which is precisely what it should
  catch).

## Round 1

❓ **Q1 — Scope cut**: what exactly lands, and what explicitly does not?

➡️ **In:** (1) `manifest-check` added to `ci-fast`; (2) a `doc-check` Makefile
target running the docs job's exact command; (3) the remote docs job calls
`make doc-check` instead of the raw command (single-gate-registry doctrine —
the same reason the gates job calls `make ci`; also kills the "duplicated CI
gate lists will drift" finding class); (4) **ADR-025** recording the tier
change with rejected alternatives. **Out:** cargo doc in `make ci` or
`ci-fast` (ADR-024 step-10 latency decision stands, stated in the goal);
`md-links` into `ci-fast` (ADR-024's transient-failure rationale applies to
it); any pre-commit hook edit (hook already delegates); `periodic.yml`;
ruleset/audit items (owner-owned). *source: agent-default · plan + ADR-024
evidence.*

❓ **Q2 — `ci`'s explicit `manifest-check` entry**: keep or drop now that
`ci-fast` carries it?

➡️ **Keep both.** `ci` is documented as "the single gate registry" and its
target list doubles as the human-readable inventory of every gate; removing
the explicit entry hides it from that list. The cost is one extra ~50 ms
deterministic run at the end of a minutes-long target — measured during
verification and recorded. Runner-up — remove for dedup — rejected:
inventory visibility beats 50 ms. *source: agent-default · Makefile comment
evidence.*

❓ **Q3 — Decision record**: where, and does ADR-024 get edited?

➡️ **New ADR-025 appended to `docs/RUNTIME_DECISIONS.md`** (the runtime
register, allowlisted, not sealed, not manifest-frozen); one-paragraph style
with rejected alternatives, per the register's convention. **ADR-024's text
is not edited** — registers are append-only records; ADR-025 states which
two appendix rows it supersedes ("manifest-classifier → ci-fast as well",
"cargo doc → local `doc-check` target, remote job calls it"). *source:
agent-default · domain.md register rules + RUNTIME_DECISIONS style.*

❓ **Q4 — `doc-check` semantics**: what does the target run, and where is it
wired?

➡️ `doc-check:` runs `RUSTDOCFLAGS="-D warnings" cargo doc --workspace
--no-deps` — byte-for-byte the remote job's command (which becomes `make
doc-check`). It joins **no tier**: not `ci` (latency decision), not
`ci-fast` (hot path). It is a named, discoverable command for humans and
agents — the navigation pointer run 3 lacked. Comment in the Makefile
records why it is unwired. *source: agent-default · ci.yml evidence + goal
constraint.*

❓ **Q5 — Tests**: what does "tested" mean for a config change?

➡️ **No unit-test seam (config only); behavioral evidence instead:**
(1) `make ci-fast` green with the classifier listed; (2) **negative probe** —
`git add` a throwaway unlisted file, assert `make ci-fast` FAILS with the
classifier's uncovered hint, remove it, assert green again; (3) `make doc-check`
rc=0 (this is the check that caught the run-3 rustdoc bug); (4) `make ci`
still EXIT 0; (5) timing: `make ci-fast` measured before/after to record the
real overhead next to the ~50 ms estimate. Probes touch only the probe file
and the index — no user file is ever staged or reverted. *source: agent-default
· evidence over assertion.*

❓ **Q6 — Latency claim**: is adding a check to the *pre-commit* tier
consistent with "keep commits fast"?

➡️ **Yes with numbers.** ADR-024's tiering intent is "tiering keeps commits
fast"; one deterministic ~50 ms set-comparison keeps that intent (the
rejected alternative was *every check*, especially transient ones). The
measurement from Q5(5) goes into the LOG and ADR-025 — if it surprises on
the wrong side, the fallback is documented in the decision record rather
than silently shipped. *source: agent-default · ADR-024 rationale.*

❓ **Q7 — Gate-file changes**: seal and future commit mechanics?

➡️ `Makefile` + `ci.yml` are sealed → `python3 tools/gate_seal.py --write`
after edits (seal is itself sealed — one regeneration covers both). Any
later commit must cite an ADR (hook: `ADR-[0-9]{3}` anywhere in the message)
— ADR-025's own number satisfies it. **No commit this run** unless the user
authorizes. *source: agent-default · rule 5 + commit-msg hook.*

❓ **Q8 — GLOSSARY / requirements / traceability?**

➡️ **No new glossary terms** (pre-commit tier, gate registry, doc-check are
already register/Makefile vocabulary, not product vocabulary);
**no `requirements.json`/traceability changes** (runtime gate hygiene has no
mapped obligation — the eventual commit body says so explicitly rather than
inventing an R-ID); **no schemas/generated code touched**. *source:
agent-default · AGENTS.md "identify requirement IDs" satisfied by honest
absence.*

❓ **Q9 — Ticket shape**: one ticket or several?

➡️ **One ticket.** The change is a single coherent vertical slice (two
Makefile lines + one ci.yml line + one ADR + seal + evidence), every piece
must land together to keep CI green at any commit point, and splitting would
create two intermediate seal states for no independence. Dedup: STATE has no
tickets for this spec. *source: agent-default · vertical-slice sizing.*

❓ **Q10 — Ordering risk**: could `ci-fast` now block legitimate commits?

➡️ **By design, yes — fail-closed:** a newly tracked file not yet
allowlisted (or not yet resealed into MANIFEST for spec-package files) blocks
the commit with the classifier's own hint. That is the gap closing, not
friction: run 3's red pushes were exactly the cost of *not* blocking. The
spec records this as intended behavior; the negative probe in Q5
demonstrates it. *source: agent-default · classifier hint text.*

## Frontier status

Empty. No refusal-category item (no secrets, money, legal, auth grants,
external publication); nothing deferred to a later round.
