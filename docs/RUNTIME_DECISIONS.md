# Runtime implementation decisions

Architecture decisions for the Hephaestus runtime, recorded from T-001 onward.
Style follows `docs/DECISIONS.md`; rejected alternatives included.

## ADR-019 — Runtime decisions live in this file, not `docs/DECISIONS.md`

`docs/DECISIONS.md` (ADR-001–018) is covered by `MANIFEST.sha256` and is part
of the frozen v1.2 evidence envelope: appending to it would break package
integrity verification. Runtime-era decisions are therefore recorded here, in
the same one-paragraph style, and this file is deliberately outside the
manifest. Rejected alternative: editing `docs/DECISIONS.md` and regenerating
the manifest — that would rewrite reviewed v1.2 evidence.

## ADR-020 — The hidden evaluator's permission scope is the Cargo dependency graph

The synthetic-world evaluator (T-003) lives in `crates/synthetic-evaluator`,
which `crates/hephaestus` does not depend on; `tests/evaluator_access.rs`
denies manifest, lockfile, and source-level paths to it. Workers and the
control plane receive `{input, output}` observation projections only.
Rejected alternative: a visibility convention inside one crate (not
enforceable); process isolation (adopted later at T-010/T-022, where OS
mechanisms exist).

## ADR-021 — Manifest approvals are keyed HMAC; bare hashes never authorize

T-004 approvals use HMAC-SHA256 over canonical manifest bytes with keys
provisioned outside the record system; verification fails closed on wrong
key, tampered payload, malformed tag, or an unkeyed content hash
(`tests/approval_deny.rs`). Rejected alternative: treating SHA-256 digests as
signatures (explicitly forbidden by IMPLEMENTATION_PLAN T-004); asymmetric
signatures (deferred until a deployment boundary needs them).

## ADR-022 — Cross-language digest conformance is vector-pinned and fails closed

The Rust canonical-JSON digest profile is pinned by 48 vectors generated from
the Python reference (`tools/gen_digest_vectors.py`); both languages assert
against the same file. Known divergence: exponent-formatted floats render
differently, which produces mismatched digests and therefore *rejects*
bindings rather than accepting them. Rejected alternative: claiming
unconditional cross-language equivalence without vectors (the package README
requires these vectors before runtime use).

## ADR-023 — Traceability is enforced twice, from one source of truth

The 119 obligation mappings are validated by both the Python reference
(`tools/traceability.py`) and a Rust port (`crates/hephaestus/src/traceability.rs`)
with parity tests, including a byte-exact render of the three generated
documents. A mismatch in either implementation fails CI. Rejected
alternative: trusting only the reference tools (independent re-derivation
catches parser/render drift on the runtime side).

## ADR-024 — Documentation and tree freshness is enforced in tiers, inside the single gate registry

Documentation, folder, and file freshness is enforced as a four-layer model
(L1 integrity of frozen bytes, L2 derived-document freshness, L3 prose
semantics, L4 tree structure) whose checks are Makefile targets in the single
gate registry (Decision 6a): the pre-commit hook runs `make ci-fast`
(fast spec checks plus a staged conflict/whitespace check), `make ci` adds the
tree-level conflict grep, a manifest-coverage classifier asserting
`git ls-files ⊆ MANIFEST.sha256 ∪ tools/runtime_allowlist.txt` and the
reverse, an offline internal link/anchor checker, a README fence assertion
(unparseable = fail), and a runtime gate seal (`tools/gate_seal.sha256`
covering Makefile, `.github/workflows/`, `.githooks/`, `tools/*.py`), and
GitHub CI re-runs `make ci` on every push. A commit-msg hook requires an
ADR citation on any diff touching gate files. All of these are contract/spec
checks only — nothing here is runtime acceptance (AT-113). Serves R-014,
R-016, R-043, R-052, R-063, R-077, R-081, R-087. Fail-closed by construction:
any checker exception or parse failure fails the gate; suppression is only by
enumerated `path:line` baseline entries that may shrink but never grow via CI;
ignore-globs are forbidden (R-077, R-052).

L3 (prose matching reality) is explicitly *not fully mechanizable*: the
mechanizable slices (links, fences, structure) are gated, and the remainder is
covered by audit-as-artifact — `validation/<date>/AUDIT.md` written by the
same authoring assistant, with a cadence watchdog on the scheduled workflow
and an advisory age report in `make ci`. Per R-086 these audits are
author-led and are not independent reviews; per R-014 and R-016 they keep
evidence and exact sources outside ephemeral conversations.

Erratum (R-088): the raw commands at `README.md:19–20` and `HANDOFF.md:27`
(`python tools/verify_package.py`, `python -m unittest discover -s tests -v`)
are valid but superseded by the `make setup` / `make ci` flow, since
validation dependencies now live in `.venv`. The MANIFEST-covered bytes and
hashes are unchanged; this correction deliberately lives outside the frozen
envelope and must be re-validated by 2027-03-30 (either fold into the next
spec version's reseal or extend by explicit decision).

Gate files can never join `MANIFEST.sha256` (spec-package-only forever, Q9a /
ADR-017 / ADR-019), so the gate seal is the admissible tamper-detection
substitute; the honest residual trust is stated here: an agent holding write
access to gates, config, and history can still subvert every in-repo check,
and the only out-of-repo anchor (a GitHub ruleset requiring CI and alerting
on workflow changes for `main`) must be configured by the repository owner.
Rejected alternatives: adding gate files to MANIFEST (would rewrite reviewed
v1.2 evidence); Renovate/Dependabot (PR flow conflicts with direct commits on
`main`); running every check on every commit (mid-task transient link
failures create retry pressure; tiering keeps commits fast while `make ci`
backs stops pushes); glob-based ignore lists (silent suppression creep);
executing README fences as tests (illustrative commands, false-fire risk);
adopting the pre-commit framework (a second gate registry, contradicting the
single-registry doctrine).

Appendix (non-normative tier table): conflict-staged → ci-fast, fail;
gen-check / verify / test-package → ci-fast, fail; conflict-tree,
manifest-classifier, md-links, readme-fences, gate-seal, fmt, clippy,
test-rust, test-py, hooks-check → `make ci`, fail; commit-msg ADR citation →
hook, fail; unreferenced-file report → `make ci`, advisory; full `make ci` →
GitHub CI (SHA-pinned actions), fail; online link audit + audit-age watchdog →
scheduled workflow, fail-in-workflow; `cargo doc -D warnings` → separate CI
job, later; GitHub ruleset → repository owner, external.

## ADR-025 — The pre-commit tier runs the manifest classifier; rustdoc gets one named command

Two checks that only GitHub Actions ran became reachable locally: the
manifest-coverage classifier (ADR-024 layer 4) joined `ci-fast`, so the
pre-commit hook — which runs exactly `make ci-fast` — now blocks a commit
whose tracked files are neither manifest-covered nor allowlisted, with the
classifier's own hint, instead of letting a push go red (run 3 pushed twice
red through precisely this hole). `make doc-check` became the single
definition of `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`,
and the remote docs job calls it, so the command cannot drift between
registry and workflow. Serves the same obligations ADR-024 cites (R-014,
R-016, R-043) by making its layer-4 classifier reachable before a push;
introduces no new obligation. Measured `ci-fast` overhead: baseline pair
0.928 s / 0.950 s (spread 0.022 s), after 0.954 s — the delta sits inside
the baseline spread; timings in the run-4 state LOG. `manifest-check` stays
explicitly listed on `ci` as well — the target list is the human-readable
gate inventory, and the deterministic double-run is free at that scale.
This supersedes two ADR-024 appendix rows: "manifest-classifier → `make
ci`" becomes "`ci-fast` (hence every commit) and `make ci`"; "cargo doc →
separate CI job, later" becomes "local `make doc-check` target, called by
the separate CI job". Rejected alternatives: putting cargo doc into `ci-fast`
or `ci` (violates ADR-024 step 10's latency decision — a multi-second rustdoc
build does not belong on the commit hot path; the remote job stays the
backstop); moving `md-links` into `ci-fast` (ADR-024 already rejected
transient link failures on the commit path); editing ADR-024's appendix in
place (registers are append-only records — supersession is stated here).

## ADR-026 — The CI runner relaxes Ubuntu's unprivileged-userns AppArmor restriction before the bwrap gates

The T-010 sandbox tests hard-require bwrap and never skip (they fail when
bwrap cannot run), but they only reached GitHub's `ubuntu-latest` runners
with the run-14 push: `crates/hephaestus/tests/sandbox_deny.rs` first exists
in the T-008..T-013 commit, so every previously green CI run predates it.
Stock Ubuntu 24.04 ships `kernel.apparmor_restrict_unprivileged_userns=1`;
under that restriction unprivileged bwrap fails inside its own netns with
`bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted` and every
sandbox test goes red — runs 36924168195 and 36924735157 (four attempts,
identical failure). The `gates` job therefore prints and then sets that
sysctl to 0 before `make ci`: the runner VM is ephemeral, the local gate is
untouched, and the tests themselves stay fail-closed — if bwrap still cannot
run they fail rather than skip, so CI green keeps meaning what it meant.
Rejected: letting the sandbox tests skip under CI (CI would be greener than
local `make ci` — the exact honesty hole ADR-024 exists to close); pinning
an older runner image (image pins rot and mask the real incompatibility);
AppArmor profile surgery instead of one sysctl (more surface, no stronger
guarantee on a throwaway VM).

## ADR-027 — Requirement citation coverage is a sealed, fail-closed gate

Every derivation ran an ad-hoc coverage scan whose scope decided the
answer: crates+python alone manufactured 25 phantom "uncited"
requirements (most audited as implemented elsewhere across runs
17–26), while including requirements data yields a tautology (the
requirements cite themselves). `tools/check_requirement_citations.py`
now scans CODE ONLY (crates, python, tests, tools, validation py,
.githooks) and gates `make ci` via the new `req-coverage` registry
entry: an uncited id fails until triaged — cited honestly, or listed
in `tools/requirement_citations.txt` with a written reason — and a
stale allowlist entry (id since cited) also fails, mirroring
manifest-check's stale detection. The allowlist joins gate_seal's
EXTRA_DATA_FILES so gap-entries cannot be silently added or dropped.
Initial content: R-054 (no cache exists; views rebuildable by
construction — gates any future cache) and R-079 (build-order
process obligation with no code form). Five honest cites (R-081,
R-085, R-087, R-088, R-089) cleared the rest of the code-scope
remainder. Rejected: scanning requirements/validation data
(tautology); leaving the scan ad-hoc (phantom-gap re-audits cost a
full grill every run); an advisory-only report (a gap in the
derivation signal is exactly what a gate should stop).

## ADR-028 — Ticket status is a gate: format in ci, stale-open sweep in the periodic audit

The 2026-10-02 audit found 28 tickets whose work had landed long ago
while their Status still read `ready-for-agent` — the dangerous stale
direction (green work, red label). Labels that live in files nobody is
forced to flip cannot be trusted to discipline alone, so status became
machinery. `tools/check_ticket_status.py --format` joins `make ci` and
enforces structure in seconds: every issue file carries a valid Status
enum, and every non-done ticket must declare a `Verify:` command — the
command that proves its work green. `tools/check_ticket_status.py
--sweep` runs in the scheduled `periodic` workflow (ADR-024 tier T3):
an open ticket whose Verify passes is STALE-OPEN and fails the sweep
until the label flips in the commit that landed the work; a done
ticket whose declared Verify fails is BROKEN; done tickets without
Verify rely on push CI for regression coverage. Triage batches verify
through `report_at_citations.py --batch`, which passes only when every
listed AT is cited in code scope or listed under that ticket's own
Candidate goals section — a disposition, never a silent skip. The
workflow rule lands in AGENTS.md and skill rule 13: flip Status in the
SAME commit that lands the work. Rejected: auto-flipping from green
gates (a green sub-test does not prove the whole ticket's
acceptance); retroactive Verify on the 111 done tickets (unverifiable
claims — push CI already covers their regressions); a manual checklist
(exactly what went stale 28 times). The same format gate then grew
the decomposition rules the skill only advises: open tickets must also
declare a `Covers:` list of spec acceptance-criterion numbers; caps of
16 scope IDs and 8 unchecked boxes enforce "small tasks only" for OPEN
tickets (landed tickets are grandfathered — one closed ticket sits at
28 IDs; history is not rewritten); and any feature with an open ticket
must cover EVERY spec AC through the union of its tickets' Covers
lists, so "as many tickets as needed" is a checkable invariant rather
than a hope. Covers numbers must exist in the spec — phantom coverage
fails. Live demos: an oversized open ticket (17 IDs) and an open
ticket without Covers both fail; removal restores green.
The format gate grew one more requirement after the user's nesting
rule (2026-10-02): every open ticket must declare `**Micro-tasks:**`
and every micro task must contain nano steps as indented bullets —
small tasks are recommended and built as nested small/micro/nano
trees from now on. Missing-section and no-nano cases were demoed
live (fail), nested content passes.
Four-level decomposition (user correction, 2026-10-02): the earlier
two-level Micro-tasks format is superseded by skill rule 18 —
TASK -> small -> micro -> nano, with per-level caps (8 small / 6
micro / 4 nano), a checkbox and Verify line per level, and the
`atomic` marker for childless units. The format gate drops the old
"8 unchecked boxes" cap and parses the four-level structure of every
open ticket (live fail demos: missing `**Small tasks:**`, micro
without nano bullets). Audit small tasks use count-based Verify
lines through the new `report_at_citations.py --min-disposed N` flag,
so the 16 scope-ID cap stays honest. The commit unit is the small
task (skill rules 5, 19).

## ADR-029 — The GitHub About is repository data: canonical file, offline gate, live audit

The 2026-10-03 user report found the GitHub About stale. The live
description did not start with the repository name, carried no spec
version, and the website and topics fields were empty. Nothing could
notice, because the About exists only on GitHub and no gate reads it.
ADR-027's discipline — the repository's own claims must be checkable from
the repository — applies to the About as well.

Decision: the About becomes repository data. `.github/repo-about.json`
holds the canonical description, homepage and topics, plus the README it
is derived from, the date it was last confirmed against GitHub, and a
maximum age. Three checks enforce it.

1. `tools/check_repo_about.py` (offline, no network) joins `make ci` as
   `about-check`. It fails when the file is missing or malformed, when the
   description exceeds GitHub's 350-character limit, when it does not
   start with the project name in README's H1, when it does not carry
   `v<x.y>` from README's `Version` line, when a topic breaks GitHub's
   token rules, or when `verified` is older than `max_age_days` (365). Two
   facts therefore force an About review: a repository rename and a spec
   version bump. An unconfirmed About expires instead of aging out of
   sight.
2. `tools/check_repo_about.py --live` (online) runs as a separate `about`
   job on every push and pull request in `.github/workflows/ci.yml`, and
   weekly in `.github/workflows/periodic.yml`. It compares the live
   description and homepage byte-for-byte and the topics as a set (GitHub
   returns them sorted) with the canonical file. Three attempts, then
   failure: an unreachable API reads as FAIL, never as pass, so a broken
   check cannot hide a stale About. The weekly
   leg exists because an About edited in the web UI while no push happens
   would otherwise stay invisible for months.
3. `make about-sync` is the repair command. It needs owner auth (`gh`),
   PATCHes the live About from the canonical file, re-verifies against the
   API, and stamps `verified` to today; the stamp is committed with the
   fix.

The CI token is the default `GITHUB_TOKEN` with `contents: read`, which
cannot edit repository settings. Detection is the CI leg's job; the repair
stays local and owner-authenticated. Rejected: writing the About back from
CI (the default token has no `administration` permission, and a stored
admin token would be a standing credential held for a cosmetic field);
editing the About once and trusting it to stay put (exactly the failure
that shipped); putting the live check inside `make ci` (network inside the
local latency budget, and a GitHub outage would block every commit);
README-only wording rules (the About is not printed in the README, so
nothing read it).

## ADR-030 — A disabled scheduler must fail the next push, not stay silent

ADR-029's note recorded the risk: GitHub auto-disables scheduled
workflows in a public repository after 60 days with no repository
activity. The `periodic` workflow is the only thing that audits link
freshness, audit age and stale ticket labels, so its own death would be
invisible — no run, no red job, just the first email. The risk was
written down with no machinery behind it.

Decision: `tools/check_scheduled_workflows.py` reads every workflow file
that declares a `- cron:` schedule, fetches the live workflow list, and
fails unless each one is `active`. It runs as the `scheduled` job in
`.github/workflows/ci.yml` on every push and pull request, and locally as
`make scheduled-check`. A workflow auto-disabled for inactivity reports
as `disabled_inactivity` with the repair command (`gh workflow enable
<file>`, then push so the 60-day clock restarts). Finding no scheduled
file at all also fails, so deleting the cron cannot pass silently.
Network or API failure retries three times and then fails — the same
fail-closed rule as ADR-029: an unreachable check must never read as a
passing one.

The detector lives in push CI because a disabled scheduled workflow
cannot run its own detecting job. Push CI is the trigger this repository
already exercises. The GitHub inactivity email stays the second signal.
Rejected: checking inside `make ci` (network in the local tier, ADR-029's
reasoning); a self-checking step inside `periodic.yml` (dead code the
moment it matters); a 60-day calendar reminder (exactly the discipline
ADR-028 rejected as a checklist); an automatic re-enable from CI (the
default token has no `administration` permission, and a scheduled job
cannot wake a workflow GitHub already stopped).

Live demo: a temporary workflow file declaring a cron that GitHub does
not know about fails with `not present in the live workflow list`; the
file is removed and the check is green again.

## ADR-031 — A Verify that runs nothing is not a green Verify

Closing out the 2026-10-03 session, `ticket-status --sweep` reported
`STALE-OPEN .scratch/dev-roadmap/issues/04-cli-canary-watch.md: Verify is
green`. Its Verify was `cargo test --test cli canary_watch`. The command
exited 0 — with `running 0 tests` and `9 filtered out`, because no test
carries that name. `canary_watch` appears in no source file; the feature
is not built. The sweep had read an empty run as landed work, and the
only action it recommends is flipping Status to done. Following it would
have recorded a completion that never happened.

Decision: the sweep never counts an empty run as green. It captures the
output and, on exit 0, requires a non-empty result — cargo's `running 0
tests` or `test result: ok. 0 passed`, unittest's `Ran 0 tests`, pytest's
`collected 0 items` all mean nothing executed. Such a Verify prints
`VACUOUS` with the command, counts against the sweep (exit 1), and can
never satisfy STALE-OPEN or pass a done ticket's proof. A vacuous Verify
is a gate defect, so it fails loudly in both directions.

The defective ticket got an honest command in the same commit:
`cargo test --test cli -- --list | grep canary_watch && cargo test --test
cli canary_watch` — it must FIND the test before it can RUN it, so it is
red until the work exists and green only after it passes. Its Status stays
`ready-for-agent`, which is the true label.

Rejected: flipping the ticket to done (the sweep's green was false, so
the flip would have been a manufactured completion — the exact failure
AGENTS.md forbids); building the feature to make the sweep green (that is
roadmap work, not this session's scope, and building work to satisfy a
label inverts the gate); dropping the ticket from the sweep (silent);
accepting exit 0 as green without reading the output (the defect that
shipped). `make ci` is unaffected: the fast `--format` leg never executes
a Verify, and only `--sweep` runs commands.

## ADR-032 — MANIFEST.sha256 must verify, not just list

Closing out the 2026-10-03 session, a routine `git status` showed a
parallel edit: `AGENTS.md` gained the agent-skills section and
`CLAUDE.md` became a symlink to it. Content compared byte-identical — no
loss, a clean single-sourcing of one rule set. The repo reseals
MANIFEST's `AGENTS.md` line in every commit that touches `AGENTS.md`
(six of six in history), so that line was stale.

Checking all 81 entries found two more rotted lines:
`tests/test_qualification.py` and `tools/verify_package.py`. No tool
read those hashes. `check_manifest_coverage.py` proved coverage (is the
path listed?) and nothing proved integrity (do the bytes still match?).
The file is named an integrity manifest and was a claim nobody checked.
The same defect class as ADR-029 (an About nobody read) and ADR-031 (a
green nobody earned).

Decision: `tools/check_manifest_coverage.py` now verifies every recorded
digest against the bytes on disk and fails closed on a mismatch, a
missing path, or a symlink where a file is expected. The three stale
lines were resealed in this change, so all 81 entries verify. A mismatch
prints the file, both digests, and the repair: reseal that one line,
deliberately.

Rejected: an auto-reseal command that rewrites every hash (it would
bless any change to a spec-package file, which is the tampering the
manifest exists to detect); hashing only the files CI touches (rot hides
in the files nobody touches); dropping the hashes and keeping coverage
alone (the name would then be a lie). Rebalancing which files belong in
the envelope at all — gate files cannot join it (ADR-024) while
`tools/verify_package.py` has sat in it since the baseline — is a
separate decision and stays untouched here.

Live demo: appending one byte to `AGENTS.md` fails with
`MANIFEST hash mismatch: AGENTS.md — recorded 0f81d9f9b710..., actual
5750741ce6d4...`; removing the byte returns green.

## ADR-033 — Workers run dispatch; settlement stays with the scheduler

AE-01/S2 removes the wave barrier so independent tasks overlap and a
child starts as soon as its parent completes (spec AC2). The non-obvious
part is where the money and the state live while tasks run in parallel.
Each dispatch round spawns scoped worker threads (`std::thread::scope`)
— one thread for a run_batch group of trivial tasks, one per other
task — and every worker sends its outcome, or its caught panic, over
one `std::mpsc` channel created outside the scope region.

Decision: the scheduler loop stays the single owner. It selects the
next round (priority class, then declaration index), reserves every
priced task before any spawn (R-055/AT-055), registers the in-flight
holds, spawns workers, then receives completions and performs every
state transition, budget commit or release, and report append itself.
Capacity for the next round is computed from the in-flight registry:
`max_in_flight` minus flights, plus in-flight resource-unit and memory
sums, with exclusivity held for the flight's duration. No worker ever
touches the ledger or the state vector.

Rejected: an async runtime pool (new dependency, and it moves selection
and settlement off the one thread whose order the replay tests pin);
worker-side settlement behind a mutex (it breaks the exclusive-borrow
check-plus-debit invariant `&mut self` gives `BudgetLedger` — two
owners can interleave reserve and commit); keeping the wave barrier
(spec AC2 forbids it); a crossbeam channel (std mpsc already carries
the one thing that crosses the boundary: a settled outcome).

Consequences a future reader needs: a worker panic is caught and
resume-unwound on the scheduler thread, so a missing completion fails
loudly instead of hanging the run; a batch message resolves all its
members atomically, so batch sizes stay exact for admission tests; while
an exclusive task is in flight no new round is selected; pre-dispatch
cancellation skips in-flight tasks (the worker holds the cancel flag
and the hold settles once at completion); `Stuck` is raised only when
the selected wave and the in-flight registry are both empty after a
cascade pass. Physical completion order may vary between runs; recorded
dispatch order, report ordering, and replay stay stable because the
scheduler thread alone writes them.
