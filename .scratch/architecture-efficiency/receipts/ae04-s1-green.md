# AE-04 S1 receipt — index artifact dependencies

Date: 2026-10-11 UTC.
Requirement IDs: R-015, R-017, R-107, R-108. Spec AC: 10.
No architectural departure: no RUNTIME_DECISIONS entry needed.

## Red (missing behavior)

Command:
cargo test -p hephaestus --test architecture_incremental_invalidation s1_index

Result: no such test target (the runner lists available targets).
Then compile failure (E0432): `DependencyIndex` and `IndexError`
are absent from `hephaestus::trustprop`.

## Green (after implementation)

Command: same as above.
Result: 5 passed, 0 failed —
s1_index_chain_resolves_transitive_dependents,
s1_index_versions_scope_dependents,
s1_index_cycles_terminate_and_dedupe,
s1_index_conflicting_inputs_refuse,
s1_index_rebuilds_from_exported_links.
Runner: `python3 .scratch/architecture-efficiency/workflow.py verify
AE-04 S1` → PASS, exit 0.
Receipt: ae04-s1-workflow-verify.log.

## Implementation

`crates/hephaestus/src/trustprop/mod.rs` — new `DependencyIndex`:
nodes are (id, version) pairs, so derivatives of v1 ignore v2;
`record` stores exact input versions per derivative version,
idempotent on identical re-records, refusing differing inputs as
conflicting provenance; `direct_dependents` and `affected` resolve
reverse edges with sorted output; `affected` excludes the queried
node and terminates on cycles through a visited set; `links` plus
`rebuild` round-trip the index for persistence. One design fix lands
inside the cycle: rebuild groups links per derivative before
recording, or multi-input derivatives falsely conflict.

## Regressions

`trust_propagation` 4 passed; `memory_quarantine` 4 passed;
`evidence_lifecycle` 7 passed. `make ci` and `make doc-check` run
before landing.

## Limitations

The index lives in memory; persistence uses the exported link list.
No invalidation propagation belongs to S1; S2 builds on this index.
Developer model is Muse Spark per user direction to continue; the Pi
protocol names MiMo. The contract code carries no model dependency.
