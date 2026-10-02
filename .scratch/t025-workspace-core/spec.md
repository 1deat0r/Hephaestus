# T-025 spec - workspace core: truthful views, controls, resumable gateway

Status: ready-for-agent

## Problem Statement

The runtime has no interactive workspace layer. Requirements pin the
contract core: progress grounded in persisted state (R-067), pause/resume/
cancellation/steering/event-cursor recovery (R-068), and identical policy +
authentication for gateway and local operations (R-069). TUI rendering is
presentation and comes later; this goal is the typed core the TUI will
render.

## Requirements trace

- R-067 / AT-067: progress and outcomes from persisted state, never
  narrated; unknown stays unknown.
- R-068 / AT-068: pause, resume, cancellation, steering, event-cursor
  recovery.
- R-069 / AT-069: same policy + authentication on gateway and local
  paths (one shared authorization function).
- §2:42: UI exposes real state and evidence.

## Acceptance Criteria

1. ProgressView reads persisted records; no invented percentages; unknown
   represented as unknown (R-067).
2. compare() diffs hypothesis records; drill_down() returns record +
   lineage + staleness.
3. Typed intervention commands with AuthToken; identical shared
   authorization for local and gateway paths (R-068/R-069).
4. Unauthorized command refused identically on both paths.
5. Event-cursor: stale/duplicate cursor handled; resume from persisted
   state (R-068).
6. Twin-run byte-identical views.

## Risks

- Over-engineering: no rendering, no crypto - typed contract core only.
- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
