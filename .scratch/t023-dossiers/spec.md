# T-023 spec - raw-data capture, cost receipts, reproduction, dossiers

Status: ready-for-agent

## Problem Statement

Results (T-022) exist in memory but nothing captures raw data + cost
receipts, verifies clean-environment reproduction, or exports a dossier.
MASTER_SPEC §15:299-300 defines the dossier contents and negative-result
semantics; R-044 requires reproducible export including negative results
and raw artifacts with both histories and separate conclusions; R-103
requires complete cost receipts as quantities, not invented prices.

## Requirements trace

- R-044 / AT-044: reproducible dossier, negative results + raw artifacts,
  both histories present with separate conclusions.
- R-003: negative result is a legitimate outcome (first-class).
- R-080: positive, negative, inconclusive, invalid, blocked all handled.
- R-103: cost receipts as quantities, no invented conversion price.
- R-097: version-bound lineage recorded.
- §15:299: full dossier field list; §15:300: negative-result kinds.

## Acceptance Criteria

1. RunRecord captures raw measurements, pinned environment, deviations,
   cost receipts as quantities (no price on human time).
2. `reproduce`: Reproduced / EnvironmentMismatch / Disagrees as typed
   outcomes.
3. Dossier carries all §15:299 fields; lineage vector recorded (R-097).
4. Negative dossier distinguishes the five §15:300 kinds; failure history
   and negative-test conclusions stay SEPARATE (R-044).
5. Export gate refuses missing raw data / missing histories / empty repro
   commands / priced human time.
6. Twin-run byte-identical export JSON.

## Risks

- Over-engineering: no filesystem I/O in this module - export returns the
  JSON document; writing is the caller's act.
- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
