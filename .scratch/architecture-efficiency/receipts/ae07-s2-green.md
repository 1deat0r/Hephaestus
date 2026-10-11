# AE-07 S2 receipt — verify independent evidence checks

Date: 2026-10-12 UTC.
Requirement IDs: R-018, R-074, R-106, R-107, R-111. Spec AC: 20.
No architectural departure: no RUNTIME_DECISIONS entry needed.

## Red (missing behavior)

Command:
cargo test -p hephaestus --test architecture_independent_qualification s2_evidence

Result: compile failure (E0432): `EvidenceItem`, `Stance`,
`ClaimCase`, `ClaimVerdict`, `ClaimEvidence`, and
`check_claim_evidence` are absent from `hephaestus::knowledge`.
One test-span fix before green: the contesting span end moves
19 to 20 so it matches the captured bytes exactly.

## Green (after implementation)

Command: same as above.
Result: 3 passed, 0 failed —
s2_evidence_supports_and_counts_origins_once,
s2_evidence_contest_defeats_and_mismatch_stays_unresolved,
s2_evidence_replays_deterministically.
Full target: 6 passed (S1 stays green).
Runner: `python3 .scratch/architecture-efficiency/workflow.py verify
AE-07 S2` → PASS, exit 0.
Receipt: ae07-s2-workflow-verify.log.

## Implementation

`crates/hephaestus/src/knowledge/service.rs` — new claim check:
`EvidenceItem` pairs a span with a support or contest stance;
`check_claim_evidence` verifies every span through `verify_span`
over captured bytes, counts shared origins once, lists verified
contests in `disagreements`, and parks failed spans in
`unresolved`. A verified contest defeats support; no verified
span either way means `Inconclusive`. Exports added in
`knowledge/mod.rs`. No oracle logic duplicates.

## Regressions

`architecture_task_context_assembly` 14 passed.
`evaluator_access` 4 passed (S1 receipt; rerun in `make ci`).
`make ci` and `make doc-check` run before landing.

## Limitations

The check trusts caller-supplied stances; stance truth stays a
reviewer duty. Quarantined sources fail verification through
the existing span rule. This unblocks AE-07 S3.
