# 01 receipt — provisional render guard

Date: 2026-10-12 UTC.
Spec AC: 1, 2, 3. Test-only change.

## Red

No behavior red: the funnel already excluded provisionals.
Guard addition, stated plainly.

## Green

`provisional_never_renders_as_achieved_in_v1_dossier`
passes: provisional hidden, measured shown.

## Implementation

One test in release_packet.rs plus gaps row 9 citing the
landed agent part.

## Regressions

`release_packet` full target in `make ci`.

## Limitations

Owner measuring (campaign numbers for live claims) stays
open in row 9.
