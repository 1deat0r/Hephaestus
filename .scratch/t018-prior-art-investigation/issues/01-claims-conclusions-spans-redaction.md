# 01: Conclusion enum, claim model, span verification, redacted queries

**Status:** done

**What to build:** The `priorart` module core: `AtomicClaim` (component,
mechanism, use context, regime, claimed result), `Conclusion` enum
(exactly the five §10:203 values), `PassageRef` (source + byte range +
captured text), `verify_span` (captured text must match corpus bytes),
`SearchPlan` with `redacted_queries` derived by stripping
mechanism-specific tokens, `full_disclosure_approved` default false.

**Acceptance:**
- [x] Conclusion enum closed - exactly 5 variants (spec AC 1/2)
- [x] verify_span passes on matching fixture bytes; rejects mismatched captured text (spec AC 3)
- [x] Redacted queries strip mechanism tokens; unredacted gated (spec AC 4)

## Comments
