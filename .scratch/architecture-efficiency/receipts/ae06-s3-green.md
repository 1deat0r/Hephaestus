# AE-06 S3 receipt — bound escalation and alternatives

Date: 2026-10-12 UTC.
Requirement IDs: R-005, R-055, R-106. Spec AC: 18.
No architectural departure: no RUNTIME_DECISIONS entry needed.

## Red (missing behavior)

Command:
cargo test -p hephaestus --test architecture_qualified_reasoning_routing s3_escalation

Result: compile failure (E0432): `EscalationPolicy`,
`EscalationState`, `IndependentCheck`, `EscalationError`, and
`request_escalation` are absent from `hephaestus::advisory`.
One green-phase ordering fix: failed checks now spend attempt
and token budget, so retry exhaustion binds as tested.

## Green (after implementation)

Command: same as above.
Result: 3 passed, 0 failed —
s3_escalation_from_passing_check_within_limits,
s3_escalation_refusals_named,
s3_escalation_counts_and_replays_deterministically.
Full target: 9 passed (S1 and S2 stay green).
Runner: `python3 .scratch/architecture-efficiency/workflow.py verify
AE-06 S3` → PASS, exit 0.
Receipt: ae06-s3-workflow-verify.log.

## Implementation

`crates/hephaestus/src/advisory/mod.rs` — new escalation bound:
`EscalationPolicy` states maximum retries and token limit;
`IndependentCheck` carries a separate verifier pass plus tokens
used; `EscalationState` counts attempts, tokens spent, and
failures forward only; `request_escalation` refuses
self-confidence with a failing check before budget spends,
binds retry and token limits with named refusals, records one
attempt plus tokens per gated call, and counts each failed
check. Identical sequences reach identical states.

## Regressions

`backend_adapter` 7 passed; `advisory_provider` 7 passed.
`make ci` and `make doc-check` run before landing.

## Limitations

The bound trusts caller-supplied check verdicts; verifier
independence stays a caller duty. No live provider exists, so
no measured escalation outcome forms part of this change. This
completes AE-06 S1–S3.
