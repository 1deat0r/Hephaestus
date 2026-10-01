# T-045 grill - R-049/R-050: bounded task contracts + deterministic reducer

Goal: R-049 (M1, Task compiler): "Compile bounded task contracts
instead of creating unrestricted mini-agents." AT-049 negative: a
nano-task with undeclared tools or no limits -> contract validation
rejects it.
R-050 (M1): "Use isolated outputs and verify parent assembly."
AT-050 negative: parallel workers mutating a shared artifact -> writes
isolated or denied; reducer detects incompatible proposals.

Existing: nothing — no task compiler module. MASTER_SPEC §17 defines
the shape: TaskSpec with tools, read/write contracts, capabilities,
budget bounds, timeout, retry policy, verification rule, isolation
profile, output schema; reducer validates schemas, checks declared
effects, resolves conflicts deterministically.

## Q1 - Seam?
**A:** New `taskplan` module: TaskSpec (bounded), compile validation
(reject undeclared tools / missing limits), WorkerProposal (isolated
output, no shared mutation), deterministic reducer. (agent-default)

## Q2 - What makes a TaskSpec valid (R-049)?
**A:** Compile rejects: undeclared tool refs, empty limits (no
timeout/no budget bound), missing output schema, missing isolation
profile. Fail-closed. (agent-default)

## Q3 - Reducer semantics (R-050)?
**A:** Workers write isolated artifacts (each proposal names its own
output key; two proposals claiming the same key = incompatible unless
byte-identical). Reducer validates schemas, detects conflicts,
assembles parent output in deterministic (sorted-key) order.
(agent-default)

## Q4 - How much scope?
**A:** Contract-level, not an executor: compile + validate + reduce.
Execution/scheduling stays in the scheduler. Keep it pure (no I/O).
(agent-default)

## Q5 - TDD seams?
**A:** Red-first: (1) AT-049 undeclared tool rejected; (2) no-limits
rejected; (3) AT-050 same-key proposals conflict unless identical;
(4) deterministic assembly order twin-run. (agent-default)

## Q6 - Vocabulary?
**A:** GLOSSARY row: Task contract. (agent-default)
