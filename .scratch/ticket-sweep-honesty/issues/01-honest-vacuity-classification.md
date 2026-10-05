# SV-01: Honest vacuity classification for the ticket sweep

**Status:** done
**Verify:** .venv/bin/python -m unittest discover -s tests -p test_ticket_status.py -v
**Covers:** 1, 2, 3, 4, 5
**Requirements:** R-080, R-087, R-091
**Read first:** spec at `../spec.md`; ADR-034 in `../../../docs/RUNTIME_DECISIONS.md`.

## Scope

The sweep reads one empty-run marker anywhere in a Verify output as
`VACUOUS`, so `make ci` (three zero-test Rust sub-suites next to 162
executed results) falsely marks the completed baseline ticket BR-01 as
proving nothing, and `--sweep` exits 1.

This TASK repairs one classifier rule inside the protected gate
`tools/check_ticket_status.py`: an exit-0 run is vacuous only when it
holds an empty marker AND no executed-test evidence in the whole
output. Every ADR-031 refusal stays: empty Cargo filter, empty unittest
run, empty pytest collection, and runs whose only non-zero signals are
`ignored` or `filtered out`. No landed Verify is narrowed, no ticket
Status is flipped but this one, no test is weakened, and no green is
created — the change removes one false `VACUOUS` label.

False-rejection evidence:
`.scratch/architecture-efficiency/receipts/ae01-s2-sweep-final.log`
and `.scratch/architecture-efficiency/receipts/ae01-s2-make-ci.log`.

Requirement IDs served:
- R-080: positive, negative, inconclusive and invalid outcomes each
  classify explicitly in `tests/test_ticket_status.py`.
- R-087: ADR-034 is recorded before the gate file changes and cited in
  the commit.
- R-091: red-first evidence, then verify, ci, doc-check and sweep
  receipts under `../receipts/`, not agent agreement.

**Small tasks:**

1. [x] **S1** Repair the sweep classifier and land the small task
   **Status:** done
   **Verify:** .venv/bin/python -m unittest discover -s tests -p test_ticket_status.py -v
   atomic

## Comments

Spec: `../spec.md`. Receipts: `../receipts/`.
Recorded, unchanged: the Pi protocol pins `pi --version` to `1.0.0`;
the installed Pi is `1.0.3`.
