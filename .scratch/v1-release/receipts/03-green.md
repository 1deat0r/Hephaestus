# 03 receipt — release packet assembly

Date: 2026-10-12 UTC.
Spec AC: 5, 6. No architectural departure.

## Red

Ticket Verify `test -f docs/RELEASE_DOSSIER_V1.md` failed:
file absent. CLI dossier tests failed: command unknown.

## Green

`cargo test --test cli dossier` + `assemble` pass. Ticket
Verify passes: dossier exists from a real gated run. S2 text
corrected during work: deterministic sections replay (the 02
precedent), not full byte identity.

## Implementation

`release/mod.rs`: `assemble_v1_dossier` measures fresh from the
trace, binds committed report digests, builds the packet
(Experimental label, observed-range uncertainty, five outcome
classes, quantity-only measures, two Minor open findings,
reproducibility pins, four research questions, one pinned
fixture benchmark), passes the R-077 gate, and renders the
dossier. `main.rs`: `fixture assemble-release`.

## Regressions

`cli` full target, `release_packet` suite in `make ci`.

## Limitations

Fixture scope only. Uncertainty is an observed range, never a
confidence interval. Two Minor findings stay open and visible.
No model, live, or confirmatory claim present. Latency and
memory sections vary run to run by design.

## Reviews

First review FAIL (7 items): empty-digest pin, foreign
provenance, infinite range, collapsed outcomes, hardcoded gate
header, thin twin, unpinned benchmark test. All fixed.
Second review PASS on all seven plus no new blocking defects.
