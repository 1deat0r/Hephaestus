# Receipt — gates green after the repair

Date: 2026-10-05
Baseline HEAD: 8b5278ffa18f083740394dca237e033814559ba4
Written after the repair's third full local gate run exited 0. The commit
that lands this file runs the same gates again through `.githooks/pre-commit`
(`make ci-fast`).

## `make ci` — exit 0

Every registry target ran: fmt-check, clippy, test-rust, test-py,
conflict-tree, manifest-check, req-coverage, about-check, ticket-status,
md-links, readme-fences, report-unreferenced, gate-seal, ci-fast
(gen-check, verify, test-package, conflict-staged, manifest-check),
hooks-check.

Selected exact output:

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
manifest-check: ok (tracked=662, manifest=81, allowlisted=581, hashes verify)
requirement-citations: ok (119 requirements, 118 cited in code scope, 1 reasoned allowlist entries)
ticket-status: ok (131 issue files; statuses valid; open tickets declare Verify+Covers; four-level nesting holds (small<=8, micro<=6, nano<=4, per-level Status/Verify, atomic markers); scope-ID caps hold; open features cover every spec AC)
md-links: ok (363 md files, 0 baseline entries)
readme-fences: ok
gate-seal: ok (26 gate files sealed)
generated.rs is up to date
hooks-check: ok
```

Test totals from that run: 84 Rust suites, 628 Rust tests passed, 0 failed
(`test result: ok` on every suite); spec package `Ran 94 tests ... OK`;
worker package `Ran 1 test ... OK`; `tools/verify_package.py` returned
`"status": "PASS"` with 119 requirements and 0 semantic validation errors.

## `make doc-check` — exit 0

```
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
```

## Rust diff subset (acceptance criterion 1)

```
$ git diff --cached --name-only -- crates/
crates/hephaestus/src/main.rs
crates/hephaestus/tests/architecture_resource_admission.rs
crates/hephaestus/tests/cli.rs
$ git diff --cached --stat -- crates/
3 files changed, 33 insertions(+), 17 deletions(-)
```

The changes are rustfmt line-wrapping only; no statement, test name, or
assertion changed.

## Post-commit findings and forward correction (`baseline-repair.2`)

Recorded 2026-10-05, after `baseline-repair.1` (14b947f) was pushed.

1. **Status-staging omission (truthful record).** Commit 14b947f carried
   the ticket at `**Status:** ready-for-agent` with unchecked S1 and
   micro boxes, because the `done` flip was made in the worktree and never
   staged. The worktree held the completed state. Pushed history was not
   rewritten; the completed ticket is staged and lands in
   `baseline-repair.2`. Verified with
   `git show HEAD:.scratch/development-baseline-repair/issues/01-restore-verification-gates.md`.
2. **Remote `ci` failed on the clean checkout.** Run 37290585047
   (<https://github.com/1deat0r/Hephaestus/actions/runs/37290585047>), job
   `gates`, step "Run environment-independent gates", stopped at
   `md-links`:

   ```
   md-links: FAIL
   .scratch/architecture-efficiency/issues/01-concurrent-execution.md:51: broken link '../spec.md' (target missing).
   .scratch/architecture-efficiency/issues/01-concurrent-execution.md:9: broken link '../pi-development.md' (target missing).
   .scratch/architecture-efficiency/issues/02-resource-admission.md:51: broken link '../spec.md' (target missing).
   .scratch/architecture-efficiency/issues/02-resource-admission.md:9: broken link '../pi-development.md' (target missing).
   make: *** [Makefile:109: md-links] Error 1
   ```

   Root cause: `check_md_links.py` scans `git ls-files`, so it sees only
   tracked files. Local runs passed because the link targets existed as
   untracked worktree files. The local green receipt above therefore
   describes worktree state, not clean-checkout state.
3. **Narrow planning integration (human authorized).** Seven documents
   were tracked with all nonblank content preserved. The conflict-staged
   gate (`git diff --check`) rejected trailing blank lines at EOF in six of
   them, so only those blank EOF lines were trimmed and every nonblank line
   is unchanged:
   `.scratch/architecture-efficiency/spec.md`, `pi-development.md`, and
   `issues/03` through `issues/07`. Their seven paths were appended to
   `tools/runtime_allowlist.txt` and the seal was regenerated
   (ADR-024, ADR-027). Result:
   `manifest-check: ok (tracked=669, manifest=81, allowlisted=588, hashes verify)`.
   `.pi/`, `.scratch/architecture-efficiency/workflow.py`, and
   `docs/PI_AGENTS_SETUP_RESEARCH_2026-10.md` stay untracked.
4. **Tracked-only clean export verification.** The staged index was
   exported with `git write-tree` + `git archive` into a fresh directory
   with its own git repository, so `git ls-files` matched the staged tree
   exactly:

   ```
   export ci exit=0
   export doc exit=0
   md-links: ok (370 md files, 0 baseline entries)
   ticket-status: ok (131 issue files; ...)
   gate-seal: ok (26 gate files sealed)
   ```

   Command environment: `CARGO_TARGET_DIR=/home/ideator/.cache/heph-export-target`
   (the first attempt used `/tmp`, which hit its 16 GB tmpfs quota),
   `CARGO_BUILD_JOBS=2` (one parallel link was signalled with 9, cause unconfirmed:
   `collect2: fatal error: ld terminated with signal 9 [Killed]`), and the
   documented one-time `make hooks` (`core.hooksPath=.githooks`) inside
   the throwaway export repository. `hooks-check` self-skips when `CI=true`,
   as it does on GitHub.

## Limitations

- The remote `ci` result for the correction commit runs after push; it is
  reported to the operator, not recorded in this file.
- `report-unreferenced` is advisory (exit 0 by design, ADR-024).
- Gates prove spec/reference, link, and formatting state. No runtime
  acceptance, performance, or scientific claim is made here.
- The export reuses a warm cargo target directory and a lower job count;
  it verifies tracked content, not GitHub runner resources.
