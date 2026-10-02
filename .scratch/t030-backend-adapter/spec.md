# T-030 spec - execution backend adapter contract

Status: ready-for-agent

## Problem Statement

Tachyon is optional until its actual interface passes contract tests — and
no real protocol exists to inspect. The honest deliverable is the adapter
CONTRACT: a typed ExecutionBackend boundary with contract tests over a
real LocalProcess adapter (cancellation, durable receipts, policy
boundaries, budgets, replay scoped to recorded responses — R-051). No
compatibility claims about Tachyon.

## Requirements trace

- R-066 / AT-066: inspect + contract-test BEFORE claiming compatibility
  (Tachyon stays Blocked — documented, not claimed).
- R-051 / AT-051: replay determinism scoped to recorded responses +
  declared environments.
- R-055/R-057 continuity: budget-aware dispatch, durable receipts.

## Acceptance Criteria

1. ExecutionBackend trait: dispatch, cancel, receipt, replay.
2. LocalProcess adapter executes deterministic tasks with recorded
   outputs; receipts append-only.
3. Contract checks: cancellation works; receipts survive restart;
   out-of-policy dispatch refused; zero-budget dispatch refused; replay
   digest-verified.
4. Tachyon explicitly Blocked (no protocol inspected, no claim).
5. Twin-run byte-identical contract reports.

## Risks

- Fabricating a Tachyon protocol: FORBIDDEN; contract tests bind the
  local adapter only.
- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
