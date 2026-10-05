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

## Limitations

- Local gates only. The remote `ci` workflow for this commit runs after
  push; that result is reported to the operator, not recorded in this file.
- `report-unreferenced` is advisory: 76 tracked files are not named by any
  markdown document (exit 0 by design, ADR-024).
- Gates prove spec/reference and formatting state. No runtime acceptance,
  performance, or scientific claim is made here.
