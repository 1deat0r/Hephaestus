# AE-03 S2 receipt — load source bytes on demand

Date: 2026-10-11 UTC.
Requirement IDs: R-014, R-015, R-017, R-107. Spec AC: 8.
No architectural departure: no RUNTIME_DECISIONS entry needed.

## Red (missing behavior)

Command:
cargo test -p hephaestus --test architecture_compact_evidence_storage s2_source

Result: 0 passed, 6 filtered out — the filter matches nothing.
Then compile failure (E0432): `store_source_bytes`,
`source_artifact_identity`, `read_verified_span`, and `OnDemandError`
are absent from `hephaestus::knowledge`.

## Green (after implementation)

Command: same as above.
Result: 5 passed, 0 failed —
s2_source_identical_bytes_share_one_identity,
s2_source_span_reads_through_identity_without_full_copy,
s2_source_corpus_span_resolves_to_its_identity,
s2_source_refusals_name_the_cause,
s2_source_identity_refuses_unknown_and_quarantined.
Runner: `python3 .scratch/architecture-efficiency/workflow.py verify
AE-03 S2` → PASS, exit 0.
Receipt: ae03-s2-workflow-verify.log.

## Implementation

`crates/hephaestus/src/knowledge/service.rs` — new on-demand layer
over the existing content-addressed `ArtifactStore`:
`store_source_bytes` stages and commits bytes, so identical bytes
share one digest identity with one object on disk;
`source_artifact_identity` resolves a corpus source to its recorded
sha256 and refuses unknown and quarantined sources;
`read_verified_span` re-hashes on read and returns only the requested
slice, so sibling sources never load. All store failures fail closed
through `OnDemandError`. Exports added in `knowledge/mod.rs`. No
`Corpus` shape change; corpus migration stays with S3.

## Regressions

`corpus_ingest` 5 passed; `corpus_search` 6 passed; `event_ledger`
19 passed. `make ci` and `make doc-check` run before landing.

## Limitations

Objects read fully from disk per span; the slice alone returns.
Corpus sources still hold resident bytes until S3 adds cache bounds.
Developer model is Muse Spark per user direction to continue; the Pi
protocol names MiMo. The contract code carries no model dependency.
