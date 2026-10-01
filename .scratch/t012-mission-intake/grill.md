# T-012 grill — mission intake & Intent-to-Mission compiler (self-interview, 2026-10-01)

Mode: `grill-with-docs`/`grilling` self-interview (hard rule 1 — no user questions).
Sources: MASTER_SPEC §4 (Mission compiler and autonomy contract), §5 architecture
("The Mission service owns intent and policy versions"), IMPLEMENTATION_PLAN T-012,
R-001/R-010/R-011/R-012, T-004 approval format, T-007 profiles/grant predicates,
T-008 BudgetLedger minor-unit money. Repo-local skill texts read as data only.

## Q1 — Scope: what is T-012 and what is explicitly out?
**A:** T-012 delivers a pure `mission` module: a versioned `Mission` record plus
`compile(goal, profile) -> Mission | AuthorizationRequests` and
`revise(previous, change) -> (Mission, ImpactReport)`. OUT: corpus/T-013+,
any UI/TUI/gateway, persistence into the EventLedger (caller records what it
dispatches; the ledger is payload-generic), and any learned controller.
Evidence: §4 fully describes compiler inputs/outputs; §5 assigns dispatch and
recovery to the scheduler, not the Mission service.

## Q2 — Mission schema: which fields?
**A:** Exactly the §4 list, no more: intended beneficiary, objective, domain
boundaries, practical constraints, resource envelope, permitted tools and
destinations, forbidden actions, confidentiality, success metrics, evidence
standard, stop conditions — plus `version`, `supersedes`, per-requirement
`provenance`, `assumptions[]` (reversible defaults, explicit), and
`authorization_requests[]`. No hypothesis field at all (R-001: no seed
hypothesis required — absence of the field beats `None`).

## Q3 — Autonomy profiles: how enforced?
**A:** Enum of the four §4 profiles: plan-only, local-research,
sandbox-experiments, supervised-external. The compiler maps profile →
permitted tools/destinations/forbidden actions. Compiler *never* emits
physical actuation or public disclosure into any profile's permitted set —
such a goal yields an authorization request, not a Mission (fail-closed).
Evidence: §4 "No profile grants physical actuation or public disclosure
implicitly."

## Q4 — Ambiguity taxonomy: default vs ask?
**A:** Two disjoint outputs. Reversible choices (terminology, initial search
vocabulary, initial corpus, tentative subdomain) → recorded in
`assumptions[]` with provenance, compilation proceeds. Unapproved spending,
external disclosure, irreversible operations, materially ambiguous risk →
`authorization_requests[]`, compilation yields "blocked on authorization"
instead of a Mission. This is classification logic over caller-supplied data —
no refusal-category content involved.

## Q5 — Value frame for unbounded goals ("invent something useful")?
**A:** Input must carry the owner's standing priorities + resource profile;
without them compilation fails closed with `MissingValueFrame`. The compiler
never invents values and never silently picks a sensitive application —
sensitive-domain goals become authorization requests. Evidence: §4 "It must
not substitute its own values, silently choose a sensitive application, or
launch an unbounded search of everything."

## Q6 — Objective vs metrics separation?
**A:** The compiled `objective` states intent; `success_metrics` + `guardrails`
(state correctness, workload scope, privacy, resource use, unacceptable
trade-offs) constrain how it may be pursued. Every inferred requirement
carries `provenance` and an override path; overriding is a `revise` producing
a new version (R-012). A "reduce latency" goal must not compile to metrics
that permit quality destruction — guardrail inferred with provenance.

## Q7 — Versioning and impact (R-012)?
**A:** Any change to goal, cost limit, dataset policy, or quality margin
produces `version: n+1` with `supersedes: n`. `ImpactReport` lists changed
sections; mapping changed sections to invalidated dependent plans is the
caller's job (scheduler owns dispatch — §5). Pause/resume and cancellation
semantics belong to the scheduler/operations layer (already built), not the
compiler.

## Q8 — Resource envelope: what money type?
**A:** Reuse T-008's integer minor-unit money + currency code; unknown prices
are explicit (`UnknownPrice`), never zero-filled. The compiler does not mint
budget reservations — it declares the envelope; the BudgetLedger enforces it.

## Q9 — No-seed-hypothesis acceptance (R-001)?
**A:** Happy-path test compiles a broad goal with zero hypothesis content and
asserts the Mission carries a value frame and no hypothesis field. A goal
*with* a supplied hypothesis is accepted but the hypothesis is recorded as
caller-supplied provenance, never validated (validation is T-016's job).

## Q10 — Seams, red-first, docs obligations?
**A:** Seams are the pure functions `compile` and `revise` — red-first unit
tests at those seams (TDD). Deny-style tests: MissingValueFrame,
sensitive-application → authorization request, actuation/disclosure never
permitted, guardrail presence on metric-gaming goals. GLOSSARY +2..3 terms
(row written FIRST). No ADR (no architectural departure — direct §4
implementation) and no requirements.json change (R-001/R-010/R-011/R-012
already exist and map).

## Q11 — Ticket split?
**A:** Three tracer bullets: 01 record + `compile` happy path + profiles
(R-001, R-010); 02 ambiguity taxonomy + value frame + authorization requests
(R-011); 03 `revise`/versioning + impact + budget-envelope binding (R-012).
Edges 1→2, 1→3. Each ticket red-first at its seam.

## Q12 — Ledger coupling?
**A:** None in T-012. Mission is a serde record the caller may persist;
compiler stays pure and dependency-free (apart from shared money/serde
types). Rationale: §5 ownership split; keeps the seam testable without I/O.
