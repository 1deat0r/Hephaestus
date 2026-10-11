# AE-05 S1 receipt — define context assembly interface

Date: 2026-10-11 UTC.
Requirement IDs: R-017, R-018, R-106, R-107, R-108. Spec AC: 13.
No architectural departure: no RUNTIME_DECISIONS entry needed.

## Red (missing behavior)

Command:
cargo test -p hephaestus --test architecture_task_context_assembly s1_interface

Result: no such test target, then compile failure (E0432):
`ContextError`, `ContextRequest`, `EvidenceRequirement`, and
`assemble_intake` are absent from `hephaestus::knowledge`.

## Green (after implementation)

Command: same as above.
Result: 3 passed, 0 failed —
s1_interface_returns_spans_with_provenance,
s1_interface_refuses_unauthorized_and_bad_intake,
s1_interface_reports_missing_evidence_explicitly.
Runner: `python3 .scratch/architecture-efficiency/workflow.py verify
AE-05 S1` → PASS, exit 0.
Receipt: ae05-s1-workflow-verify.log.

## Implementation

`crates/hephaestus/src/knowledge/record.rs` — new intake types:
`EvidenceRequirement` names an exact source span plus a mandatory
flag; `ContextRequest` carries snapshot id, caller-evaluated
authorization flag, task, requirements, and token limit;
`assemble_intake` refuses unauthorized snapshots, empty tasks,
empty requirements, and zero token limits, then resolves spans
against the corpus with provenance (`source:id:[start..end)@sha256`)
and explicit `missing` entries for unknown, quarantined, and
out-of-bounds sources. Exports added in `knowledge/mod.rs`.

## Regressions

`corpus_ingest` 5 passed; `corpus_search` 6 passed.
`make ci` and `make doc-check` run before landing.

## Limitations

Token accounting sums span byte lengths only; S2 adds budget rules.
Authorization stays caller-evaluated, mirroring `review::advance`.
This unblocks AE-05 S2.
