# Spec — Formal M0+M1 exit assessment

Status: ready-for-agent
Goal source: derived:roadmap (perpetual directive, cycle 10)
Grill record: `.scratch/m0m1-exit-assessment/grill.md` (10 questions, all self-answered)

## Problem Statement

Both completed milestones have Exit criteria written in IMPLEMENTATION_PLAN
(:26 for M0, :44 for M1) and both are now evidence-complete — but nothing in
the repository says so with receipts. No assessment document exists (grep
clean), so M2 entry rests on remembered green runs rather than a
skeptic-recheckable record: which command was run, what it returned, which
tests carry each criterion, and what the criteria explicitly do NOT
establish. AGENTS.md demands evidence preservation and the spec-package ≠
invention distinction kept visible; a formal disposition is the artifact
that satisfies both.

## Solution

A single new durable document, `docs/M0_M1_EXIT_ASSESSMENT.md`:

1. **Header** — scope (M0+M1 only; M2–M6 deliberately not assessed),
   method (fresh runs this session, receipts over prose), date, and an
   explicit "what this document is NOT" (does not change release labels,
   does not establish scientific validity or production security).
2. **M0 section** — walks plan:26's criterion verbatim (schema, fixture,
   traceability, protected-evaluator access tests) → disposition →
   evidence (command + exit code + test names/counts) → limitations.
3. **M1 section** — walks plan:44's five clauses (recorded replay;
   denied stays denied; bounded parallel budgets; non-duplicating
   ambiguous effects; CLI runs and recovers a fixture DAG) the same way,
   with the CLI demo JSON excerpt as the receipts for the CLI clause.
4. **Limitations and non-claims** — in-process durability, userns-
   dependent sandbox, no real-external reconciliation, no performance or
   scientific claims, spec-package ≠ working invention (AGENTS.md).

Evidence gathered by fresh runs in this cycle: `make ci`,
`cargo test --workspace`, `make doc-check`, and a re-run of the
`hephaestus fixture run|recover` demo — every quoted number carries its
command.

## User Stories

1. As a maintainer, I want each Exit sub-criterion mapped to a fresh
   command result, so that M2 entry rests on receipts, not memory.
2. As a reviewer, I want dispositions limited to PASS/PARTIAL/FAIL as
   literally warranted, so that the document cannot flatter the project.
3. As a skeptic, I want every PASS to name a test or command I can re-run,
   so that verification is independent.
4. As an auditor, I want an explicit "Not established by this assessment"
   list, so that nobody reads PASS as scientific or production endorsement.
5. As a release manager, I want the document to state it does not change
   release labels, so that plan:26's label rules stay authoritative.
6. As an operator, I want the CLI clause quoted with the actual demo JSON,
   so that plan:44's fourth clause is checkable at a glance.
7. As a future reader, I want M2–M6 explicitly out of scope, so that no
   disposition exists for work not performed.
8. As a maintainer, I want the file in `docs/` (runtime-era, allowlisted
   at landing), so that it outlives workflow sessions without touching the
   frozen `validation/` envelope.
9. As a user of the gates, I want link hygiene that keeps `make ci`
   green (existing files only, no unverified anchors).
10. As a reviewer, I want R-IDs cited only where tests already name them,
    so that traceability claims remain checkable, not invented.
11. As an auditor, I want limitations restated in the document itself, so
   that it cannot be quoted out of context as a clean bill of health.
12. As a developer, I want zero code changes in this cycle, so that the
   assessment describes the shipped state rather than moving it.
13. As a maintainer, I want one coherent ticket, so that evidence
    gathering and the document cannot drift apart.
14. As a future milestone owner, I want the assessment to end with the
    M2-entry statement (M0+M1 dispositions complete), so that the gate is
    explicit.
15. As a reader, I want date-stamped evidence (each command's result as
    of the run), so that staleness is visible rather than implied.

## Implementation Decisions

- **File:** `docs/M0_M1_EXIT_ASSESSMENT.md` only — no code, no gate, no
  glossary/ADR/requirements changes (grill Q7/Q8).
- **Structure** exactly per grill Q4; **dispositions** per Q5; **evidence**
  = fresh runs this session per Q3 (commands and results recorded
  verbatim in the document); **links** per Q6; **honesty guards** per Q10.
- Criterion text is quoted from IMPLEMENTATION_PLAN (inline, short) so the
  document is self-contained but attributed.
- Landing: file staged + one allowlist entry + reseal (standing
  procedure; the only gate-file touch of the cycle).

## Testing Decisions

- The document is verified by the gates themselves (md-links, coverage)
  plus a self-check pass: every PASS row must cite a command whose output
  was captured this session; the ticket checklist mirrors the two Exit
  lines clause-by-clause so coverage is mechanical.
- Suite: `cargo test --workspace`, `make doc-check`, `make ci` — evidence
  lines to the state LOG (no Rust changes expected — a green suite here is
  a regression check, not a code proof).

## Out of Scope

- M2–M6 dispositions; any code/config change; release-label changes;
  validation/ edits (frozen envelope); remote publication of the
  assessment; re-running historical archived checks (v1.0/v1.1 archives
  stay as they are).

## Further Notes

- Limitation: the assessment is a point-in-time record — its receipts
  date-stamp themselves; later changes re-open only the criteria they
  touch.
- Limitation: evidence strength is tiered (test suites = automated
  re-runnable; CLI demo = re-runnable command; rationale citations =
  documentary) — the document labels each.
