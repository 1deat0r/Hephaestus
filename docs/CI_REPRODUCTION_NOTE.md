# CI reproduction arm — second-runner verification (2026-10-12 UTC)

Every push re-verifies the release artifacts on a GitHub
ephemeral runner, a genuine second machine.

## Chain

- Workflow `.github/workflows/ci.yml`, job `gates`, runs
  `make ci`.
- `make ci` runs `test-rust`: `cargo test --workspace`.
- The workspace suite includes `cli reproduce_v1_committed`,
  which rebuilds the binary, regenerates pilot, baseline, and
  dossier reports into temp space, and compares deterministic
  sections with the committed files.
- A drifted artifact fails the push gate. No human intervenes.

## Limits (explicit)

- Same repository revision, ephemeral Ubuntu runner: proves
  no-drift, not independent origination.
- Sandbox tests need the documented userns relaxation; the
  repro guard itself needs no special environment.
- Measured sections (latency, memory, compute) compare as
  labeled-varying, never as identical bytes.
- A second owner-controlled machine stays unproven and waits
  on the owner (gaps-register row 6, owner part).

## Status

Agent part of gaps row 6: done, running since the guard
landed. Recorded at main 3c3a44a421251673059f75b022a3d53e1493fdc4; CI runs this chain on every
push. Owner part: open.
