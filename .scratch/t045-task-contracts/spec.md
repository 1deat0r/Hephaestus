# T-045 spec - R-049/R-050: bounded task contracts + deterministic reducer

Status: ready-for-agent

## Problem Statement

R-049 (runtime M1): compile bounded task contracts instead of
unrestricted mini-agents; AT-049 rejects nano-tasks with undeclared
tools or no limits. R-050 (runtime M1): isolated worker outputs and
verified parent assembly; AT-050 denies shared-artifact mutation and
detects incompatible proposals. No task-compiler surface exists.

## Requirements trace

- R-049 / AT-049, R-050 / AT-050 (OBLIGATIONS; runtime M1, contract
  M0). Source: MASTER_SPEC §17.
- Continuity: R-024 no silent elimination; deterministic replay (§17).

## Design

`taskplan` module (pure, no I/O; execution stays in scheduler):

- `TaskSpec`: id, declared tools, input refs, write contracts (output
  keys + schemas), capability grants, budget bounds, timeout, retry
  policy, isolation profile, output schema.
- `compile_validation(&TaskSpec) -> Result<(), Vec<String>>`: rejects
  undeclared tool refs, missing limits (timeout/budget), missing
  output schema, missing isolation profile — fail-closed (AT-049).
- `WorkerProposal`: worker id, output key, schema, payload bytes.
- `reduce(&[WorkerProposal], parent_schema) -> Result<Reduced,
  Vec<String>>`: schema-checks each proposal, treats same output key
  from two workers as incompatible unless payloads are byte-identical
  (AT-050), assembles parent output in deterministic sorted order.

## Acceptance Criteria

1. AT-049: undeclared tool and no-limits specs are rejected with
   named reasons.
2. Valid spec passes compile validation.
3. AT-050: conflicting same-key proposals rejected; identical
   duplicates accepted.
4. Deterministic assembly: twin-run byte-identical.

## Risks

- Do not build an executor; contract layer only.
