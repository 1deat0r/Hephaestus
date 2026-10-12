# 01 receipt — license allowlist policy

Date: 2026-10-12 UTC.
Spec AC: 1, 2, 3, 4. Config only; no dependency changes.

## Red

`cargo deny check licenses` FAILED on default noise (no
project policy).

## Green

`cargo deny check licenses` exits 0 on the frozen lockfile;
raw output in `receipts/check.log` (one unmatched-allowance
note for Zlib stands; exit stays 0).

## Implementation

`deny.toml`: explicit permissive allowlist, deny-by-default
sources, six slash-shorthand exceptions with SPDX readings,
yanked warn, LGPL covered by dual-license alternatives,
CDLA data license allowed with note.

## Regressions

None touched; `make ci` gate runs before landing.

## Limitations

Policy records intent; upgrade nothing. License posture
decided at this revision; re-run on lock changes.

## Reviews

First review FAIL: wrong num-cmp version, missing
CDLA/LGPL rationale, no check log. Fixed all three plus a
deprecated key. Second review PASS.
