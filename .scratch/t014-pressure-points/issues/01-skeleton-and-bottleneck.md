# 01: Discovery module skeleton + bottleneck extraction (tracer bullet)

**What to build:** The `discovery` module with its input/output contract —
`TraceRecord` (structured authorized trace record with unit-tagged values and
span-bound evidence), `Opportunity` (all MASTER_SPEC:150 fields, validity
separate from polish), `RejectedCandidate` (reason-coded), and the first
operator: bottleneck extraction. A hidden performance bottleneck in a trace
(R-019/AT-019 negative case) becomes an emitted opportunity citing the slow
span; traces without a material bottleneck emit nothing. Twin runs
byte-identical.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] `TraceRecord` + `Opportunity` + `RejectedCandidate` types; `Opportunity` carries all §7:150 fields with explicit-unknown `Option`s and a `validity` field independent of narrative polish (R-021)
- [x] `analyze(&Corpus, &[TraceRecord]) -> AnalysisOutcome` seam, red-first
- [x] Bottleneck operator: material slow-span in trace → opportunity citing that span (evidence reference present)
- [x] No-bottleneck trace → zero opportunities, zero rejections (honest empty, not a claim)
- [x] Twin-run byte-identical output (determinism)
- [x] Every emitted opportunity has ≥1 evidence reference or explicit uncertainty (spec AC 6)

## Comments
Done 2026-10-01T06:18Z: `discovery/{record,operators,mod}.rs`; 4/4 in `discovery_pressure_points.rs` (red: unresolved module, then genuine threshold red — global-median rule was self-inflating, replaced with others-median ≥4x rule).
