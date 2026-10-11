# AE-06 S1 receipt — route deterministic operations

Date: 2026-10-11 UTC.
Requirement IDs: R-005, R-051, R-055. Spec AC: 16.
No architectural departure: no RUNTIME_DECISIONS entry needed.

## Red (missing behavior)

Command:
cargo test -p hephaestus --test architecture_qualified_reasoning_routing s1_deterministic

Result: no such test target, then compile failure (E0432):
`OperationKind`, `Route`, `RouteError`, and `route_operation`
are absent from `hephaestus::backend`.

## Green (after implementation)

Command: same as above.
Result: 3 passed, 0 failed —
s1_deterministic_ops_route_local,
s1_deterministic_ops_refuse_model_route,
s1_deterministic_advisory_may_route_to_provider.
Runner: `python3 .scratch/architecture-efficiency/workflow.py verify
AE-06 S1` → PASS, exit 0.
Receipt: ae06-s1-workflow-verify.log.

## Implementation

`crates/hephaestus/src/backend/mod.rs` — new operation router:
`OperationKind` names parsing, arithmetic, authorization, hashing,
statistical decisions, and advisory tasks; `route_operation` sends
deterministic classes local and advisory tasks to the provider;
`request_route` accepts explicit route requests but refuses the
advisory route for deterministic operations with the operation
named. Routing is total and twin-stable.

## Regressions

`backend_adapter` 7 passed; `advisory_provider` 7 passed.
`make ci` and `make doc-check` run before landing.

## Limitations

The router classifies; it does not enforce at dispatch. Dispatch
enforcement (rejecting model execution of deterministic ops)
belongs to the execution path when a live provider exists. Today
only Null and Fixture providers exist, both consent-gated.
This unblocks AE-06 S2.
