# 01: Campaign benchmark admission gate — originated hypotheses or refusal

**What to build:** The R-073 admission seam in the eval suite: a
benchmark run refuses to start with no supplied hypotheses, refuses
any hypothesis lacking opportunity/mechanism lineage (not
independently originated), and otherwise returns the admitted cohort —
so the generator is always evaluated on independently originated
opportunities and mechanisms, never on an empty or seeded-only run.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] AT-073: `begin_benchmark(&[])` → `NoSuppliedHypotheses`
- [x] Empty opportunity_id or mechanism_id →
      `NotIndependentlyOriginated` naming the hypothesis; full lineage
      → `Ok(BenchmarkSession)` with admitted ids
- [x] Glossary rows (Domain-only discovery, Independently originated);
      allowlist +3 + reseal; gates green; tests cite R-073/AT-073

## Comments

Grill: `.scratch/t052-domain-only-discovery/grill.md`.
Spec: `.scratch/t052-domain-only-discovery/spec.md`.
