# AE-07 S1 receipt — protect generation and evaluation separation

Date: 2026-10-12 UTC.
Requirement IDs: R-018, R-074, R-106, R-107, R-111. Spec AC: 19.
No architectural departure: no RUNTIME_DECISIONS entry needed.

## Red (missing behavior)

Command:
cargo test -p hephaestus --test architecture_independent_qualification s1_isolation

Result: compile failure (E0432): `Role`, `ArtifactRef`,
`AccessRequest`, `AccessError`, and `authorize_access`
are absent from `hephaestus::security::sealed`.

## Green (after implementation)

Command: same as above.
Result: 3 passed, 0 failed —
s1_isolation_allows_legitimate_access,
s1_isolation_refuses_generator_and_stale_versions,
s1_isolation_denial_ignores_version_currency.
Runner: `python3 .scratch/architecture-efficiency/workflow.py verify
AE-07 S1` → PASS, exit 0.
Receipt: ae07-s1-workflow-verify.log.

## Implementation

`crates/hephaestus/src/security/sealed.rs` — new access gate:
`Role` separates generator, evaluator, and reviewer;
`ArtifactRef` marks protected evaluation artifacts with a
requested version; `AccessRequest` pairs role with artifact and
current version; `authorize_access` denies generators on
protected artifacts before version checks, then refuses stale
versions for every role. The gate opens bytes only; it grants
no promotion, budget, or method qualification.

## Regressions

`evaluator_access` 4 passed.
`make ci` and `make doc-check` run before landing.

## Limitations

The gate trusts caller-supplied roles and version facts;
authentication stays a caller duty. Hidden evaluator
separation still rests on the dependency-graph tests. This
unblocks AE-07 S2.
