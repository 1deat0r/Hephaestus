# T-027 spec - prospective pilot campaigns

Status: ready-for-agent

## Problem Statement

The confirmatory campaign needs a pilot layer: preregistered metrics +
analysis (R-103), stratified sampling, train/pilot/confirmatory partitions
separated BY REPOSITORY, variance estimation from pilot outcomes, and a
gate blocking premature confirmation. Twenty missions is not a threshold.

## Requirements trace

- R-103 / AT-103: preregistered sampling, denominators, costs; partition
  separation; episodes sampled before confirmation.
- Campaign: stratify by size and dependency topology; partitions
  separated by repository; pilot for variance, not evidence.

## Acceptance Criteria

1. PilotPlan carries preregistration id, stratification, partition
   assignment (one partition per repository — violations named).
2. Batch size free; debug-vs-evidence intent recorded; no twenty-rule.
3. `record_outcome` yields means, variances, paired-difference variance,
   failure counts retained in denominators; empty -> explicit None.
4. `ready_for_confirmation` refuses: missing variance, unresolved oracle,
   confirmatory-partition contamination, unqualified analysis.
5. Twin-run byte-identical.

## Risks

- Fabricating variance from thin data: arithmetic on recorded outcomes
  only; None when absent.
- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
