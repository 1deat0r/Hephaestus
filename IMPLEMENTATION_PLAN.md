# Implementation plan
Version 1.2 · 30 September 2026

## Implementation contract

Build a working end-to-end computational invention loop before adding optional infrastructure. This package specifies the product; it does not contain the product runtime. The Python code under `reference/` validates selected specification semantics and is not a production scheduler, authorization service, or scientific analysis library.

Use Rust for the authoritative local runtime, Python for isolated experiment workers, and provider-neutral JSON contracts at process boundaries. A real Tachyon backend may replace local execution only after inspecting its actual protocol and passing the same tests. Do not assume compatibility from its name.

Every implementation task must return its requirement IDs, changed artifacts, commands actually executed, raw receipts, remaining failures, and any architectural deviation. Stubbed or mocked behavior must remain visibly labeled and cannot satisfy an integration gate requiring the real behavior.

## M0 — Contracts and protected evaluation foundations

T-001 creates the Rust workspace, Python worker package, development commands, formatting and static checks, and a minimal CI configuration. Pin tool versions after inspecting the development machine. No automatically downloaded local model or GPU package is required.

T-002 generates or hand-maintains strongly typed Rust records from the JSON contracts and adds round-trip tests. Introduce a migration registry. Reject unsupported schema versions rather than guessing at their meaning. Add format, unit, reference, and cross-record semantic validation.

T-003 implements a fixture corpus and a hidden synthetic-world evaluator in a separate permission scope. Include true mechanisms, false mechanisms, confounders, impossible constraints, ambiguous evidence, and changed boundary conditions. The invention workers receive observations and tool access, not hidden ground truth.

T-004 defines the local security model, approval format, artifact identity, and sealed-data access rules. Write deny tests before exposing code execution. Introduce deterministic manifest signing or authentication appropriate to the chosen deployment boundary; do not treat a plain content hash as an authorization signature.

T-005 imports the 119 requirement/test mappings and makes missing, orphaned, nonreciprocal, unmapped-clause and generated-document drift fail validation. Use `release-scopes.json` to distinguish contract readiness from runtime completion. Maintain design-decision records and positive/negative coverage rationale.

Dependencies: T-001 precedes implementation tasks; T-002 and T-004 can proceed in parallel. T-003 depends on the data contract but not on a model. T-005 is continuous.

Exit: schema, fixture, traceability, and protected-evaluator access tests pass. The release label is specification/engineering foundation, not autonomous invention.

## M1 — Authoritative durable local execution

T-006 implements an append-only event ledger, transactional projections, and content-addressed artifact storage. Each write records its actor, mission version, policy version, task, and input snapshot. Validate temporary-to-committed artifact transitions and garbage collection of abandoned temporary files.

T-007 implements capability grants and the deterministic policy engine. Grants bind operation, scope, destination, expiration, artifact identity where relevant, and approved cost. Provider credentials stay outside model-visible context.

T-008 implements the budget ledger with integer minor currency units, reservations, spend reconciliation, release, and unresolved external charges. Unknown prices are explicit. Concurrent reservations must not oversubscribe the shared budget.

T-009 implements the typed task DAG and bounded worker scheduler. Validate DAG acyclicity, dependency outputs, read/write sets, retry semantics, timeouts, resource limits, and cancellation. Batch trivial deterministic tasks instead of spawning a process for every field check.

T-010 implements isolated Python workers, declared tools, network policy, safe output collection, and resource metering. Use an OS isolation mechanism appropriate to the risk and document its real limitations. Merely invoking `subprocess` is not a completed sandbox.

T-011 adds durable operation IDs, effect receipts, retry and reconciliation, event replay, and fault injection. Simulate crashes before and after dispatch, before output commit, after external effect, and during cancellation.

Dependencies: T-006 through T-008 depend on M0. T-009 depends on all three. T-010 can develop against a mock executor after T-007. T-011 requires the integrated execution path.

Exit: recorded operations replay, denied operations stay denied, parallel budgets remain bounded, and ambiguous non-idempotent effects do not duplicate. A CLI can run and recover a deterministic fixture DAG.

## M2 — Autonomous discovery and hypothesis generation

T-012 implements mission intake and the Intent-to-Mission compiler. Separate inferred reversible defaults from missing authorization. A mission created from a goal must not require a supplied hypothesis.

T-013 implements corpus ingestion, exact source-span provenance, full-text retrieval, source coverage reports, and typed evidence edges. Initial data can be local authorized files and one real research adapter. No graph server is necessary.

T-014 implements pressure-point operators: bottleneck extraction, conflicting objectives, failure patterns, anomalies, assumptions, and changed capabilities. Each opportunity must carry evidence or explicit uncertainty.

T-015 implements a mechanism-operator registry. Start with abduction, contradiction resolution, structural transfer, composition, subtraction, and failure resurrection. Operators emit structured mechanism records and rejected-applicability reasons.

T-016 implements the Hypothesis Compiler and semantic validators. Require comparator, units, conditions, predictions, competitors, and operational falsifiers for test-ready candidates. Keep engineering targets separate from mechanism claims.

T-017 implements bounded search, lineage, deduplication, a diversity archive, and rejection audits. Start with a transparent beam/best-first policy rather than a complex learned controller.

T-018 implements two-stage prior-art investigation and claim charts with search scope. A zero-hit search cannot yield global-novelty language. Confidential mechanism details cannot leave an approved boundary implicitly.

Dependencies: T-012 and T-013 precede T-014. T-015 can start from fixture opportunities. T-016 depends on the mechanism contract. T-017 orchestrates T-014 through T-016. T-018 integrates before test readiness.

Exit: broad domain-only goals create grounded opportunities and testable hypotheses on hidden synthetic worlds, without target mechanisms in prompts or fixtures visible to workers. Generation cost and failure modes are recorded. This is a hypothesis-generation milestone, not proof of invention quality in the wild.

## M3 — Experiment, realization and mandatory self-improvement loop

T-019 implements experiment-plan compilation with primary endpoints, comparators, controls, sampling unit, analysis method, stopping rule, protected evaluator digest, budget, and frozen hypothesis version.

T-020 implements a prototype worker that may change candidate artifacts but not protected evaluators or baselines. Add interface and end-to-end assembly verification.

T-021 implements the fixed-sample repository-level method family in `docs/CONTEXT_ASSEMBLY_CAMPAIGN.md`, with independently qualified bounded-outcome intervals, fixed-family error allocation, estimands, missingness, and power/precision design. A protected method registry binds assumptions and implementation identity. Sequential methods remain a later separately qualified extension.

T-022 implements the independent evaluator and typed result interpreter. Preserve valid versus invalid execution, supported versus contradicted versus inconclusive science, and met versus not-met engineering targets. Check guardrails independently.

T-023 implements raw-data capture, complete cost receipts, clean-environment reproduction, and exportable dossiers. Include failed attempts and deviations.

T-024 connects results back into the evidence graph, opportunity miner, and portfolio queue. Corrections and retractions must invalidate dependent interpretations. Restarting research still requires available budget.

Dependencies: T-019 precedes confirmation work. T-020 and T-021 can proceed independently against frozen contracts. T-022 integrates both. T-023 and T-024 depend on typed result outputs.

Exit: end-to-end scenarios cover supported, contradicted, inconclusive, invalid, and blocked outcomes. A negative result is exported honestly. Candidate workers cannot inspect the sealed evaluation manifest. The same artifact can be independently reproduced under the declared environment.

T-033 (mandatory core, M3) implements the self-improvement service in `docs/SELF_IMPROVEMENT.md`: outcome-triggered autonomous hypotheses, bounded budget, durable provenance-bearing learning, matched fresh protected evaluations, authorized champion updates, canary/drift monitoring, crash reconciliation and automatic rollback. Demonstrate a subsequent mission using the improved persisted champion, plus rejection of false/inconclusive/permission-expanding changes. Prompts/operator/context/retrieval/scheduling improvements are core; code proposals and isolated tests are supported, with deployment subject to code-specific capabilities and integration gates. Dependencies: T-007–T-011 and T-019–T-024; complete before the M3 exit. Requirements: R-070–R-072, R-100–R-101, R-115–R-119.

## M4 — Product qualification and interaction

T-025 adds the conversational TUI, truthful progress, hypothesis comparison, evidence drill-down, intervention controls, and a resumable authenticated gateway. It should feel like an interactive research workspace, not a scripted agent play.

T-026 implements the evaluation suite: active-retrieval model baseline, fixed generate-review baseline, simple search baseline, and relevant existing domain methods. Match budgets and access. Include graph, review, decomposition, and optional-decision-model ablations.

T-027 runs prospective pilot campaigns and estimates variance for the larger qualification study. Predeclare metrics and analysis before the confirmatory campaign. Twenty pilot missions are a possible debugging batch, not a mandatory evidence threshold.

T-028 prepares the release dossier, security findings, scope label, reproducibility report, cost/latency measurements, and unresolved research questions. No critical engineering failure remains open in the declared scope.

Exit: ship as EXPERIMENTAL unless evidence supports QUALIFIED_FOR_DECLARED_SCOPE. “Faster,” “more novel,” or “better” claims require the relevant measured baseline comparison and uncertainty.

## M5 — Optional acceleration and advanced search

T-029 evaluates a specific Jev or other typed decision provider for bounded advisory work. Measure calibration, abstention, rejection false negatives, cost, and end-to-end quality. Keep deterministic control-plane gates authoritative.

T-030 inspects and integrates the actual Tachyon execution protocol, or another backend, using adapter contract tests. Check cancellation, durable receipts, policy boundaries, budgets, and replay. Do not port or rewrite an existing repository merely to meet a conceptual naming plan.

T-031 evaluates graph-specific retrieval, vector search, GPU workers, Monte Carlo tree search, quality-diversity search, and learned allocation separately. Promote only when matched ablations demonstrate value within quality guardrails.

Exit: each optional feature can be disabled. The core path stays functional. More parallelism is not automatically a better result.

## M6 — New domains and advanced learning

T-032 adds independently qualified domain packs. A numerical or physical-science pack specifies governing assumptions, units, simulator validity, equipment authorization, and appropriate human review. Physical actuation remains separate from ordinary code execution.

Advanced extensions to T-033 may add optional fine-tuning and new learning strategies. The required T-033 core is completed at M3; M6 cannot waive or defer it.

T-034 performs external or genuinely independent reproduction for the first candidate whose novelty and utility claims warrant it. Retain the possibility of an inconclusive or negative outcome.

Exit: new capabilities are qualified separately rather than inheriting proof from the software domain.

## Build execution discipline

A coding agent should work on the smallest task whose prerequisites are met. Before editing, inspect the actual repository, its instructions, toolchain, and current tests. Preserve existing user work. Do not silently replace the stack, create unsupported compatibility promises, or report mocks as real integrations.

Local fast checks run on every coherent change. Broader integration, security, and experiment-validation checks run on the corresponding boundary. CI should reproduce the important environment-independent gates, cache dependencies safely, and avoid rerunning unrelated heavy benchmarks on every trivial edit. A local pass is not an excuse to omit reproducible release validation.

Every milestone ends with a receipt bundle. An implementation task is complete only when its applicable tests run, artifacts exist, failures are recorded, and the parent integration gate accepts the result. A generated file or a passing syntax check alone is not proof of behavior.

## Version 1.1 task amendments

The existing task IDs remain stable. These additions refine their deliverables and dependencies rather than introduce a second implementation sequence.

| Tasks | Additional obligations | Required deliverables |
|---|---|---|
| T-002, T-004, T-005 | R-094–R-101, R-112–R-114 | 18 typed contracts, immutable transitions/migration registry, external trust context, scoped grant and receipt subject bindings, clause/release matrix |
| T-007, T-010, T-011 | R-099, R-108–R-111 | Authenticated grant expiry/revocation checks, effective sandbox attestations, brokered egress, protected monitoring and race tests |
| T-013, T-018 | R-106, R-107, R-112 | Adjudicated deep/wide/counterevidence retrieval fixtures, captured-byte span checks, scoped claim charts and rediscovery labels |
| T-019, T-021, T-022 | R-094–R-101 | Registered payload canonicalization vectors, protected family/method registry, broker-authenticated exposure ledger, complete result/guardrail qualification |
| T-023, T-024 | R-097, R-098, R-109, R-112 | Version-bound reproduction/promotion assessments, stale/quarantine propagation, explicit historical applicability and negative results |
| T-003, T-026–T-028 | R-102–R-105, R-113 | Independent import-closure oracle, tuned caching baseline, frozen campaign manifest, complete denominators/costs, qualified inference and release receipts |
| T-032, T-033 | R-100, R-101, R-114 | New-domain validity/migration gates and fresh or separately qualified holdouts for challenger evaluation |

M0 defines all added contracts. Their runtime gates apply at the milestones in the register. Authentication, sandbox attestations, scientific methods and independent reproduction remain unimplemented until these tasks produce actual receipts. Domain-only discovery remains a requirement; the pack must not seed target mechanisms.

M3 release scope includes R-070–R-072 and R-115–R-119. A functioning invention loop without autonomous bounded self-improvement cannot claim M3 completion.
