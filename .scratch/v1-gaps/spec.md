# v1-gaps spec — one tracked register of v1 blockers

Status: ready-for-agent

## Problem Statement

Open items scatter across the dossier, the posture audit, and
the release track. No single register names each gap, its
owner, and whether it waits on owner consent or agent work.
This track writes that register and keeps it current.

## Acceptance Criteria

1. Every dossier question and posture gap appears exactly once
   with owner and consent flag.
2. Each entry states what unblocks it and what claim it gates.
3. Agent-workable entries link their landing ticket or state
   none exists yet.
4. The register carries a fresh date and a no-silent-drop rule:
   entries close only with evidence or explicit owner call.

## Notes

The register tracks gaps; it closes none. Consent-flagged
entries wait on the owner, never on guesses.
