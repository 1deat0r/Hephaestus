# Coding-agent handoff

## Start here

Read `MASTER_SPEC.md`, its four normative supplements under `docs/`, `docs/OBLIGATIONS.md`, `requirements.json`, `release-scopes.json`, `IMPLEMENTATION_PLAN.md`, `ACCEPTANCE_TESTS.md`, `REVIEW_AND_RISKS.md`, and `validation/REPORT.md`. Treat the master spec and requirements as the product contract. This 1.2 package is not a finished application.

First inspect the real development workspace. Preserve existing work. Establish whether this is a new repository or an integration into an existing one. Do not assume that Tachyon's present interfaces match this proposal. Do not use the abandoned project name Axiom.

## Implementation prompt

Implement Hephaestus, the Autonomous Invention Harness defined in this package, beginning with M0 and M1. The critical product requirement is autonomous discovery and hypothesis generation from a broad goal: do not turn the system into a tool that requires the user to supply the invention hypothesis.

Use the specified staged architecture: provider-neutral Rust control plane, isolated Python scientific workers, typed contracts, evidence ledger, protected evaluators, and optional adapters. Keep Jev, Tachyon, graph databases, GPUs, and complex search optional. Do not spend time building a large agent swarm before the authoritative local execution and validation loop works.

For each task, state the requirement IDs and prerequisites, inspect the relevant code, implement the smallest coherent change, run the applicable checks, preserve raw receipts, and update traceability. Do not claim a test was run when only its code was written. Do not treat a schema-valid object as scientifically valid.

Models propose; deterministic policy authorizes; validated tools execute; protected evaluators assess; evidence governs promotion. No worker may grant itself permission, change the confirmatory endpoint after seeing results, edit sealed evaluators, silently weaken a baseline, discard a negative result, or declare global novelty from an empty search.

Keep scientific support, engineering targets, novelty search status, and execution validity separate. Preserve supported, contradicted, inconclusive, invalid, and blocked paths. Require real receipts for model calls, tool effects, experiment data, and tests. Unknown external side effects must be reconciled, not blindly repeated.

Stop at material authorization gaps or a blocking contract contradiction. For reversible implementation details, choose the simplest contract-compliant option, record the decision, and continue. Do not ask the user to make every low-level design choice. Do not extend the product into physical experimentation or public disclosure without separate explicit authorization.

At the end of each milestone, report what actually works, what tests actually ran, which requirements are satisfied, unresolved failures, measured performance, and the next prerequisite-ready task. An experimental release must be labeled experimental until the value-qualification study supports stronger claims.

## First concrete work session

Run `python tools/verify_package.py` and `python -m unittest discover -s tests -v` to understand the supplied contract checks. These do not test a runtime. Create the typed domain package, implement schema and semantic validation, build the event/budget/policy foundation, and add actual runtime tests corresponding to the acceptance IDs.

Do not mistake the reference validator for production security or statistical code. Replace or strengthen it with typed runtime implementations and tests as the milestones require.

Implement R-094–R-114 alongside the existing obligations. Preserve the externally authenticated trust boundary and receipt subject/plan bindings; never load trust from a model-provided record. Begin with the narrow Python import-closure pack and a qualified protected oracle. Contract-ready M0 does not satisfy M3 workflow acceptance. Preserve archived v1.0 bytes and produce actual migration receipts only when a runtime migration is implemented. All 119 runtime tests remain specifications until run against real behavior.

Self-improvement is mandatory at M3: implement R-115–R-119 and the promoted R-070–R-072 gates. Do not postpone the autonomous champion/challenger, persisted-learning and rollback loop until M6 or substitute human-written suggestions for deployment under standing scope.
