# 01 receipt — first honesty sweep

Date: 2026-10-12 UTC.
Spec AC: 1, 2, 3. AC 4 unexercised (no rot found).
No source or gate-logic changes; allowlist and seal
bookkeeping only.

## Red

Ticket Verify (composite) failed before the evidence
existed. An earlier draft used `--sweep` as the Verify and
self-recursed; replaced with a terminating composite.

## Green (raw, all legs logged in `receipts/verify.log`)

- Sweep leg: `check_ticket_status.py --sweep`, exit 0:
  `0 stale-open, 0 broken-done, 0 vacuous, 0 honestly open`
  (raw line in `receipts/sweep.log`).
- Guard leg: `cargo test -p hephaestus --test cli
  reproduce_v1_committed`, exit 0:
  `1 passed; 0 failed; 27 filtered out`.
- Manifest leg: `fixture verify-manifest` on committed
  files, exit 0, `{"verdict":"Valid"}`.
- Format leg: `check_ticket_status.py --format`, exit 0:
  ok, 147 issue files.

## Implementation

Evidence gathering plus tracker files. The ticket Verify
chains guard, manifest, and format checks with `&&`, so any
leg fails the whole.

## Regressions

None touched; `make ci` gate runs before landing.

## Limitations

Same-machine checks only. AC 4 (rot filing) untested: a
round that finds rot covers it. Suite counts re-verified
through `make ci`, not counted here.

## Reviews

Two FAILs before landing: self-recursive Verify
replaced with a terminating composite; raw per-leg evidence
added to `receipts/verify.log` after a reviewer showed the
receipt cited logs it did not have. Final review PASS.
