# license-policy spec — project license allowlist

Status: ready-for-agent

## Problem Statement

The supply-chain audit found no deny.toml: license posture
undecided, cargo-deny licenses red on default noise. This
track adds an explicit allowlist matching the locked tree,
deny-by-default, with clarifications for non-SPDX forms.

## Acceptance Criteria

1. `cargo deny check licenses` passes on the frozen lockfile.
2. The allowlist names permissive licenses only; copyleft and
   unknown licenses stay denied by default.
3. Non-SPDX license forms carry explicit clarifications, not
   silent passes.
4. LGPL and CDLA entries record why they pass (dual-license
   alternatives, permissive data).

## Notes

Policy records a decision; it does not upgrade anything.
Upgrades stay separate tickets.
