# frozen-campaign spec — freeze the qualifying campaign inputs

Status: ready-for-agent

## Problem Statement

The QUALIFIED label needs a frozen campaign input, but no
artifact binds the preregistered plan, the trace bytes, the
report digests, and the code version into one checkable
manifest. Rebuilding that binding later invites drift. This
track freezes it now from the fixture campaign.

## Acceptance Criteria

1. A freeze command binds preregistration, plan, trace digest,
   report digests, and package version into one manifest.
2. The frozen section replays byte-identical across twin runs;
   toolchain provenance stays labeled as run-local.
3. The manifest marks the campaign as fixture scope, never as
   confirmatory evidence.

## Notes

The manifest freezes inputs, not conclusions. It supports a
future frozen_campaign qualification input; it is not one.
No live data enters. Toolchain strings are provenance, not
frozen content.
