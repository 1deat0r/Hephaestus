# 01 receipt - supply-chain scans

Date: 2026-10-12 UTC.
Spec AC: 1, 2, 3, 4. No code changed; docs only.

## Red

Ticket Verify `test -f docs/SUPPLY_CHAIN_AUDIT.md`
failed: file absent.

## Green

cargo-audit 0.22.2 over 221 locked deps: zero advisories,
one yanked transitive with path and disposition. cargo-deny:
advisories, bans, sources ok; licenses default-noise stated
as noise. Direct deps inventoried; syn duplicate noted.

## Implementation

Two tool runs plus one audit document. No upgrades, no pins
touched.

## Regressions

None touched; `make ci` gate runs before landing.

## Limitations

Point-in-time scan, not monitoring. License posture waits
on a project deny.toml. All new files ASCII-checked before
landing (one slip caught mid-work, all files rechecked).

## Reviews

First review FAIL: ASCII slip in tracker files with
an inaccurate receipt claim, plus stale gaps row 8. Fixed
with full ASCII recheck, corrected wording, and a row update
linking the landed ticket. Second review PASS.
