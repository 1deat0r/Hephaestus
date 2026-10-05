# Handoff — ticket sweep honesty (ADR-034)

Task: `.scratch/ticket-sweep-honesty/issues/01-honest-vacuity-classification.md`
Status: done. Spec: `.scratch/ticket-sweep-honesty/spec.md`.
Baseline: 639e695bdaeed49ea740af872e7e792dc3675d8e (pushed, CI run 37306052327 green).
Landing commit: the child of that commit on the same branch.

## What changed

- `tools/check_ticket_status.py`: added `vacuous_output()` and
  `EXECUTED_RE`. Vacuity now needs an empty marker and no executed
  evidence in the whole output. Updated the `--sweep` docstring.
- `tests/test_ticket_status.py`: nine behavioral tests. They drive
  `sweep_mode()` on a temporary ticket tree.
- `docs/RUNTIME_DECISIONS.md`: ADR-034, written before the gate edit.
- `tools/runtime_allowlist.txt` and `tools/gate_seal.sha256`: new paths
  registered, gate resealed with `make seal`.
- `.scratch/ticket-sweep-honesty/`: spec, issue, receipts, this file.

## Commands

```sh
.venv/bin/python -m unittest discover -s tests -p test_ticket_status.py -v
make ci
make doc-check
make ticket-status
make ticket-status-sweep
make seal
```

## Receipts

| File | Result |
| --- | --- |
| `red.log` | rc 1 before the fix. Two false-rejection failures, ten classifier errors. |
| `verify.log` | rc 0 after the fix. Nine tests pass. |
| `ci.log` | `make ci` rc 0. |
| `doc-check.log` | `make doc-check` rc 0. |
| `sweep.log` | rc 1. `1 stale-open, 0 broken-done, 0 vacuous, 6 honestly open`. |
| `format.log` | `make ticket-status` rc 0 after the status flip. |
| `sweep-final.log` | rc 0. `0 stale-open, 0 broken-done, 0 vacuous, 6 honestly open`. |

Receipt logs have trailing spaces removed. `git diff --check` rejects
them otherwise. Test bytes and result lines are unchanged.

## Results

Before the repair, `--sweep` printed
`ticket-status: VACUOUS .scratch/development-baseline-repair/issues/01-restore-verification-gates.md`.
That ticket is done and its Verify, `make ci`, is green.
After the repair the same sweep prints `0 vacuous`.

The refusals hold: an empty Cargo filter, an empty unittest run, an
empty pytest collection, and an all-ignored or filtered-out run still
print `VACUOUS` and still exit 1. A failing run still prints `BROKEN`.

## Limitations

- Classification stays at Verify-command granularity. One empty
  sub-step inside a composite command stays undetectable when other
  parts of the same command ran tests. ADR-034 records this.
- An exit-0 output that prints both marker kinds reads as not
  vacuous. ADR-034 records this accepted risk.
- MiMo wrote and reviewed this change in one session. No independent
  review exists.
- `make doc-check` used cached rustdoc output. No Rust file changed.

## Recorded, unchanged

The Pi preparation protocol pins `pi --version` to `1.0.0`. The
installed Pi reports `1.0.3`. An earlier spec note recorded `1.0.2`.
No install change belongs to this task.

## Next

Start AE-01 S3 only after push CI is green for the landing commit.
The six architecture-efficiency tickets stay honestly open.
