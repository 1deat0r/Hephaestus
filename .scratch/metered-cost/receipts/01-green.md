# 01 receipt — wall and CPU time per run

Date: 2026-10-12 UTC.
Spec AC: 1, 2. No architectural departure.

## Red

`acceleration_eval metered_compute` failed: E0432, no
`ArmCompute` or `measure_arm_compute` in `acceleration`.

## Green

`metered_compute_clocks_each_run` passes: 10 runs per arm,
wall totals above zero, CPU totals present-or-gap. Baseline
report carries the Compute section from the same clocking
pass. CLI and library suites green.

## Implementation

`acceleration/mod.rs`: `cpu_now_ns` reads the process CPU
clock through libc (None on failure, never zero);
`MeasuredRun` carries wall and CPU nanoseconds;
`summarize_compute` totals per arm with gap propagation;
`measure_arm_compute` serves one clocking pass for both
comparison and compute. Report section labels scope and
variance. Currency stays 0 by absence.

## Regressions

`cli`, `acceleration_eval` in `make ci`.

## Limitations

Clocks wrap the chain call only; tokenizer and harness time
excluded and disclosed. CPU time is process-wide, not
per-run isolated. Non-Linux clocks degrade to labeled gaps.
