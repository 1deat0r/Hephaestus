# Sandbox flake note — reproduction attempt (2026-10-12 UTC)

Question: why did sandbox_deny fail once in CI while passing
alone and on retry.

## Logged campaign (evidence)

- Solo series: 5 runs, 17/17 pass each, load average near 2.
  Log: `.scratch/flake-hunt/receipts/solo.log`.
- Spinner-loaded series: 6 runs, 17/17 pass each, load 5-8.
- Live-CI-contention series: 4 runs during `make ci` (exit 0),
  17/17 pass each, load near 6.
  Log: `.scratch/flake-hunt/receipts/loaded.log`.
- Logged total: 15 runs, 15 green, 0 failures.

## Anecdotes (not evidence)

Two failures appeared in terminal-only runs outside the logs,
at load 10-13, in timing and namespace-setup tests
(`wall_timeout_kills_a_runaway` among others). One further
attempt lost its output to a shell filter. None of these is
logged; none counts as evidence. They motivate suspicion of
load, nothing more.

## Verdict: inconclusive

The logged campaign did not reproduce the failure. The
original CI failure cause remains unestablished. No
correlation claim follows: the anecdotes point at load, the
logs show 15 clean runs including loaded ones.

## Consequence

- CI sandbox failures without code changes retry before
  investigation; this note records why that policy exists.
- A fix needs its own ticket with a failing test first. None
  exists yet.
- Gaps-register row 10 cites this note as inconclusive, not
  as a correlation finding.
