# Hephaestus

Shared language for the Autonomous Invention Harness project: the specification package and the runtime that will be built from it.

## Language

**Spec package**:
The reviewed, versioned contract at this repo's root (`MASTER_SPEC.md`, `requirements.json`, `schemas/`, `tools/`) — the product contract, frozen per version.
_Avoid_: the app, the product

**Runtime**:
The not-yet-built Rust control plane and Python workers that implement the spec package.
_Avoid_: the app, the product, Hephaestus (ambiguous between the two)

**Obligation**:
A stable requirement ID (`R-nnn`) in `requirements.json` that a mapped runtime test must eventually satisfy.
_Avoid_: requirement, ticket

**Task**:
A work item (`T-nnn`) in `IMPLEMENTATION_PLAN.md` with declared prerequisites and an exit gate.
_Avoid_: ticket, story

**Milestone**:
A release gate (`M0`–`M6`) grouping tasks behind a labeled exit condition.
_Avoid_: phase, sprint

**Event ledger**:
The append-only, sha256-chained history of contract events in the Rust control plane; the authoritative record that every other view is rebuilt from.
_Avoid_: log, event stream, journal

**Projection**:
A derived view (such as the timeline index) built from the event ledger after its events are durable; rebuildable at any time and never authoritative.
_Avoid_: cache, materialized view

**Staged artifact**:
Bytes written to the store's staging area under a temporary name; addressable only after the atomic commit rename places them at their digest.
_Avoid_: temp file (too broad), pending artifact

**Content addressing**:
Naming an object by the sha256 of its bytes, so the name attests the content and reads can re-verify them.
_Avoid_: hashing, checksum (weaker claim)

**Capability grant**:
An approved, revocable authorization binding one operation — capability scope, destination, artifact identity, policy version, validity window, and approved cost — issued by the standing local authority and evaluated before dispatch.
_Avoid_: permission (too broad), token (implies bearer)

**Policy engine**:
The deterministic, fail-closed decision function that answers allow/deny for an operation from enumerated facts and protected trust — never from a model or a credential.
_Avoid_: rules engine (implies configurable policy), gate (a check, not a decision)

**Reason code**:
A stable, per-facet denial string (or enum) emitted by the policy engine, in fixed order, so a denial says exactly which binding broke.
_Avoid_: error message (unstable prose), log line

**Standing local authority**:
The bootstrap issuer allowed to mint capability grants without a per-use prompt; it authorizes grant *use* through the policy engine, not its own recursion.
_Avoid_: root trust (overclaims), admin

**Budget ledger**:
The single-currency, integer-minor-unit reservoir tracking reserved, spent, and unresolved amounts against one authorized limit (released capacity shows up as available); capacity is never oversubscribed.
_Avoid_: wallet, balance (implies mutable truth), float money

**Reservation**:
An all-or-nothing claim on capacity, keyed by an opaque id, taken before dispatch and settled exactly once as spend (or returned untouched); the check and the debit are one step.
_Avoid_: hold (ambiguous), lock (concurrency primitive, not money)

**Unresolved charge**:
An external charge awaiting reconciliation — an explicit amount when known, or null-with-a-mandatory-reason when the price is unknown; never a silent zero.
_Avoid_: pending fee (downplays unknowns), estimated cost (sounds settled)

**Task DAG**:
A typed, validated plan of tasks — dependencies, read/write sets, retry and timeout declarations, costs, priority class — that must pass fail-closed validation (acyclic, outputs cover reads, conflicts ordered) before any scheduler sees it.
_Avoid_: pipeline (implies fixed stages), workflow (too broad)

**Scheduler**:
The bounded, priority-class dispatcher that reserves budget before every dispatch, enforces in-flight/resource/exclusivity limits, and maps executor outcomes to commit, release, or unresolved — it owns no clock and no durability.
_Avoid_: orchestrator (marketing), runner (the executor runs, not the scheduler)

**Sandbox provider**:
The capability contract that runs code inside OS-level isolation (namespaces, minimal binds, scrubbed env, network off, rlimits) and refuses to spawn when attestation fails.
_Avoid_: container (implies different machinery), executor (the worker executes, the provider isolates)

**Isolation spec**:
One run's declared boundary: allowed tools, binds, env allowlist, wall/CPU/memory/nproc/fsize limits, network policy, and output caps — validated before anything spawns.
_Avoid_: config (too broad), profile (the mission selects profiles; the spec is the concrete instance)

**Attestation**:
The pre-dispatch evidence that the sandbox tool exists and the spec is self-consistent; failure denies the dispatch rather than degrading.
_Avoid_: certificate (crypto implication), self-attestation (this attests the host side, not the payload)

**Trivial batch**:
Several simultaneously-ready trivial deterministic tasks issued to the executor as one `run_batch` call instead of one process per field check.
_Avoid_: micro-batch (jargon), chunk (ambiguous)

**Operation**:
One durably-identified unit of execution whose lifecycle (planned, dispatched, receipted, cancel-requested) is recorded in the event ledger *before* each corresponding effect — its state is always derived, never stored.
_Avoid_: job (runner-level), task (the DAG node it wraps)

**Effect receipt**:
The content-addressed record of what an effect actually reported — outcome, cost, wall-time, reason, attempt, artifacts — committed to the artifact store and hash-referenced by its ledger event.
_Avoid_: log line (not durable/addressed), result (loses the audit meaning)

**Recovery plan**:
The pure classification of replayed operations into requeue / unresolved / terminal / cancelled / corrupt, plus the explicit apply step that reconciles budget exactly once — retries only where permitted.
_Avoid_: restart script (implies blind rerun), garbage collection

**Mission**:
The versioned record of authorized intent: beneficiary, objective, domain boundaries, constraints, resource envelope, permitted tools and destinations, forbidden actions, confidentiality, success metrics, evidence standard, and stop conditions — carrying no hypothesis field by construction.
_Avoid_: goal (the uncompiled input), plan (a downstream artifact compiled from a mission)

**Value frame**:
The owner's standing priorities plus resource profile that bound an otherwise open-ended goal; without it the compiler fails closed instead of substituting its own values.
_Avoid_: preferences (implies taste), defaults (reversible compiler assumptions are not the frame)

**Authorization request**:
The compiler's output when a goal needs unapproved spending, external disclosure, an irreversible operation, or materially ambiguous risk: a reason-coded record the owner must grant before any Mission exists — never a silently-guessed Mission.
_Avoid_: permission prompt (implies UI), approval (T-004 approvals authenticate manifests, this requests intent authorization)

**Corpus**:
The in-memory owner of captured source bytes plus derived edges; every view (search hits, coverage, tallies) rebuilds deterministically from those bytes, and callers persist what they accept.
_Avoid_: database (implies a server), index (one derived view among several)

**Source span**:
A byte-offset range into a source's captured bytes with parser identity and transform record, verified against the capture itself — never against a live file or mutable locator.
_Avoid_: quote (loses the coordinates), reference (too broad)

**Coverage report**:
The honest count of what retrieval saw and skipped — documents searched and matched, spans returned, inaccessible and quarantined excluded — carrying no claim about what was not found.
_Avoid_: recall (implies a bounded reference set adjudication), completeness (forbidden by R-009)

**Evidence edge**:
A typed link (Supports, Contradicts, Mentions, SharesOrigin) binding an evidence record to span coordinates with provenance; quarantine invalidates incident edges without deleting them.
_Avoid_: citation (one-way pointer without type or lifecycle), annotation (free text)

**Pressure point**:
A structured, evidence-cited observation of strain in authorized traces — bottleneck, conflicting objectives, failure pattern, anomaly, assumption, or changed capability — mined by a named operator with a rejection contract, never a supplied idea.
_Avoid_: pain point (vague marketing), issue (tracker item), idea (unsourced)

**Opportunity**:
The operator output record: pressure-point type, problem statement, beneficiary, context, span-bound evidence references, suspected bottleneck, explicit causal uncertainty, qualitative value estimate, feasibility envelope, prior-art query plan, and unanswered questions — validity (evidence-backed / challenged / speculative) reported independently of narrative polish.
_Avoid_: hypothesis (a downstream T-016 record), feature request (no grounding semantics), lead (implies sales pipeline)

**Mechanism record**:
The T-015 contract record T-016's hypothesis compiler builds on: entities, variables, relationships, prerequisites, expected effects, operating regime, failure modes, and a minimal realization — placeholder language ("use AI", "add a graph", "make it adaptive") is refused at construction.
_Avoid_: idea (names no mechanism), approach (unstructured), design (downstream realization detail)

**Operator registry**:
The versioned, deterministically-ordered set of genesis transformations (abduction, contradiction resolution, structural transfer, composition, subtraction, failure resurrection), each with an input contract, applicability check, bounded output, and provenance stamping.
_Avoid_: plugin system (implies dynamic loading), pipeline (implies fixed sequence)

**Rejected applicability**:
The structured, retained reason an operator declined a candidate — machine-stable reason code plus detail, never a silent drop; heuristic (non-justified-filter) rejections are separately audited and never eliminate (R-024).
_Avoid_: error (the operator worked correctly), filter (implies elimination)

**Hypothesis**:
The operational record a mechanism compiles into: context, intervention, comparator, proposed mechanism, predictions, estimands, effect bounds, boundary conditions, competing explanations, required observations, analysis requirements, and falsifiers — mechanistic claim and engineering target held separately, readiness decided only by semantic validation.
_Avoid_: theory (implies established support), conjecture (unstructured), bet (implies stakes)

**Operational discriminator**:
The observation that distinguishes a hypothesis from its competing explanations — without one, the hypothesis stays exploratory (preserved, never deleted); it is the gate between EXPLORATORY and test-ready.
_Avoid_: test (the procedure, not the distinguishing observation), metric (a measurement, not a distinguisher)

**Falsifier**:
An explicit, stated outcome that would count against the claim — never a restatement of the hypothesis's own success metric; stripping falsifiers denies test-ready promotion (AT-025).
_Avoid_: risk (a possibility, not a countable outcome), caveat (a hedge, not a decision rule)

**Guardrail class**:
One of the four R-104 guardrail families — scoped error, correctness, precision/power, noninferiority — ALL of which must be declared before qualification; a per-class named gap is reported, never a generic refusal.
_Avoid_: check (the evaluation, not the declared class), test suite (runtime evidence, not the pre-registration gate)

**Import closure**:
The declared module set checked against independently resolved Rust imports of the first domain — participants (importers + targets) must exactly match the declaration; unsupported resolution blocks with the import named, never silently omitted.
_Avoid_: dependency graph (broader, unresolved), module list (the input, not the verified verdict)

**Oracle disagreement**:
Divergence between two independent closure recounts — the declared closure is refused (conservative), never resolved by picking a winner.
_Avoid_: bug (the disagreement is data), tiebreak (implies a winner is chosen)

**Trust propagation**:
The R-108 rule that derived records (summaries, caches, graph edges, cross-session memory) inherit the MINIMUM trust of their inputs — one untrusted source contaminates the derivation, and trust is never laundered by deriving through a trusted co-reference.
_Avoid_: averaging (trust is not a mean), escalation (trust never rises through derivation)

**Multi-objective archive**:
The R-032 frontier over candidates scored on declared, non-fabricated objective axes (cost x value band) where retention is Pareto dominance, not a single universal score — a cheap uncertain candidate and an expensive plausible candidate both remain when neither dominates (AT-032).
_Avoid_: leaderboard (implies one ranking), weighted score (collapses axes), confidence (a fabricated number)

**Task contract**:
The R-049 bounded TaskSpec a worker compiles against - declared tools, write contracts with schemas, capability grants, budget and timeout bounds, retry cap, snapshot-isolated profile - absent any of which compile validation rejects it as an unrestricted agent, not a task (AT-049).
_Avoid_: mini-agent (an unrestricted delegation), job description (no bounds), prompt (uncompiled)

**Typed rejection**:
The R-061/R-062/R-063 stable rejection record — a machine-comparable code (SEMANTIC_CONTRADICTION, PROVIDER_LEAK, UNKNOWN_SCHEMA_VERSION, MISSING_REFERENCE, UNSUPPORTED_CAPABILITY) plus detail - so a verifier failure is always attributable and never a bare error string.
_Avoid_: error (unstructured), exception (implies unexpected), rejection reason (free text)

**Review vote**:
A reviewer's recorded stance on a subject, typed as a judgment — it never counts as empirical validation and is never consulted by the advancement gate (R-034, AT-034).
_Avoid_: approval count (a tally decides nothing), evidence (votes are not evidence), confidence score (a fabricated number)

**Objection**:
A concrete reviewer concern retained on the review record with its disposition (open, or resolved with a note); an open blocking objection stops advancement (R-036, AT-036).
_Avoid_: comment (no lifecycle), resolved flag (loses history), issue (unscoped)

**Review authority**:
One of the three separated scopes — generator, analyzer, promoter, with a candidate workspace inheriting generator-side limits — that requests an operation; only the analyzer scope registers an evaluator-edit proposal, and no scope can mutate an evaluator through the review service (R-035, AT-035).
_Avoid_: reviewer (a person, not a scope), permission (that is the grant system), owner (implies transferable ownership)

**External act**:
One of the three actions that leave the research boundary — adoption, publication, or manufacturing — each of which needs its own authorization before it may happen (R-045, AT-045).
_Avoid_: deployment (too narrow — publication is an external act too), release (that is the scoped release packet), go-live (vague)

**Separate authorization**:
A per-act, subject-digest-bound grant recorded from the standing authority; one act's authorization never covers another act or a different subject version (R-045, AT-045).
_Avoid_: approval (the HMAC manifest tag), permission (the capability-grant system), consent (implies a model or user opinion, not a standing authority)

**Provisional target**:
A section-26 engineering budget recorded before measurement — it is never an achieved result and never appears in the achieved-benchmark display funnel (R-076, AT-076).
_Avoid_: goal (too soft), SLA (implies a commitment), benchmark (reserved for measured runs)

**Achieved benchmark**:
A performance measurement pinned to a receipt, its value, and the recorded reference machine — the only kind of performance claim the achieved display funnel returns (R-076, AT-076).
_Avoid_: result (unpinned), score (not a measurement), provisional target (the other variant)

**Stronger comparator**:
The best existing implementation a proposed change must be benchmarked against — for context caching, the exact-cache beside full reconstruction; running only the straw baseline without a recorded exclusion justification fails benchmark review (R-083, AT-083).
_Avoid_: baseline (ambiguous about strength), control (experiment vocabulary), fair comparison (unverifiable)

**Exclusion justification**:
The recorded reason a stronger comparator was left out of a benchmark review — the only alternative to actually running it, and never an empty string (R-083, AT-083).
_Avoid_: note (too weak), excuse (informal), waiver (implies authority to override)

**Evidence label**:
The dossier's required measured-or-unmeasured marker: `Measured` must carry its version-bound run receipt, `Unmeasured` carries the reason and source — no dossier exists without one (R-082, AT-082).
_Avoid_: tag (informal), quality (vague), provenance (covers origin, not measurement status)

**Synthetic fixture**:
Packaged example data explicitly labeled `synthetic_fixture` at its source records — it exports only under an unmeasured evidence label and never renders as real experimental evidence (R-082, AT-082).
_Avoid_: sample (could be real), mock (test jargon), dummy (no provenance meaning)

**Domain-only discovery**:
Evaluating the generator on opportunities and mechanisms it originated itself — a benchmark run without supplied, independently lineaged hypotheses is refused at admission (R-073, AT-073).
_Avoid_: brainstorming (informal), seed refinement (the thing this guards against), open-ended search (unmeasurable)

**Independently originated**:
A hypothesis whose lineage names both the opportunity and the mechanism it came from — the only admissible benchmark input (R-073, AT-073).
_Avoid_: novel (a judgment), seeded (the opposite), generated (says nothing about lineage)

**Agent agreement**:
Reviewers' unanimous (or not) approval offered at mission completion — recorded as opinion, structurally never consulted, and never a substitute for evidence (R-091, AT-091).
_Avoid_: consensus (implies a decision), sign-off (a process artifact), validation (that requires evidence)

**Completion evidence**:
The version-bound evidence refs a mission must present to complete — the only input the completion gate reads, and the record a validated-candidate claim would need (R-091, AT-091).
_Avoid_: approval (that is agreement), proof (overclaims), results (unscoped)

**Classified record**:
A record in a bundle export that carries all four of its own facts — status, evidence ids, scope, and reproduction state — the only kind a mixed-state bundle export accepts (R-092, AT-092).
_Avoid_: tagged record (partial), enriched record (implies decoration), validated record (status is one of four facts)

**Reproduction state**:
Which reproduction outcome a record carries — Reproduced, EnvironmentMismatch, or Disagrees — always present as a typed field, never inferred from other facts (R-092, AT-092).
_Avoid_: reproducibility (a property, not a state), verified (collapses the three outcomes)

**Held-out workload manifest**:
A confirmatory-partition repository's assignment and episodes — readable only by the protected evaluator, denied to workers by name (R-084, AT-084).
_Avoid_: test set (implies leakage is fine), secret (it is data, not credentials), private data (scope, not classification)

**Clustering unit**:
The unit the analysis is declared and computed over (for pilots, the repository) — declared non-empty in the plan and carried verbatim into every variance estimate (R-084, AT-084).
_Avoid_: granularity (vague), grouping (a verb), sample unit (the sampling unit is a different, finer concept)

**Evidence class**:
Which of the schema's kinds a statement stands as — observation, assumption, model judgment, derivation, source claim, or human judgment; an observation additionally requires a grounded source span, and repetition never changes the class (R-007, AT-007).
_Avoid_: evidence type (the field name, not the concept), confidence (a number), reliability (a property)

**Grounded span**:
A source span backing a statement — the only thing that can make an observation an observation; absence of it keeps a repeated model statement a model judgment forever (R-007, AT-007).
_Avoid_: citation (a reference, not coordinates), source (the captured bytes themselves), proof (overclaims)

**Retrieved directive**:
Instruction-like content recovered from a retrieved document — untrusted by origin, its own type with no path into a policy request, and evaluation always inert (R-058, AT-058).
_Avoid_: prompt (model framing), command (implies execution), directive alone (drops the untrusted origin)

**Inert instruction**:
A retrieved instruction evaluated to exactly one recorded denial with no side effects — the required outcome when a paper tells the harness to disclose secrets or bypass policy (R-058, AT-058).
_Avoid_: ignored (it is recorded, not dropped), sandboxed (that is the code half), blocked (no mechanism claimed)
