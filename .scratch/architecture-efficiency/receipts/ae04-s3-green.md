# AE-04 S3 receipt — authorize exact cache reuse

Date: 2026-10-11 UTC.
Requirement IDs: R-015, R-107, R-108, R-111. Spec AC: 12.
No architectural departure: no RUNTIME_DECISIONS entry needed.

## Red (missing behavior)

Command:
cargo test -p hephaestus --test architecture_incremental_invalidation s3_reuse

Result: 0 passed, 10 filtered out — the filter matches nothing.
Then compile failure (E0432): `ReuseKey`, `ReusePresentation`,
`ReuseRefusal`, and `authorize_reuse` are absent from
`hephaestus::knowledge`.

## Green (after implementation)

Command: same as above.
Result: 3 passed, 0 failed —
s3_reuse_exact_match_authorizes,
s3_reuse_each_mismatch_names_its_cause,
s3_reuse_stale_tracker_derivative_refuses.
Runner: `python3 .scratch/architecture-efficiency/workflow.py verify
AE-04 S3` → PASS, exit 0.
Receipt: ae04-s3-workflow-verify.log.

## Implementation

`crates/hephaestus/src/knowledge/service.rs` — new exact-reuse
gate: `ReuseKey` binds content digest, ordered transforms, provider
version, and policy version with a deterministic key digest;
`authorize_reuse` requires all four bindings plus caller-evaluated
live flags (`grant_current` rechecks the current grant,
`dependencies_valid` rechecks dependency validity, e.g. via the S2
tracker) and refuses the first mismatch with a named cause.
Exports added in `knowledge/mod.rs`. No `Corpus` shape change.

## Regressions

Full `architecture_incremental_invalidation` target: 13 passed.
`corpus_ingest` 5 passed; `corpus_search` 6 passed.
`make ci` and `make doc-check` run before landing.

## Limitations

Live flags stay caller-evaluated, mirroring `review::advance`.
No automatic grant or tracker hook exists yet; AE-05 can wire one.
This completes AE-04 S1–S3 and unblocks AE-05.
Developer model is Muse Spark per user direction; the Pi protocol
names MiMo. Contract code carries no model dependency.
