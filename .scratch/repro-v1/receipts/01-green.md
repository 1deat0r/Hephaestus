# 01 receipt — reproduction guard test

Date: 2026-10-12 UTC.
Spec AC: 1, 2, 3, 4. No architectural departure.

## Red

No behavior red exists: reproduction already held. The guard
is new coverage, stated as such in the ticket. Verify was
vacuous before the test existed, green after.

## Green

`reproduce_v1_committed` passes: pilot byte-identical,
baseline and dossier deterministic sections replay against
the committed files. Measured sections excluded by design.

## Implementation

`cli.rs`: `det_lines` keeps deterministic lines and strips
measured values riding on shared lines; dossier input-digest
lines normalize as run-local provenance. One test regenerates
all three reports into temp space.

## Regressions

`cli` full target in `make ci`.

## Limitations

Same-machine reproduction only. The guard proves no drift,
not portability. Dossier digest lines track regeneration
inputs by construction.

## Reviews

First review FAIL: guard dropped false promotions,
run counts, per-run digests, and pins while the comment
claimed them. Filters widened; comment corrected. Second
review PASS with no new defects.
