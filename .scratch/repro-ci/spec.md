# repro-ci spec — CI as the second reproduction runner

Status: ready-for-agent

## Problem Statement

Gaps-register row 6 needs cross-machine reproduction. A
second physical machine waits on the owner, but CI already
runs the full workspace suite on GitHub runners, including
the reproduction guard. That agent part needs recording with
its exact mechanism and limits.

## Acceptance Criteria

1. The note names the CI job, the gate chain, and the guard
   test that re-verifies artifacts per push.
2. Limits state what CI does not prove (same-repo, ephemeral
   runner, no independent corpus).
3. The gaps register reflects the landed agent part.

## Notes

Recording only. No workflow edits in this ticket; the
mechanism already runs.
