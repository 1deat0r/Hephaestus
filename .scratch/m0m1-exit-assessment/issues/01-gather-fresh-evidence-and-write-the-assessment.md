# 01: Gather fresh evidence and write the M0+M1 exit assessment

**What to build:** `docs/M0_M1_EXIT_ASSESSMENT.md` — every M0 and M1 Exit
clause walked with a fresh command result, a PASS/PARTIAL/FAIL disposition,
test/command receipts a skeptic can re-run, and an explicit non-claims
section — the documented gate before M2.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] Fresh evidence captured this session and quoted with commands +
      exit codes: `make ci`, `cargo test --workspace`, `make doc-check`,
      and a re-run of the `hephaestus fixture run|recover` demo (JSON
      excerpt in the M1 CLI clause)
- [x] M0 section: plan:26's four clauses (schema, fixture, traceability,
      protected-evaluator access) each dispositioned with receipts
- [x] M1 section: plan:44's five clauses each dispositioned with receipts
      (replay/deny-stay-denied/bounded-budgets/no-duplicate-ambiguous/
      CLI run+recover)
- [x] Header states scope (M0+M1 only), method, date, and the
      "not established" framing; closing Limitations-and-non-claims
      section covers in-process durability, userns sandbox, no real
      externals, no science/perf claims, spec-package ≠ invention
- [x] Explicit statement that the document changes no release labels and
      assesses no M2–M6 criterion
- [x] Link hygiene passes md-links (existing files only; test paths as
      inline code); no code/config/glossary/ADR edits
- [x] Gates green with evidence in the state LOG; file staged + allowlisted
      + resealed (standing landing procedure)

## Comments

2026-10-01T02:30:13Z — Done; evidence-first (runs before prose); all checklist items verified; gates green incl. md-links 48 files; one historical transient documented in the assessment itself.

Closed 2026-10-02 — work landed earlier; verified green: M0_M1_EXIT_ASSESSMENT.md carries PASS receipts for every M0/M1 clause.
