# dev-roadmap spec - product-track tasks (Track B)

Status: ready-for-agent

## Acceptance Criteria

1. Fixture campaign driver runs a 20-mission batch end-to-end and
   emits pilot variance computed from the actual runs.
2. A real-corpus mission run ingests this repository's own docs and
   produces grounded opportunities from those real files.
3. A docs domain pack passes `domainpack::qualify` (R-006).
4. A `hephaestus fixture canary-watch` command runs the bounded
   monitor loop and exits with its receipt.
5. The exit assessment carries fresh campaign receipts for claims 1
   and 2 (grep-visible section).

## Notes

Track A (T-066 tickets 03-06) lives in its own feature dir and is not
covered here. Consent-gated items (provider keys, target corpus,
confirmatory campaign) stay `blocked` rows — never guessed.
