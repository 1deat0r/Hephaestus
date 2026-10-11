# AE-03 S1 receipt — stream ledger verification

Date: 2026-10-11 UTC.
Requirement IDs: R-014, R-015, R-017, R-107. Spec AC: 7.
No architectural departure: no RUNTIME_DECISIONS entry needed.

## Red (missing behavior)

Command:
cargo test -p hephaestus --test architecture_compact_evidence_storage s1_ledger

Result: compile failure, 8 errors, all E0599 —
`line_offsets` and `resident_line_bytes` absent on `EventLedger`.
The missing target alone did not demonstrate behavior; the six tests
below define the required behavior.

## Green (after implementation)

Command: same as above.
Result: 6 passed, 0 failed —
s1_ledger_empty_open_has_no_offsets,
s1_ledger_corrupt_middle_line_names_the_line,
s1_ledger_broken_link_names_the_line,
s1_ledger_torn_tail_trims_and_reports_offsets,
s1_ledger_missing_final_newline_normalizes_and_appends,
s1_ledger_stream_verifies_chain_and_retains_compact_offsets.
Runner: `python3 .scratch/architecture-efficiency/workflow.py verify
AE-03 S1` → PASS, exit 0.
Receipt: ae03-s1-workflow-verify.log.

## Implementation

`crates/hephaestus/src/ledger/event_ledger.rs` — `open` streams the
file through a one-line lookahead reader; the running previous-hash
carries the chain. Retained state changes from every line's bytes to
compact per-line offsets plus the final line's bytes (which the next
append hashes). Corruption, broken-link, torn-tail, and
missing-newline paths keep their exact line numbers and recovery
values. The private whole-file splitter is removed; its unit test
goes with it. Integration coverage replaces it.

## Regressions

`event_ledger` 10 passed; `operations_deny` 19 passed; `cli` 19 passed.
`make ci` and `make doc-check` run before landing.

## Limitations

Offsets assume contiguous lines, which the append path guarantees.
Resident material equals the final line; large single lines still
reside fully. No storage or latency measurement belongs to S1.
Developer model is Muse Spark per user direction to continue; the Pi
protocol names MiMo. The contract code carries no model dependency.
