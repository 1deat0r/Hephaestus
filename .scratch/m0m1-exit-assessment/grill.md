# Grill — Formal M0+M1 exit assessment

Self-interview (Phase 2, perpetual-directive cycle 10). Frontier emptied in
one round.

Goal: *formal M0+M1 exit assessment document with fresh evidence and
pass/fail dispositions — the gate before M2 entry.*

## Evidence gathered first

- No assessment exists anywhere (grep across docs/ + .scratch/ clean).
- IMPLEMENTATION_PLAN:26 (M0 Exit): "schema, fixture, traceability, and
  protected-evaluator access tests pass. The release label is
  specification/engineering foundation, not autonomous invention."
- IMPLEMENTATION_PLAN:44 (M1 Exit): four pillars + the CLI clause — all
  demonstrable now (run-9 live demo captured in the prior report; CLI
  tests assert all four pillars).
- Fresh green evidence available on demand: `make ci` (fmt, clippy, rust+py
  tests, verify_package, manifest, md-links, seal, hooks), `cargo test
  --workspace` (291), `make doc-check`, CLI demo.
- `validation/REPORT.md` and siblings are MANIFEST-frozen v1.x archives —
  a new assessment must be a NEW runtime-era file, not an edit there.
- AGENTS.md: preserve evidence, report limitations, keep the
  "specification/reference checks ≠ working invention" distinction visible.

## Round 1

❓ **Q1 — Scope**: assess **M0 and M1 only**. M2–M6 exit lines exist but
their work does not — assessing them would fabricate dispositions. Each
assessment covers its plan line verbatim, criterion by criterion.
*source: plan text + honesty rule.*

❓ **Q2 — Location**: `docs/M0_M1_EXIT_ASSESSMENT.md` — durable project
doc beside `RUNTIME_DECISIONS.md`; runtime-era (not in MANIFEST; allowlist
at landing per standing procedure). Not `validation/` (frozen envelope),
not `.scratch/` (must outlive workflow sessions).
*source: manifest evidence + docs conventions.*

❓ **Q3 — Evidence method**: **fresh runs this session** — `make ci`,
`cargo test --workspace`, `make doc-check`, plus a re-run of the CLI demo —
each criterion cites command + exit code + concrete receipts (test names,
counts, JSON excerpts). No claim without a recorded command+result; no
reliance on remembered numbers where a fresh run is cheap.
*source: AGENTS.md evidence rule; receipts over prose.*

❓ **Q4 — Format**: header (scope, method, date, what this document is
NOT: it does not change release labels or establish scientific validity);
per milestone: criterion → disposition (`PASS`/`PARTIAL`/`FAIL`) →
evidence (command/result/tests) → limitations; closing section
"Limitations and non-claims" (AGENTS.md distinctions: in-process
durability, userns-dependent sandbox, no real externals, spec-package ≠
invention).
*source: plan Exit wording + AGENTS.md.*

❓ **Q5 — Disposition rules**: `PASS` requires every sub-criterion to have
fresh green evidence; `PARTIAL` for honest gaps (pre-scan: M0's four gates
are green in every run today; M1's five clauses are green including the
CLI demo — anticipate PASS×2 but the writing step re-verifies each
independently; any surprise becomes PARTIAL with the gap stated).
*source: evidence-first; never inflate.*

❓ **Q6 — Link hygiene**: the md-links gate checks relative links — link
only to files that exist (IMPLEMENTATION_PLAN.md, test paths as inline
code not links, no heading anchors unless verified); keeps `make ci` green.
*source: tools/check_md_links.py behavior.*

❓ **Q7 — Ticket shape**: **one ticket** — gather fresh evidence, write
the assessment, run gates, stage+allowlist. Coherent single artifact; no
meaningful second slice.
*source: vertical-slice sizing (like run-4's config change).*

❓ **Q8 — Docs/obligations**: no ADR (documentation of existing state,
not a decision); no glossary terms (exit assessment = plan vocabulary);
no requirements/traceability edits; R-IDs appear only as evidence
citations where tests already name them. New file allowlisted at landing
(standing).
*source: domain.md + AGENTS.md.*

❓ **Q9 — Gates**: report_unreferenced will advisory-list the new file
(known, non-failing); md-links must pass (Q6); full `make ci` +
`doc-check` green before close; no gate-file edits (no seal churn beyond
the standing allowlist landing step).
*source: house gate mechanics.*

❓ **Q10 — Honesty guards inside the document**: explicit "Not
established by this assessment" list (scientific validity, production
security, multi-tenant isolation, real-external reconciliation,
performance); dispositions scoped to criteria as literally written;
numbers quoted with their source command. Retro-style self-check: every
`PASS` must point at a receipt a skeptic could re-run.
*source: AGENTS.md "package contains specification/reference checks, not a
working invention application".*

## Frontier status

Empty. No refusal-category item (local files + local commands only).
