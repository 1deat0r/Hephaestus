# prov-targets spec — provisional targets stay provisional

Status: ready-for-agent

## Problem Statement

Gaps-register row 9 needs its agent part: confirm every
provisional target path funnels through achieved_benchmarks
and none renders as achieved. The release funnel has unit
coverage; the v1 dossier renderer needs its own guard.

## Acceptance Criteria

1. A render-level test proves provisional targets never
   display in the v1 dossier while pinned benchmarks do.
2. The inventory names every performance_claims touchpoint.
3. The gaps register reflects the landed agent part.

## Notes

Guard addition; the funnel behavior already holds. Owner
measuring stays open.
