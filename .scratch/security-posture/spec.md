# security-posture spec — v1 security gates audit at v0.2.0

Status: ready-for-agent

## Problem Statement

v1 needs passing security gates, but no single document states
the current posture with fresh receipts: which deny suites ran,
what they returned, which findings stay open, and what was not
tested. This track records that posture honestly.

## Acceptance Criteria

1. Every deny and containment suite re-runs this session with
   counts recorded.
2. Sandbox results repeat across runs; flakes get named, not
   hidden.
3. The posture document lists open findings and untested areas
   explicitly.
4. No finding is marked resolved without its resolving evidence.

## Notes

An audit records posture; it does not certify security. Live
adversarial testing stays unrun and stated.
