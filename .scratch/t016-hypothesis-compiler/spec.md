# T-016 spec — Hypothesis Compiler and semantic validators

Status: ready-for-agent

## Problem Statement

M2's chain stops at mechanisms: T-015 emits structured MechanismRecords, but
nothing transforms them into operational hypotheses — context, intervention,
comparator, predictions, estimands, effect bounds, competing explanations,
and falsifiers — with semantic validation gating test-ready status
(IMPLEMENTATION_PLAN:56, MASTER_SPEC §9, R-025/026/027).

## Solution

Extend `genesis` with a pure hypothesis compiler: `compile(mechanism) ->
Hypothesis` (all §9:178 fields; readiness `TestReady | Exploratory |
BlockedTestability`), semantic `validate()` that denies test-ready
promotion with structured reasons (no discriminator, stripped falsifiers or
competitors, missing comparator, self-fulfilling metric), threshold
provenance enforcement (never compiler-invented numbers, R-026), and
`revise()` versioning where substantive changes block silent inheritance of
confirmatory support (R-027). Mechanistic claims and engineering targets
are separate fields with separate conclusions (§9:180).

## User Stories

1. As a genesis caller, I want a mechanism compiled into a hypothesis with
   all §9:178 fields, so that every downstream test request is complete.
2. As a readiness caller, I want semantic validation to DENY test-ready
   when falsifiers or competing explanations were stripped (AT-025
   negative), so that untestable-but-polished candidates never promote.
3. As a genesis caller, I want a hypothesis without a credible
   discriminator to stay `EXPLORATORY` (never deleted), so that potential
   value is preserved without mislabeled readiness (§9:182).
4. As a metrics caller, I want every threshold and effect bound to carry
   provenance, so that arbitrary targets are returned for completion
   (R-026/AT-026 negative).
5. As a lineage caller, I want revision to mint a new version where a
   changed mechanism/boundary/endpoint/falsifier forces support
   reassessment, so that confirmatory support is never silently inherited
   (R-027/AT-027 negative).
6. As a reviewer, I want mechanistic claims and engineering targets
   separate, so that "target met for unrelated reasons" has its own
   conclusion.

## Out of Scope

- Search orchestration over hypotheses (T-017), prior-art (T-018), running
  experiments (M3). No numbers invented anywhere (thresholds come only from
  provenance-cited sources).

## Acceptance Criteria

1. Compile a mechanism → hypothesis with all 12 §9:178 fields present.
2. Stripped falsifiers or competitors → `validate` denies test-ready with
   structured reasons; hypothesis remains (not deleted) (AT-025).
3. No-discriminator hypothesis → `Exploratory`, preserved (§9:182).
4. Threshold without provenance → returned for completion (AT-026).
5. Revision with changed mechanism → new version, `NeedsReassessment`
   forced (AT-027); unchanged fields inherit normally.
6. Mechanism claim and engineering target are separate fields; a target
   met without mechanism support is representable as inconclusive-for-
   mechanism.
7. Twin-run compile/revise byte-identical; no compiler-minted thresholds.
8. `cargo test --workspace` green; clippy `-D warnings`; fmt; `make ci`
   EXIT=0.

## Success Risks

- Validators becoming checkbox theater: each denial reason must name the
  §9:184 question it answers.
- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
