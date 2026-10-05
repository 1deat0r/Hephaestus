# Architecture efficiency task plan

**Status:** pending

## Purpose

This plan decomposes seven architecture proposals. It does not implement them or claim measured improvements.
Accuracy and qualification gates constrain resource optimization. Lower memory use must preserve correctness, provenance, authorization, and audit evidence.

## Pi development target

Pi Agent 1.0.0 uses xiaomi/mimo-v2.6-flash for all development model work.
Read [Pi development protocol](pi-development.md) when starting, resuming, or verifying these TASKS.
Run `python3 .scratch/architecture-efficiency/workflow.py next` to select the next unblocked small task.
The project settings and task template control model selection and bounded context use.
The model choice governs development. Runtime capability contracts remain independent.

## Execution rules

Each issue file is a TASK. Each small task is one local commit unit after its Verify passes.
Each micro task is one red-to-green cycle. Each nano task changes one file or performs one measurement.
Future integration tests use explicit targets. Missing targets are expected failures until implementation starts.
Each small-task test prefix must match real tests. Zero matching tests never proves completion.
Each implementation commit records requirement IDs, relevant checks, receipts, and limitations.
Record architectural departures in docs/RUNTIME_DECISIONS.md before implementation. Preserve frozen specifications and protected evaluators.
Register new tracked files in tools/runtime_allowlist.txt when preparing implementation commits. Preserve existing seal requirements.
Run make ci and make doc-check before landing each small task. Flip its status and checkbox in the same commit.
This request authorizes planning files only. It does not start runtime implementation or publish changes.

## Order and dependencies

Resource admission and compact evidence storage have independent dependencies. Develop one small task at a time by default.
Concurrent execution depends on resource admission.
Incremental invalidation depends on compact evidence storage. Context assembly depends on incremental invalidation.
Reasoning routing depends on context assembly. Final qualification integrates all six preceding TASKS.
Each TASK establishes its tests and baseline before optimization. Final qualification does not replace earlier correctness checks.

## Acceptance Criteria

1. Concurrent execution: Define bounded submission, completion identities, cancellation, and ambiguous-effect outcomes.
2. Concurrent execution: Overlap independent tasks; unblock children after completion; retain one owner for budget and state.
3. Concurrent execution: Test cancellation races, duplicate completions, crash replay, and unresolved costs without duplicate effects.
4. Resource admission: Declare memory, CPU, token, and cost envelopes; refuse oversized or invalid requests before dispatch.
5. Resource admission: Reserve capacity for active work and queued bytes; release reservations exactly once on terminal outcomes.
6. Resource admission: Measure worker peaks; enforce hard limits; include buffers and resident workers; preserve receipts on violations.
7. Compact evidence storage: Verify exact-byte chains incrementally; retain compact offsets; preserve corruption and torn-tail recovery behavior.
8. Compact evidence storage: Store immutable source bytes once; retrieve verified spans through artifact identities without copying entire corpora.
9. Compact evidence storage: Limit resident cache bytes; evict reloadable data; retain referenced originals; measure cold and warm memory peaks.
10. Incremental invalidation: Build rebuildable reverse dependencies using artifact versions; handle cycles and multiple dependency paths.
11. Incremental invalidation: Propagate correction, quarantine, and retraction; preserve audit originals; block stale qualification while review remains pending.
12. Incremental invalidation: Bind cache keys to content, transformations, provider versions, and policy; recheck current grants and dependency validity.
13. Task context assembly: Accept authorized snapshot, task, evidence requirements, and token limit; return provenance and explicit missing evidence.
14. Task context assembly: Deduplicate evidence; retain counterevidence and shared origins; preserve exact spans; refuse budgets that omit mandatory evidence.
15. Task context assembly: Compare Python dependency contexts against the protected oracle; handle cycles, deletions, renames, and unsupported imports conservatively.
16. Qualified reasoning routing: Keep parsing, arithmetic, authorization, hashing, and statistical decisions deterministic; avoid model calls for these operations.
17. Qualified reasoning routing: Select the least costly eligible provider using qualified task evidence; keep optional providers behind capability contracts.
18. Qualified reasoning routing: Escalate from independent checks; enforce retry and token limits; count failures; avoid treating self-confidence as qualification.
19. Independent qualification: Deny generator access to protected evaluation artifacts; preserve advisory-only authority and current-version qualification gates.
20. Independent qualification: Verify claims against captured spans; count shared origins once; retain disagreements and inconclusive outcomes.
21. Independent qualification: Compare baseline and candidate at matched envelopes; record correctness, false promotion, tokens, cost, latency, throughput, and peak memory.

## Verification and measurement

Preserve reserve-before-dispatch, cancellation, unresolved costs, and replay semantics.
Do not require identical physical completion order across concurrent runs. Require reproducible recorded replay and stable report ordering.
Record hardware, OS, workload snapshots, provider versions, cache state, and total acquisition-through-evaluation costs.
Count failed, blocked, and inconclusive attempts. Keep missing measurements explicit.
Compare against tuned exact caching where the context campaign applies. Use its frozen oracle and independent repository-level observations.
The existing provisional campaign targets remain at least 20% lower latency and at most 10% additional peak memory.
Zero observed context omissions or staleness remains a guardrail. It does not prove perfect reliability.
No architecture change inherits approval or qualification from an earlier artifact version.

## TASK index

- [AE-01: Concurrent execution](issues/01-concurrent-execution.md)
- [AE-02: Resource admission](issues/02-resource-admission.md)
- [AE-03: Compact evidence storage](issues/03-compact-evidence-storage.md)
- [AE-04: Incremental invalidation](issues/04-incremental-invalidation.md)
- [AE-05: Task context assembly](issues/05-task-context-assembly.md)
- [AE-06: Qualified reasoning routing](issues/06-qualified-reasoning-routing.md)
- [AE-07: Independent qualification](issues/07-independent-qualification.md)

## Exclusions

This plan does not select a new external framework, enable live providers, change scientific methods, or claim acceleration.
Implementation resolves runtime choices through capability contracts and explicit decision records when required.
