# 04: Assumptions + changed capabilities + speculative validity (R-021 end-to-end)

**What to build:** Two operators plus the R-021 end-to-end guarantee.
(a) Assumption mining: unstated assumptions declared in trace metadata
(`assumed` records) surface as opportunities carrying the assumption and its
explicit uncertainty. (b) Changed capabilities: a newly verified capability
(`capability` record with verified availability) emits an opportunity only
with a concrete account of which prior constraint it changes
(MASTER_SPEC:144); availability without the changed-constraint account is
rejected. (c) Validity independence: a polished, well-written candidate with
no evidence stays `speculative` with low validity regardless of narrative
completeness (R-021/AT-021) — polish never lifts validity; and a
conjecture-only candidate is constructible but flagged `speculative`
(MASTER_SPEC:150).

**Blocked by:** 01 (seam + types).

**Status:** done

- [x] Declared unstated assumption → opportunity with explicit uncertainty
- [x] Verified capability + concrete changed-constraint account → opportunity (spec AC 5)
- [x] Verified capability without changed-constraint account → rejected with reason
- [x] Polished unevidenced candidate → `speculative`, validity low despite complete narrative (AT-021)
- [x] All six operators wired through `analyze`; module docs state scope + T-015 boundary

## Comments
Done 2026-10-01T06:23Z: 6 new tests, 19/19. `assess_validity` derives the verdict from attached evidence; `narrative_polish` counts filled §7:150 fields — the two accessors are independent by construction and both directions are tested (polished+unevidenced → speculative; thin+evidenced → evidence-backed). Capability operator requires a matching `constraint` record.
