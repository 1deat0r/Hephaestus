# 03 receipt — verify-manifest command

Date: 2026-10-12 UTC.
Spec AC: 1, 2, 3. Completes the frozen-campaign track.

## Red

`cli verify_manifest_cmd` failed: command unknown.

## Green

Valid manifest exits 0 with `{"verdict":"Valid"}`;
tampered trace exits 1 naming `trace_sha256`. Help documents
the command.

## Implementation

`main.rs`: `verify-manifest` parses the manifest file,
re-verifies against live inputs, prints the typed verdict.
`campaign.rs`: serde derives on `ManifestVerdict`.

## Regressions

`cli` full target in `make ci`.

## Limitations

Same-machine verification only. Verdict JSON mirrors the
enum; new variants surface automatically.
