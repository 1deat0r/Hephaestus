# 01 receipt — frozen campaign manifest

Date: 2026-10-12 UTC.
Spec AC: 1, 2, 3. No architectural departure.

## Red

Ticket Verify `test -f docs/CAMPAIGN_MANIFEST.json`
failed: file absent. CLI freeze tests failed: command unknown.

## Green

`cli freeze` → 2 passed. Manifest frozen from a real run:
plan, trace digest, report digests, package version bound;
frozen section replays byte-identical across twins.

## Implementation

`campaign.rs`: frozen manifest types plus marker-checked
freeze and canonical JSON render. `main.rs`:
`freeze-campaign` command with help lines. Fixture scope
marked on the face of the file.

## Regressions

`cli` full target in `make ci`.

## Limitations

Freezes inputs, not conclusions. Supports a future
frozen_campaign input; is not one. Toolchain facts are
provenance, never frozen content.

## Reviews

First review FAIL (9 items, mostly comment and
coverage gaps): misleading digest comment removed, screening
labeled as screening, render comment matched, byte identity
asserted same-machine, refusal and digest tests added.
Second review PASS with no new defects.
