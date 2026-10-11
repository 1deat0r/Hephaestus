# AE-05 S2 receipt — assemble bounded evidence context

Date: 2026-10-11 UTC.
Requirement IDs: R-017, R-018, R-106, R-107, R-108. Spec AC: 14.
No architectural departure: no RUNTIME_DECISIONS entry needed.

## Red (missing behavior)

Command:
cargo test -p hephaestus --test architecture_task_context_assembly s2_assembly

Result: 0 passed, 3 filtered out — the filter matches nothing.
Then compile failure (E0432): `AssemblyError` and
`assemble_bounded_context` are absent from `hephaestus::knowledge`.

## Green (after implementation)

Command: same as above.
Result: 4 passed, 0 failed —
s2_assembly_deduplicates_identical_spans,
s2_assembly_retains_counterevidence_and_exact_spans,
s2_assembly_refuses_budget_that_omits_mandatory,
s2_assembly_omits_optional_over_budget.
Runner: `python3 .scratch/architecture-efficiency/workflow.py verify
AE-05 S2` → PASS, exit 0.
Receipt: ae05-s2-workflow-verify.log.

## Implementation

`crates/hephaestus/src/knowledge/service.rs` — new
`assemble_bounded_context`: dedupes by exact (source, start, end)
so twin requirements resolve once; same text from two sources
carries two keys so counterevidence stays; spans keep exact
coordinates and bytes with no truncation; intake missing carries
through; a budget excluding a mandatory span refuses with its id
while optional spans past the budget drop silently in order.
Exports added in `knowledge/mod.rs`.

## Regressions

Full `architecture_task_context_assembly` target: 7 passed.
`corpus_ingest` 5 passed. `make ci` and `make doc-check` run
before landing.

## Limitations

Token cost sums span byte lengths; no tokenizer budget exists.
Live grant/dependency flags enter through S1 intake and S3 reuse.
This unblocks AE-05 S3.
