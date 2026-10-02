# T-049 grill - R-076: label performance targets provisional until measured

Goal: R-076/AT-076 (Protected release verifier; contract M0, runtime
M4, MASTER_SPEC §26) — an unimplemented feature with a p95 target must
not be displayable as an achieved benchmark.

Existing surfaces (facts):
- §26 first paragraph IS this requirement: "Targets below are
  provisional engineering budgets to be measured on a recorded
  reference machine. They are not achieved results." (10 ms/100 ms
  p95 budgets, <10% orchestration overhead).
- `release::assemble_release` (T-028) is the protected release
  verifier for R-077/078/080/R-103 — R-076 has no code at all (zero
  `target`/`p95`/`benchmark` hits in src).
- `ReleasePacket.cost_latency: Vec<Measurement>` = quantity-only
  achieved numbers (R-103); `ScopeLabel` already EXPERIMENTAL unless
  five qualifications (R-092/093 overlap — why those were deferred).
- ReleasePacket is constructed in exactly two places (record.rs +
  tests/release_packet.rs) — field addition is low-churn.

## Q1 - What is the claim model?
**A:** `PerformanceClaim` enum — `ProvisionalTarget { feature,
metric_label, target_value }` (the §26 budget; by definition not an
achieved result) vs `MeasuredBenchmark { feature, metric_label,
measured_value, unit, benchmark_receipt, reference_machine }`
(§26's "measured on a recorded reference machine"). The enum makes
"provisional shown as achieved" unrepresentable at the type level.
(agent-default)

## Q2 - Where does the verifier gate live?
**A:** Inside `assemble_release` (the enforcement named by the
obligation): every `MeasuredBenchmark` must carry a non-empty
receipt, measured value, and reference machine — else
`ReleaseBlock::UnmeasuredTargetDisplayed { feature }`. Provisional
targets may be RECORDED (§26 explicitly records them). (agent-default)

## Q3 - How is "cannot display as achieved" pinned?
**A:** One display funnel: `achieved_benchmarks(&packet) ->
Vec<&MeasuredBenchmark>` returns ONLY measured claims; the AT-045-style
negative test asserts a provisional p95 target for an unimplemented
feature is recorded in the packet (legal) yet the achieved funnel
excludes it, and a forged MeasuredBenchmark with an empty receipt is
refused by the verifier. Dossier/UI consume this funnel — wiring those
callers is composition, out of scope (T-047/T-048 precedent).
(agent-default)

## Q4 - Field addition churn?
**A:** `ReleasePacket.performance_claims: Vec<PerformanceClaim>` —
two construction sites (record.rs, tests/release_packet.rs);
existing tests updated with `performance_claims: vec![]`. No schema
count change (module records, not principal contracts).
(agent-default)

## Q5 - Files, tests, TDD?
**A:** release/record.rs (claim types + packet field),
release/mod.rs (verifier check + funnel), extend EXISTING
tests/release_packet.rs (deny-first: forged measured claim refused;
provisional excluded from funnel; proper measured accepted) — no new
test file → allowlist +3 (.scratch t049 grill/spec/issue) + reseal.
Glossary rows: Provisional target, Achieved benchmark. Citations
R-076/AT-076 in module docs + test names. Red-first at
`assemble_release`. (agent-default)

## Q6 - Scope cuts?
**A:** (1) workspace/dossier view wiring — composition recorded in
spec; (2) §26 numeric budgets themselves stay documentation (the
labeling gate is this ticket; measuring budgets is qualification work);
(3) no commit/push (rule 5). (agent-default)
