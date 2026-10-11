# Release dossier v1 - fixture scope

Scope label: EXPERIMENTAL (default; qualification inputs all absent).

Declared scope: fixture pilot batch + deterministic baseline comparison (v0.0.1 track).

Campaign: prereg-fixture-campaign-1 over 20 missions; failures retained: 0.

## Comparison carried

baseline base: correctness 0.000000, tokens: 2975, cost 0.0, latency total 10 ms, throughput mean 1355121.789 tok/s, peak memory max 4579328 bytes
candidate cand: correctness 1.000000, tokens: 2975, cost 0.0, latency total 10 ms, throughput mean 1340517.812 tok/s, peak memory max 4579328 bytes

false promotions: 0 (no gate ran; count by absence).

## Outcome counts (all five classes)

positive: 10
negative: 10
inconclusive: 0
invalid: 0
blocked: 0

## Uncertainty (observed range, not a confidence interval)

suite prereg-fixture-campaign-1; n 20; observed cand-arm value range (0.330538, 0.333560).

## Cost and latency (quantities only)

- tokens: 5950 fixed-tokenizer-tokens
- latency: 20 ms-wall-local-total
- wall-time: 4416851 ns-wall-local-total
- cpu-time: 4399934 ns-cpu-local-total
- peak-memory: 4579328 bytes-vmhwm-max

## Findings (R-077 gate: passed: no critical unresolved finding in scope)

- compute-wall-time-metered [Minor, resolved] Wall and CPU time metered per run by local clocks; currency spend stays absent.
- model-arms-absent [Minor, open] No consented provider contract exists; no model arm ran.

## Reproducibility

- pin os: linux
- pin arch: x86_64
- run `hephaestus fixture pilot-batch --trace <TRACE> --out docs/PILOT_CAMPAIGN_REPORT.md`
- run `hephaestus fixture baseline-compare --trace <TRACE> --out docs/BASELINE_COMPARISON_REPORT.md`
- run `hephaestus fixture assemble-release --trace <TRACE> --pilot docs/PILOT_CAMPAIGN_REPORT.md --baseline docs/BASELINE_COMPARISON_REPORT.md --out docs/RELEASE_DOSSIER_V1.md`
- input docs/PILOT_CAMPAIGN_REPORT.md: digest present: true
- input docs/BASELINE_COMPARISON_REPORT.md: digest present: true

## Unresolved research questions

- Do pilot margins hold on live missions with owner consent?
- Do consented model arms beat deterministic baselines at matched envelopes?
- What is the metered compute cost per qualified outcome?
- Which scope qualifies for QUALIFIED_FOR_DECLARED_SCOPE, and with what frozen campaign?

## Benchmarks pinned

- fixture-cand-arm-positive-margin-rate: positive-margin fraction over 10 fixture repos = 1 fraction (receipt pilot-report:c67110b0274b9a360da1e07d32f67014147384a95a68e729bb7357eddadabe0e; machine linux-x86_64)

## Input digests (run-local provenance)

- pilot report: c67110b0274b9a360da1e07d32f67014147384a95a68e729bb7357eddadabe0e
- baseline report: 8ebd32f677ca180bf0342b0aecf8932fed8b7e480ba3a42d1153df025f340341
