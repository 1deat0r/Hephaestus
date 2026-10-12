# supply-chain spec - dependency audit at v0.2.0

Status: ready-for-agent

## Problem Statement

Gaps-register row 8 needs a supply-chain audit pass. The repo
ships a frozen lockfile but never recorded an advisory scan,
a yanked-crate check, or a license-policy state. This track
runs the scans with installed tools and records results.

## Acceptance Criteria

1. A RustSec advisory scan runs against the frozen lockfile
   with the advisory count recorded.
2. Yanked crates list with their dependency paths and a
   disposition each.
3. License-policy state recorded honestly, including the
   missing project policy file.
4. Direct dependencies inventoried with pins; duplicates noted.

## Notes

The audit scans; it upgrades nothing without a separate
ticket. Default-tool noise stays labeled as noise, never as
findings.
