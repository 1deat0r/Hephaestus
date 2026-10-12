# supply-ci spec — cargo-deny and audit run in CI

Status: ready-for-agent

## Problem Statement

License and advisory posture checks run only on a developer
machine. Drift between pushes goes unseen until someone runs
them by hand. This track wires both tools into CI per push,
outside `make ci` like doc-check.

## Acceptance Criteria

1. A Makefile `supply-check` target runs cargo audit plus
   cargo deny advisories, licenses, bans, and sources.
2. A CI job installs pinned tool versions and runs the target
   on push and pull request.
3. Tool installs pin versions; mutable install refs stay out.
4. Local `make ci` stays untouched in latency and membership.

## Notes

CI definition lives in one place per command (ADR-025):
the Makefile target owns the command, the workflow owns
scheduling.
