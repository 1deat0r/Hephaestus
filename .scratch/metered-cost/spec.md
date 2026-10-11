# metered-cost spec — measure local compute per run

Status: ready-for-agent

## Problem Statement

The v1 dossier records compute and wall time as unmetered. v1
needs metered cost per outcome. Currency spend stays absent
(no metered provider exists), but local wall time and CPU time
are measurable with the standard library plus libc, both already
in use. This track meters them per run and carries the quantities
into the comparison and dossier reports.

## Acceptance Criteria

1. Every deterministic comparison run records wall nanoseconds
   and CPU nanoseconds from real clocks around the actual work.
2. The baseline report carries a compute section with per-arm
   totals from measured runs, labeled local and varying.
3. The dossier cost section lists compute quantities; the old
   unmetered finding resolves with a named successor.
4. Twin runs replay deterministic fields; measured compute varies
   and stays labeled as varying.

## Notes

No provider currency exists to meter; cost stays 0 by absence.
Clocks measure the chain call only, with tokenizer and harness
overhead disclosed. Linux-only sources degrade to explicit gaps.
