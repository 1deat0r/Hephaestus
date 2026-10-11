# 04 receipt — ship v1.0.0 EXPERIMENTAL

Date: 2026-10-12 UTC.
Spec AC: 7, 8. No architectural departure.

## Red

Ticket Verify `grep -q '^version = "1.0.0"' Cargo.toml`
failed: version read 0.1.0.

## Green

Version reads 1.0.0; workspace lock refreshed for both
crates; changelog written; release scopes leave Future gates.

## Implementation

Root `Cargo.toml` version to 1.0.0 (workspace crates follow;
lock refreshed by build). New `CHANGELOG.md` records the
fixture-scope shipment and its explicit exclusions.
`release-scopes.json` status names the measured parts and the
unrun parts. Label stays EXPERIMENTAL: all five qualification
inputs absent, per the dossier packet.

## Regressions

`make ci` and `make doc-check` before landing.

## Limitations

Version number marks a scoped EXPERIMENTAL shipment, not
maturity. Live, model, metered, and confirmatory work stays
unrun and stated. The v0.01 packet doc stays as history.
