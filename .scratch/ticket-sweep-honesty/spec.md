# Spec — Ticket sweep vacuity classification (ADR-034)

Status: ready-for-agent
Goal source: blocking sweep repair authorized 2026-10-06, before AE-01 S3
Baseline HEAD: 639e695bdaeed49ea740af872e7e792dc3675d8e (pushed; CI run 37306052327 all jobs green)

## Problem Statement

The periodic sweep (`python tools/check_ticket_status.py --sweep`)
prints `VACUOUS` for any Verify that exits 0 while its captured output
contains one empty-run marker: `running 0 tests`,
`test result: ok. 0 passed`, `Ran 0 tests`, `collected 0 items`.
The search is a substring scan over the whole output.

`make ci` always runs `cargo test --workspace`, and that command
always prints three zero-test sub-suites:

- `Running unittests src/main.rs` → `running 0 tests`
- `Doc-tests hephaestus` → `running 0 tests`
- `Doc-tests synthetic_evaluator` → `running 0 tests`

The same run reports 162 executed results. So the completed baseline
ticket `.scratch/development-baseline-repair/issues/01-restore-verification-gates.md`
(Status `done`, Verify `make ci`) is reported as
`VACUOUS ... Verify exited 0 with zero tests run — it proves nothing`,
and the sweep exits 1.

Evidence:
`.scratch/architecture-efficiency/receipts/ae01-s2-sweep-final.log`
and `.scratch/architecture-efficiency/receipts/ae01-s2-make-ci.log`
(lines 40, 1132, 1138 hold the three empty markers).

This defect is in the opposite direction of ADR-031. It manufactures a
gate defect where none exists, and it targets a landed ticket whose
Verify is genuinely green. Acting on it would mean either re-opening a
healthy baseline ticket or narrowing its Verify command. Both are
prohibited repairs.

## Solution

1. Classify a completed Verify as vacuous only when its output holds an
   empty-run marker AND no executed-test evidence anywhere in it.
2. Executed-test evidence: `running N tests`, `test result: ok. N passed`,
   `Ran N tests`, `collected N items`, with N >= 1.
3. Keep every ADR-031 refusal: an empty Cargo filter, an empty unittest
   run, an empty pytest collection, and a run whose only non-zero
   signals are `ignored` or `filtered out`.
4. Record ADR-034 in `docs/RUNTIME_DECISIONS.md` before the protected
   gate file changes, then regenerate `tools/gate_seal.sha256`.
5. Add behavioral regression tests in `tests/test_ticket_status.py`
   that drive `sweep_mode()`: the composite false rejection must fail
   on the old classifier, and the empty-run refusals must hold on both.

## User Stories

1. As a maintainer, I want the sweep green on a genuinely green done
   ticket, so that a healthy baseline does not read as broken.
2. As a reviewer, I want a Cargo filter that matches nothing still
   reported `VACUOUS`, so that ADR-031 keeps holding.
3. As an auditor, I want a red test run before the classification
   change, so that the repair is demonstrated and not asserted.
4. As a maintainer, I want ADR-034 recorded before the gate edit and a
   regenerated seal, so that every gate change stays traceable.
5. As a developer, I want the red, verify, ci, doc-check and sweep
   receipts kept with this task, so that the result stays auditable.

## Acceptance Criteria

1. `python tools/check_ticket_status.py --sweep` prints no `VACUOUS`
   line for `.scratch/development-baseline-repair/issues/01-restore-verification-gates.md`,
   keeps reporting the six architecture-efficiency tickets as honestly
   open, and exits 0 with this ticket's own Verify green.
2. `.venv/bin/python -m unittest discover -s tests -p test_ticket_status.py -v`
   runs nine tests through `sweep_mode()` and the classifier: one
   composite false-rejection regression, one stale-open direction check,
   four retained empty-run refusals, one failing-run check, and two
   classifier checks over executed-evidence variants.
3. `make ci` and `make doc-check` both exit 0 after the change, and
   `make gate-seal` exits 0 against the regenerated
   `tools/gate_seal.sha256`.
4. `docs/RUNTIME_DECISIONS.md` holds ADR-034, appended before the edit
   of `tools/check_ticket_status.py`, and the commit message cites it.
5. Every new tracked file of this feature is registered in
   `tools/runtime_allowlist.txt`; `.pi/prompts/` and
   `docs/PI_AGENTS_SETUP_RESEARCH_2026-10.md` stay untracked.

## Out of Scope

- Any AE-01 work. AE-01 S3 stays pending and untouched.
- Any change to a landed Verify command, ticket Status, spec obligation,
  protected evaluator, or test assertion outside this classifier.
- Installing, upgrading, or downgrading Pi. The preparation protocol
  pins `pi --version` to `1.0.0`; the installed Pi is `1.0.3`
  (an earlier spec note recorded `1.0.2`). Recorded, not changed.
- Any claim of independent qualification. This is one MiMo session's
  gate repair with receipts, not an independent review.
