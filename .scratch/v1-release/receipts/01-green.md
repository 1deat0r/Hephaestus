# 01 receipt — pilot campaign fixture batch

Date: 2026-10-12 UTC.
Spec AC: 1, 2. No architectural departure.

## Red

Ticket Verify `test -f docs/PILOT_CAMPAIGN_REPORT.md` failed:
file absent. CLI tests `pilot_batch_*` failed: `fixture
pilot-batch` unknown (usage error).

## Green

`cargo test --test cli pilot_batch` → 2 passed. Ticket Verify
passes: report exists. Command rerun is byte-identical (test).

## Implementation

`campaign.rs`: `render_pilot_report` renders plan, 20 mission
receipts, variance, denominators, cost honesty note, failures.
`main.rs`: `fixture pilot-batch --trace --out` wires the batch
to the file. Report generated from a real run over the
world-derived trace fixture.

## Regressions

Full `cli` target plus `fixture_campaign` rerun in `make ci`.

## Limitations

Fixture batch only: variance calibrates debugging, not evidence.
Provider spend absent by construction, stated not implied.
Compute and wall time unmetered, stated. Live missions out of
scope pending owner consent.
