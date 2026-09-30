Status: ready-for-agent

## Problem Statement

Hephaestus's CI proves the code compiles, tests, and that generated/spec
artifacts are fresh — but nothing proves the rustdoc documentation itself
stays buildable. A doc comment can rot silently (broken intra-doc links,
malformed markdown that rustdoc rejects once promoted to an error) and no
gate notices. ADR-024's sequence explicitly deferred this as step 10, and
adding it to `make ci` would breach the local latency budget the same ADR
protects.

## Solution

Add a `docs` job to the existing GitHub Actions workflow that runs
`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` as a parallel
sibling of the `gates` job, on the same triggers. Local `make ci` is
untouched; wall-clock impact on the workflow is ~zero (parallel), and
doc-rot becomes a red check on every push.

## User Stories

1. As a maintainer, I want rustdoc warnings treated as errors in CI, so that broken intra-doc links never land on `main`.
2. As a maintainer, I want the doc job to run in parallel with the existing gates job, so that CI wall-time does not grow.
3. As a developer, I want `make ci` to stay exactly as fast as it is today, so that the commit loop is not slowed (ADR-024 development-speed rule).
4. As a developer, I want the CI doc command to be runnable verbatim locally, so that a red doc check is reproducible on my machine.
5. As a reviewer, I want doc failures to appear as a distinct named check, so that I can tell doc-rot from test failures at a glance.
6. As a release engineer, I want `--no-deps` so that third-party crates' rustdoc noise can never fail our build.
7. As a process auditor, I want this job to be the step-10 artifact of ADR-024's recorded sequence, so that the implementation matches the decision record.
8. As a fresh-clone agent, I want the doc job to work on a clean runner with only the pinned toolchain, so that no local cache is secretly required.
9. As a maintainer, I want both workspace crates (`hephaestus` and `synthetic-evaluator`) documented, so that the evaluator crate's doc comments cannot rot out of sight.
10. As a developer, I want no doc-artifact upload, so that CI stays lean and no publishing side effects exist.
11. As a maintainer, I want the gate seal regenerated after the workflow edit, so that `make ci`'s seal check stays green in the working tree.
12. As a developer, I want failures to fail closed (red), never warn-only, so that the gate cannot be silently ignored.

## Implementation Decisions

- One new job (`docs`) inside the existing CI workflow, same
  `on: push / pull_request` triggers; no new workflow file.
- Command and flags exactly as ADR-024 step 10 specified:
  `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`.
- No changes to the Makefile, hooks, or `make ci` composition.
- Reuses the workflow's existing pinned toolchain setup (rust-toolchain is
  SHA-pinned; no new actions introduced).
- After the workflow edit, the runtime gate seal is regenerated so the
  in-tree `gate-seal` check verifies; an ADR-024 cite binds at commit time.
  (This run does not commit — skill rule 5.)
- No new ADR: the decision already exists (ADR-024, appendix tier table).

## Testing Decisions

- A good test asserts external behavior only: the exact CI command exits 0.
- Seam (single, per spec policy): executing
  `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` locally —
  the same command the job runs. Secondary: full `make ci` still green
  (proves the seal/Makefile were not disturbed).
- Prior art: the existing `gates` job and this session's Phase-6-style
  gate runs; fact already established (rc=0, 2.44s warm).
- TDD proper is not applicable: no behavior-bearing code is touched.

## Out of Scope

- Improving doc-comment content or adding missing docs.
- Integrating `cargo doc` into `make ci` or the pre-commit hook.
- Uploading doc artifacts or publishing to docs.rs.
- Doc builds for dependencies (`--no-deps` stays).
- Any commit/push (skill rule 5 for this invocation).

## Further Notes

Verified green before spec: `RUSTDOCFLAGS="-D warnings" cargo doc
--workspace --no-deps` → rc=0. Cold-runner cost expected 1–2 min in
parallel; local warm run 2.44s. Traceability: ADR-024 (sequence step 10),
R-052/R-077 (deterministic, fail-closed gates) as contract/spec checks
(AT-113 labels — this is not runtime acceptance).
