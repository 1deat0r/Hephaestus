# 02 receipt — manifest re-verification

Date: 2026-10-12 UTC.
Spec AC: 1, 2, 3. No architectural departure.

## Red

`campaign_manifest verify_manifest` failed: E0432, no
`ManifestVerdict` or `verify_manifest` in `campaign`.

## Green

3 passed: matching bytes validate, tampered bytes name the
field, foreign version names both versions, twins agree, and
the committed manifest file parses and validates.

## Implementation

`campaign.rs`: `ManifestVerdict` (Valid, DigestMismatch,
VersionMismatch) plus `verify_manifest` recomputing digests
in fixed order with first-mismatch-wins. Version binding
means code changes re-freeze.

## Regressions

`campaign_manifest`, `fixture_campaign` in `make ci`.

## Limitations

Byte-level verification only; semantic drift (same bytes,
new meaning) stays out. Cross-machine independence unproven.

## Reviews

First review FAIL (10 items): top-level fields
unchecked, plan unbound, single tamper path, ticket text
mismatch. Fixed with consistency checks, plan binding, full
tamper coverage, and ticket correction. Second review PASS.
Tests executed green separately (4 passed).
