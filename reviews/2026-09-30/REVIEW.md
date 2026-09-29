# Hephaestus specification review — 30 September 2026

## Disposition and scope

Yes, the specification can be materially improved. Its core scientific principles and conservative local architecture should be preserved. The highest-value revision is to turn those principles into enforceable, version-bound qualification contracts and a concrete first-domain evaluation protocol.

This is a single-assistant technical review of the master specification, requirements, acceptance specifications, contracts, reference validator, implementation plan, and risk register, with targeted primary-source research. It is not an independent human expert panel, a production security audit, or validation of invention performance. Research coverage is targeted rather than exhaustive; external claims below are limited to what the inspected primary pages support. The authoritative specification and packaged files have not been changed.

The review used 30 September 2026 in Pacific/Auckland; the tool clock was 29 September UTC. The repository directory is a specification package without a Git repository or `.omp` directory.

## Findings, in priority order

### F1 — Promotion must verify evidence, not accept status assertions

**Priority:** High; resolve in M0 contract design, enforce before M3 promotion.
**Relevant requirements:** R-002, R-025, R-035, R-037, R-040, R-044, R-061, R-080.
**Location:** `reference/semantic_validator.py:95`, `:110`, `:154`; master sections 13–16 and 21.

An in-memory probe changes the synthetic result to `valid/supported/met`, supplies arbitrary well-formed digest strings, and changes the dossier to `validated_candidate/independent_pass`. Every record passes JSON Schema validation and the semantic validator returns no errors. The referenced experiment plan remains draft, blocked, exploratory, unqualified, and without sample size or frozen registration. The result has no findings and no data-access time. No experiment or reproduction was performed; these are explicitly adversarial metadata inputs.

The package honestly says it implements selected reference checks. This is therefore a demonstrated reference-validation gap, not a discovered runtime exploit. However, these reference semantics must not become the production promotion contract.

**Recommended specification addition:** A promotion assessment must identify every required claim and guardrail, resolve its exact hypothesis/plan/artifact versions, and verify an authenticated execution receipt, applicable preregistration, qualified analysis, endpoint-complete findings, raw artifact existence and integrity, and reproduction evidence. Qualification must come from a protected registry or assessment record, not a candidate-supplied enum. Statistical, deterministic, and formal analyses need different evidence paths; do not demand statistical sample sizes for formal proofs. An arbitrary digest and an asserted status are insufficient evidence.

**Acceptance addition:** Reject the supplied adversarial bundle; reject missing primary endpoints, failed controls, missing guardrails, nonexistent artifacts, invalid receipt issuer, and unsupported analysis qualification. Accept a fully qualified record chain through the applicable method-specific path.

### F2 — Define cross-record lineage and freshness invariants

**Priority:** High; M0/M1.
**Relevant requirements:** R-012, R-027, R-047, R-054, R-060, R-061, R-063.
**Location:** `reference/semantic_validator.py:154`; master sections 4, 16, 18, 21.

A second schema-valid probe adds an unrelated mission and attaches the purported validated dossier to it while keeping the original hypothesis and results. The semantic validator returns no errors. It verifies some reference kinds and plan/hypothesis bindings, but not the full dossier-to-mission chain.

The prose requires versioning and stale-evidence propagation. It does not yet supply a complete invariant table defining which changes invalidate which grants, plans, results, caches, and labels.

**Recommended addition:** Specify typed lineage rules across mission → opportunity → mechanism → hypothesis → plan → result → dossier. Every assessment binds the versions it evaluated, the evidence snapshot, policy, analysis, evaluator, and candidate digests. Reuse across missions requires an explicit applicability assessment; historical records remain valid historical records even when no longer applicable to current claims. Stale, revoked, or mismatched dependencies block current promotion. Approvals must be rechecked at dispatch and effect boundaries, with defined cancellation/revocation behavior.

**Acceptance addition:** Cross-mission substitutions; a new hypothesis with old results; changed evaluator/candidate; source retraction; expired/revoked grant; and cache reuse under a changed policy must produce explicit blockers or a new assessed reuse record.

### F3 — Make adaptive inference and holdout reuse executable contracts

**Priority:** High; M0 design, M3 enforcement.
**Relevant requirements:** R-037, R-040–R-042, R-070, R-075, R-078.
**Location:** `schemas/contracts.schema.json:1456`; master sections 14, 24–26; AT-041.

The spec correctly requires a family ledger and multiplicity strategy. The included plan represents the multiplicity strategy as nonempty prose; no principal record represents family membership, selection history, holdout access, or error-budget allocation. Repeated campaign and champion/challenger feedback can reveal a sealed holdout even if workers never read its raw data.

**Recommended addition:** Define a versioned experiment-family record, selection lineage, qualified analysis registry, and sealed-data access ledger. Choose one concrete confirmatory method for the first domain, with its dependence, missingness, clustering, and stopping assumptions. Define policies for choosing hypotheses on exploratory data, confirming on genuinely fresh data, and repeated testing across campaigns. Independent confirmation can address exploratory selection under appropriate conditions; not every exploratory test must receive the same correction. Count feedback queries, restrict returned detail, and retire/rotate confirmation sets when the approved reuse allowance is exhausted.

POPPER is relevant because its statistical guarantees arise from an explicit construction and assumptions; a similar-looking agent loop does not inherit them. [Primary paper](https://proceedings.mlr.press/v267/huang25n.html).

**Acceptance addition:** Test selected winners, repeated looks, correlated endpoints, family resets, reused datasets, and repeated challenger queries. Invalid methods remain unqualified regardless of model-generated rationale.

### F4 — Specify the first campaign sufficiently to fail decisively

**Priority:** High for qualification; define early, execute M3/M4.
**Relevant requirements:** R-019–R-021, R-073–R-078, R-083–R-084.
**Location:** master sections 25–28; AT-074 and AT-078.

The evaluation framework is sensible but still leaves the mission distribution, correctness oracle, metric denominators, baseline implementations, superiority/noninferiority rules, and qualification thresholds open. Passing 93 qualitative acceptance scenarios would not demonstrate invention value.

**Recommended addition:** Publish one versioned computational domain pack and campaign protocol. Define eligibility and task sampling, strong tuned baselines, oracle validity and limitations, independent sampling units, seeds/repetitions, total costs including failed runs and human work, and a prospective confirmation phase. Keep opportunity quality, mechanism evidence, engineering value, and novelty separate. A useful primary measure could be independently qualified useful outcomes per total authorized budget, but its denominator and treatment of negative results need owner-approved definitions; this review does not prescribe an unsupported universal score.

Define a false-promotion qualification bound and an adequately powered evaluation. For illustration only, under independent identically distributed Bernoulli failures, zero failures in 20 cases has a one-sided exact 95% upper failure-rate bound of about 13.9%; zero in 299 cases brings that bound below 1%. These are mathematical illustrations, not recommended sample sizes or guarantees for heterogeneous missions. Dependence and coverage can make the simple calculation inappropriate.

AgentIdeaBench reports that active exploration improves grounding-related dimensions without improving measured originality under its critics. This supports retaining separate originality and usefulness measurements. [Primary preprint](https://arxiv.org/abs/2609.07611).

### F5 — Qualify retrieval and prior-art assessment independently

**Priority:** Medium; M2/M4.
**Relevant requirements:** R-016, R-018, R-028–R-030, R-074–R-075.
**Location:** master sections 6, 10, 25.

Exact source spans and honest search scope are necessary, but they do not measure whether search finds relevant counterevidence or near matches. A beautifully traceable retrieved subset can still be systematically incomplete.

**Recommended addition:** Add deep target-paper discovery and wide relevant-set retrieval fixtures, difficult terminology variants, cross-domain equivalents, citation chains, and known near matches. Evaluate retrieval precision/recall only against declared reference sets whose completeness and adjudication are documented. Track inaccessible sources, contradiction retrieval, and the effect of missing material on claim readiness. Pin parser versions and preserve span coordinates robustly across source transformations.

AutoResearchBench explicitly separates deep and wide literature-discovery tasks and reports substantial difficulty under its evaluated conditions. It is a relevant retrieval-qualification precedent, not evidence of Hephaestus performance. [Primary preprint](https://arxiv.org/abs/2604.25256).

Graph2Idea reports improvements under an automatic idea-evaluation protocol. That supports testing a graph adapter; it does not establish prospective invention value or justify making graphs mandatory. [Primary preprint](https://arxiv.org/abs/2606.09105).

### F6 — Extend security to durable poisoning and repeated execution

**Priority:** High before autonomous execution; M0/M1.
**Relevant requirements:** R-014, R-047, R-058–R-060, R-070–R-072.
**Location:** master sections 16, 20, 24; AT-059 and AT-060.

The spec covers untrusted content and sandbox escape. It should explicitly cover malicious content persisting through summaries, caches, graph edges, operator memories, and future sessions, plus sandbox configuration drift across long campaigns.

**Recommended addition:** Preserve trust origin through derived artifacts; forbid summaries or memory from elevating source content into authority; restrict behavioral-memory and policy writes; quarantine suspect material and propagate invalidation to derived records. Define the supported isolation mechanism and threat model, verify its effective configuration before execution, broker network access and credentials, and monitor actual boundary violations with a fail-closed stop path. Include redirects, DNS/loopback/private-address routes, inherited file descriptors, installed dependencies, and evaluator feedback in the threat model where applicable.

Bad Memory studies persistent prompt injection across agent sessions and motivates protecting memory updates. It is a preprint with system-specific results, not a universal attack-rate estimate. [Primary preprint](https://arxiv.org/abs/2607.14611).

Anthropic's August 2026 account describes layered isolation verification and monitoring for high-risk cyber evaluations. Those deployment conditions differ from Hephaestus; the applicable inference is to test containment continuously rather than trust a one-time configuration check. [Primary account](https://www.anthropic.com/news/improving-alignment-security-efforts).

### F7 — Resolve the rediscovery/promotion taxonomy

**Priority:** Medium; M0 clarification.
**Relevant requirements:** R-002, R-009, R-028, R-046, R-092.
**Location:** `reference/semantic_validator.py:166`; master sections 1, 10, 16.

The product treats useful rediscovery as legitimate and says novelty is orthogonal to scientific/engineering status. The reference promotion rule nevertheless rejects `known` prior art while accepting `near_match` and `no_match_within_search_scope`. The latter two also do not establish actual novelty.

This requires an explicit product decision, not silently weakening the novelty guardrail. Define a validated solution/prototype label available to useful known mechanisms. Reserve any invention-candidate label for the required documented claim-level differences, while still reporting search-limited uncertainty. Alternatively allow known candidates at the common validation level with a prominently separate rediscovery label. Update schemas, requirement wording, and acceptance cases consistently after the decision.

**Acceptance addition:** Identical valid scientific evidence yields the same scientific conclusion under `known`, `near_match`, and scoped-no-match prior-art statuses; only the appropriately defined product/novelty label changes.

### F8 — Tighten release scope and normative traceability

**Priority:** Medium; M0.
**Relevant requirements:** R-005, R-006, R-061, R-077, R-079, R-081.
**Location:** requirements register; master sections 26–27; `tools/verify_package.py`.

R-005 assigns execution of the core workflow to M0, although the full experiment loop arrives at M3. R-006 similarly addresses domain activation at M0 while broader domain support is deferred. These can be resolved by distinguishing a contract/design obligation from a runtime acceptance obligation, and declaring which tests block each release scope.

The verifier checks ID existence and selected metadata. It does not prove that every normative statement is mapped, that requirement/test mappings agree in both directions, or that a test establishes every clause of its requirement. For example, AT-060 tests artifact modification but does not by itself test operation, destination, policy, expiry, or revocation changes.

**Recommended addition:** Provide a normative obligation matrix with exact clause anchors, coverage rationale, enforcement service, positive/negative cases, milestone applicability, and evidence required for completion. Split broad tests into meaningful parameterized cases. Add explicit state-transition tables and policy/schema migration rules. Distinguish M0 contract readiness from later runtime qualification. Maintain a small CLI vertical slice; optional graphs, Jev, Tachyon, advanced allocation, and new domains should remain behind measured integration gates.

## Research additions and architectural restraint

Add AutoDiscovery to the background register because it addresses system-chosen questions directly. Its February 2026 revision describes Bayesian surprise based on LLM belief updates and search over nested hypotheses. That is a useful comparator for exploration policies, but its surprise measure must not be treated as a calibrated scientific posterior or practical user value without qualification. [Primary paper](https://arxiv.org/abs/2507.00310v3).

The existing treatment of Jev as typed probabilistic advisory inference agrees with its provider's description. Keep local calibration and abstention qualification; do not import the provider's speed or reliability claims into acceptance criteria. [Provider announcement](https://typesafe.ai/blog/introducing-system-one-models-and-jev).

The review finds no evidence requiring mandatory graph infrastructure, a larger agent fleet, a specific model, or a distributed runtime. Preserve software-first scope, provider-neutral capabilities, the authoritative ledger, explicit unknown outcomes, and negative-result dossiers.

## Checks actually performed

- The packaged file manifest passes for every listed file.
- Package verifier passes: 31 sections, 93 requirements, 93 runtime acceptance specifications, 12 schemas, 12 schema-valid synthetic records.
- All 28 reference tests pass.
- The two adversarial metadata probes pass schema validation and incorrectly receive zero selected-semantic errors. A third probe confirms rejection of a `known` candidate by the current novelty gate.
- Validation dependencies were installed in a temporary virtual environment because the default Python environment lacked `jsonschema`.

`checks.json` contains command outputs and exit codes. `probes.py` reproduces the findings using deep copies of the synthetic fixture; it never overwrites that fixture or invents a real result. No runtime acceptance test, live adapter, scientific experiment, sandbox attack, or independent reproduction was run.

## Recommended revision order

First resolve promotion, lineage, qualification receipts, and taxonomy; then specify the first analysis family and campaign; then add retrieval qualification, durable-memory security, and precise release traceability. Record affected requirement IDs and approved architectural/product decisions. Revise the authoritative spec, register, schemas, acceptance cases, examples, PDF, and manifest together when implementing that revision. This review does not itself approve or implement those changes.
