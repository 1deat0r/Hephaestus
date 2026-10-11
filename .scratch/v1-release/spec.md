# v1-release spec — road from v0.0.1 to v1.0.0

Status: ready-for-agent

## Problem Statement

The runtime is v0.0.1 EXPERIMENTAL. Implementation tickets are done.
Milestone exits M0 through M6 pass on machinery. The M4 ship clauses
read NOT-RUN: no campaign flown, no baseline comparison measured, no
release packet assembled. This track closes that gap with evidence.

## Acceptance Criteria

1. A pilot batch of at least 20 missions runs end to end with fresh receipts.
2. Variance, cost, and failure modes come from actual runs, not estimates.
3. Matched baseline comparisons back every release performance claim.
4. Model arms run only behind consented provider contracts; absences stay explicit.
5. The release packet assembles: scope label, findings gate, uncertainty, outcome counts, cost and latency quantities, reproducibility report.
6. No critical in-scope failure stays open at assembly; twin runs are byte-identical.
7. The version bumps to 1.0.0 with a changelog; release scopes leave "Future gates".
8. The scope label defaults to EXPERIMENTAL; QUALIFIED needs all five qualification inputs.

## Notes

Consent-gated items (provider keys, live spend, confirmatory scope)
stay blocked rows, never guessed. Machine-only comparisons proceed
first; consented arms follow after owner approval. No claim exceeds
its receipts at any step.
