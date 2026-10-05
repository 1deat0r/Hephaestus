# Receipt — baseline gate failures before repair

Date: 2026-10-05
HEAD: 8b5278ffa18f083740394dca237e033814559ba4 (main, already pushed)
Scope: local gate commands only. No secrets. No source behavior changed.

## 1. cargo fmt --all -- --check — FAIL

Command: `cargo fmt --all -- --check`
8 rustfmt diff hunks across 3 files, exit 1 via Makefile:

- `crates/hephaestus/src/main.rs:149`
- `crates/hephaestus/tests/architecture_resource_admission.rs:93, 206, 279, 291, 313, 348`
- `crates/hephaestus/tests/cli.rs:109`

`make ci` stopped at its first target:
`make: *** [Makefile:28: fmt-check] Error 1`.

## 2. make req-coverage — FAIL

```
requirement-citations: stale allowlist entry R-079 — now cited; remove it from tools/requirement_citations.txt.
make: *** [Makefile:77: req-coverage] Error 1
```

Real citation confirmed: `crates/hephaestus/tests/traceability.rs:141`,
test `m5_milestone_not_required_for_m0_through_m4_completion`, added by
commit 8e9ce81 `t066-b3.2: cite second-half ATs at verified mirrors`.

## 3. make gate-seal — FAIL

```
gate-seal: FAIL
seal mismatch: tools/runtime_allowlist.txt — if the change is intentional, regenerate with `python tools/gate_seal.py --write` and cite ADR-024
```

Intentional allowlist change verified in git history: HEAD 8b5278f added
`crates/hephaestus/tests/architecture_concurrent_execution.rs` and
`.scratch/architecture-efficiency/issues/01-concurrent-execution.md`;
e8fa862 added `crates/hephaestus/tests/architecture_resource_admission.rs`
and `.scratch/architecture-efficiency/issues/02-resource-admission.md`.

## 4. Other baseline gates — PASS

gen-check, verify, test-py, conflict-tree, manifest-check, about-check,
ticket-status, md-links, readme-fences, report-unreferenced, hooks-check.

## 5. Remote ci workflow before repair — FAIL

`gh run list` shows the last five `ci` runs completed with failure:
37282196122 (8b5278f), 37279823192 (8604d25), 37273367457 (0fadabc),
37272043325 (e8fa862), 37268694185 (484a5b1).
