# 01 receipt — supply checks in CI

Date: 2026-10-12 UTC.
Spec AC: 1, 2, 3, 4. Config only; no dependency changes.

## Red

`make supply-check` failed: no rule to make target.

## Green

`make supply-check` exits 0 locally; raw output in
`receipts/check.log` (audit clean, all four deny checks ok).

## Implementation

Makefile `supply-check` target (audit plus four deny
checks), CI `supply` job with pinned installs (deny 0.20.2,
audit 0.22.2), SHA-pinned toolchain. `make ci` membership
unchanged. CI run itself stays unverified until a push
executes it — stated, not claimed.

## Regressions

`make ci` gate runs before landing (membership check
included).

## Limitations

Pinned installs cost CI minutes. The job has not executed
on CI yet; first push proves it.
