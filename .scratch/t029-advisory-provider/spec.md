# T-029 spec - advisory provider contract layer

Status: ready-for-agent

## Problem Statement

A decision provider (Jev or equivalent) may do bounded ADVISORY work, but
deterministic control-plane gates stay authoritative (R-005, AGENTS.md).
No provider exists to integrate; this goal is the contract layer: the
typed advisory interface, calibration/abstention/rejection measurement
records over fixtures with known outcomes, provider-identity labeling
(R-090), and structural proof that advisory input cannot move control
gates. No fabricated calibration numbers.

## Requirements trace

- R-005 / AT-005: core workflow runs without Jev/Tachyon/GPU (NullProvider
  + module isolation).
- R-090 / AT-090: official Jev vs third-party implementations distinct.
- §16 continuity: model probabilities stored as model JUDGMENTS.
- R-103 continuity: cost as quantities.

## Acceptance Criteria

1. AdvisoryDecision carries recommendation + confidence-as-model-judgment
   + rationale ref.
2. `evaluate_provider` measures calibration buckets, abstention rate,
   rejection false negatives over fixtures with known outcomes;
   NotMeasured elsewhere.
3. Provider identity labeled official vs third-party (R-090).
4. `assert_control_authority` attestation: advisory cannot change gates,
   mint budget, or qualify methods.
5. NullProvider (always abstains) works — core runs without a provider.
6. Twin-run byte-identical reports.

## Risks

- Fabricating calibration: computed ONLY from known-outcome fixtures.
- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
