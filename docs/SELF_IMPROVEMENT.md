# Required autonomous self-improvement

Version 1.2 · 30 September 2026 · Normative supplement to master section 24. Requirements R-070–R-072 and R-115–R-119 apply. This is a required runtime capability from M3, not a promise that every iteration improves or a claim of implemented learning.

## Autonomous improvement loop [R-115]

Hephaestus MUST improve its own research process using observations from completed, failed, invalid, blocked and inconclusive investigations. A bounded self-improvement service MUST observe, diagnose, propose, compile, shadow-test, independently qualify, deploy permitted changes, monitor and roll back. It starts from broad missions without a user-supplied improvement hypothesis. It MUST generate its own improvement hypotheses and use the same evidence discipline applied to inventions.

Within an approved standing improvement policy, the service MUST execute routine reversible local improvements without asking the owner to design each change or approve each iteration. The owner authorizes the scope, resource envelope and promotion policy; a deterministic protected service mints exact grants and decides whether evidence permits promotion. Merely generating suggestions, accumulating summaries or asking a human to implement every change does not satisfy autonomous self-improvement.

The mandatory M3 path improves at least prompts, hypothesis/operator selection, context compilation, retrieval settings and bounded scheduling/allocation configuration. It must also support proposing and sandbox-testing changes to worker/runtime code; deployment of code changes requires the applicable separately scoped capability and integration/migration/security checks. Code deployment can remain blocked while the approved configuration path operates. Model fine-tuning, new domains and distributed execution are later optional mechanisms; self-improvement is not postponed until they exist.

Triggers are recorded: mission completion, a predeclared batch/interval, independently measured drift, repeated failure class, or an owner request. Each trigger produces a bounded proposal or a recorded no-justified-change outcome. Each cycle has a reserved budget, concurrency/depth/candidate cap, deadline and stop rule. Recursive improvement proposals remain within the original envelope; the service cannot multiply budgets by spawning improvement missions. If no change qualifies, retain the incumbent and preserve the negative/inconclusive learning result.

## Durable learning and change targets [R-116]

Persist mission outcomes, interventions, operator yields, rejection reasons, verified failures, corrections and champion/challenger identities with source and trust provenance. The service MUST use this history in subsequent candidate generation and routing, subject to confidentiality and stale/quarantine rules. Restarting cannot lose learned decisions or reset holdout exposure. A changed condition may reopen an old failure only through a new version with evidence of that changed condition.

Learning targets include better opportunity discovery, discrimination, source/counterevidence retrieval, hypothesis diversity, analysis-plan completeness, context efficiency, scheduling and implementation correctness. Neither an unverified memory nor a model rating is ground truth. Weight updates or learned routing outputs remain advisory unless qualified for their declared domain; deterministic authorization, budgets and promotion gates remain protected. No local fine-tuning or particular model is required for the system to improve.

## Evidence and deployment contract [R-117]

Every `improvement_candidate` MUST identify the incumbent, challenger, target, supporting observation records, immutable evaluation manifest, fresh evaluation partition/family, total resource budget, intended primary benefit, guardrails, rollout and rollback artifact. The manifest freezes effect/noninferiority bounds, sampling units, complete unsuccessful-run denominators, tooling access, method qualification, stopping and multiplicity rules before accessing outcomes.

Compare champion and challenger at matched data, model/tool access and resource allowance. Frozen replay catches regressions; fresh protected tasks test generalization. Repeated challenger queries use the same protected family/exposure ledger rules as other confirmation. A benchmark/evaluator change is evaluated separately with explicit owner authorization and cannot certify its own candidate. Correlated agents or repeated timings are not independent confirmation.

Promotion MUST require a protected, authenticated assessment bound to the exact improvement payload, verified candidate/evaluation/rollback artifacts, a supported predeclared benefit, passing scientific/correctness/security guardrails and current scoped grant. Improving speed alone while weakening evidence, diversity, correctness or false-promotion control is not qualification. Inconclusive or failed evaluations do not deploy. A model cannot attest its own qualification, change the incumbent metric after seeing results, or hide failed candidates.

The assessment includes `benefit_status`, `guardrail_status` and evaluator identity. The reference code checks externally authenticated metadata; it does not compute statistical significance or authenticate an issuer. Live contexts reject synthetic qualification. The runtime must independently verify the actual study, family/exposure history, assumptions and integration evidence before establishing trusted improvement context.

## Canary, recovery and rollback [R-118]

Deploy a qualified change first to a bounded canary/shadow scope specified in the grant. Capture code, prompt, model, policy, schema and data identities; update the champion pointer transactionally without overwriting the incumbent. Monitor predeclared quality, cost, latency, false-promotion, security and drift indicators. Violating a protected guardrail or a failed migration MUST stop the affected rollout and restore the verified incumbent under a standing rollback capability. Do not erase experiments or pretend completed side effects were undone.

Each rollout records its observed scope, qualified assessment, deployment receipt and rollback target. Restart and crash recovery MUST reconstruct the champion and reconcile interrupted deployments. A rolled-back candidate retains the reason and observations; retries need new evidence/version rather than automatic repeated rollout. A changed improvement policy or permission invalidates affected grants. Automatic deployment remains limited to actions covered by standing authorization.

## Required demonstration and boundaries [R-119]

M3 MUST demonstrate an end-to-end controlled loop: a broad mission reveals a nonseeded reproducible failure/opportunity; the service proposes a challenger; protected fresh evaluation qualifies or rejects it; a qualified permitted change becomes the persisted champion; a later mission uses it; restart preserves the state; an injected regression triggers rollback. False, confounded, overbudget, permission-expanding and inconclusive challengers must remain undeployed. A generated patch or an edited prompt without these behaviors is insufficient.

Actual improvement claims report the evaluated tasks, baseline, budgets, effect/uncertainty, failed proposals, scope and remaining limits. An experimental runtime may ship with functioning self-improvement gates before general benefit is established; it cannot claim monotonically increasing intelligence. The system MUST NOT rewrite its safety policy, spending limits, disclosure scope, protected data, evaluator, analysis qualification or promotion authority through this loop. Changes to those boundaries remain owner-controlled and separately authorized.
