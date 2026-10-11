# AE-06 S2 receipt — select qualified providers

Date: 2026-10-12 UTC.
Requirement IDs: R-005, R-055, R-106. Spec AC: 17.
No architectural departure: no RUNTIME_DECISIONS entry needed.

## Red (missing behavior)

Command:
cargo test -p hephaestus --test architecture_qualified_reasoning_routing s2_routing

Result: compile failure (E0432): `ProviderOffer`,
`TaskRequirements`, `SelectError`, and `select_provider`
are absent from `hephaestus::advisory`. Extra E0753 from the
new inner doc comment fixed to outer style before green.

## Green (after implementation)

Command: same as above.
Result: 3 passed, 0 failed —
s2_routing_selects_cheapest_eligible,
s2_routing_refuses_unqualified,
s2_routing_tie_breaks_deterministically.
Full target: 6 passed (S1 stays green).
Runner: `python3 .scratch/architecture-efficiency/workflow.py verify
AE-06 S2` → PASS, exit 0.
Receipt: ae06-s2-workflow-verify.log.

## Implementation

`crates/hephaestus/src/advisory/mod.rs` — new provider selector:
`ProviderOffer` carries identity, one-unit cost, optional measured
quality, qualified case count, and capability list; `TaskRequirements`
states required capabilities, minimum quality, and minimum evidence
size; `select_provider` refuses invalid costs, keeps offers with
full capability match plus quality floor on enough known-outcome
cases, and returns the least costly eligible identity with
name-order tie break. Missing evidence never qualifies.

## Regressions

`backend_adapter` 7 passed; `advisory_provider` 7 passed.
`make ci` and `make doc-check` run before landing.

## Limitations

Cost uses one unit per call; multi-unit conversion is out of scope.
Quality trusts caller-supplied measured values; measurement stays in
`evaluate_provider`. Escalation limits belong to S3. This unblocks
AE-06 S3.
