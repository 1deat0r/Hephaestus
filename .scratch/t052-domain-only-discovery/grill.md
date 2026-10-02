# T-052 grill - R-073: evaluate domain-only discovery, not just refinement of seeded ideas

Goal: R-073/AT-073 (Protected campaign evaluator; contract M0, runtime
M4, MASTER_SPEC §25) — negative: "Run the full benchmark without
supplying hypotheses." Required: "The generator is evaluated on
independently originated opportunities and mechanisms."

Existing surfaces (facts):
- evalsuite (T-026): `EvalSuite`, `define_baseline`, `check_matched`,
  `ablation`; ArmKind = ActiveRetrievalModel / FixedGenerateReview /
  SimpleSearch / DomainMethod — NO domain-only discovery path.
- eval_suite.rs header cites R-074/R-075 (already-covered siblings) —
  R-073 has no named surface at all.
- "Independently originated" maps to the lineage the system already
  models: opportunity + mechanism ids (genesis/§8-9).

## Q1 - What is the seam?
**A:** `begin_benchmark(&[OriginatedHypothesis])` in evalsuite — the
campaign evaluator's admission gate. `OriginatedHypothesis {
hypothesis_id, opportunity_id, mechanism_id }` (non-empty lineage =
independently originated). (agent-default)

## Q2 - Refusals?
**A:** Empty input slice -> `BenchmarkError::NoSuppliedHypotheses`
(AT-073 negative, verbatim). Any hypothesis with an empty opportunity
or mechanism id -> `BenchmarkError::NotIndependentlyOriginated {
hypothesis_id }` (required outcome: evaluation runs on independently
originated opportunities AND mechanisms). Success ->
`BenchmarkSession { hypothesis_ids }` (the admitted cohort, observable
for receipts downstream). Typed-rejection convention. (agent-default)

## Q3 - Why not an ArmKind discovery arm?
**A:** The obligation gates the BENCHMARK RUN (admission), not one
comparison arm — a discovery arm without the admission gate would let
the negative case through (benchmark runs with zero hypotheses). Arm
kinds stay T-026's; this gate composes before any arms are used.
(agent-default)

## Q4 - Files/gates?
**A:** evalsuite/record.rs (OriginatedHypothesis + error/session
types), evalsuite/mod.rs (gate + exports); extend tests/eval_suite.rs
(header gains R-073 + 2 deny-first tests); allowlist +3 (.scratch
t052); reseal; glossary rows: Domain-only discovery, Independently
originated. Citations R-073/AT-073. Red-first at begin_benchmark.
No commit (rule 5). (agent-default)
