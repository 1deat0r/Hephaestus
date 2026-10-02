# 01: Dossier evidence label — unmeasured exports can never read as real evidence

**What to build:** The R-082 half of the render path: every dossier
carries a required `EvidenceLabel` (measured-with-receipt or
unmeasured-with-reason), export refuses a measured label without a
receipt by name, and an unmeasured dossier built from the packaged
context-assembly example exports with the synthetic-fixture reason in
the output JSON — so no synthetic fixture can be rendered as real
experimental evidence.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] AT-082: packaged example (origin `synthetic_fixture`) →
      dossier labeled `Unmeasured { SyntheticFixture }` → export JSON
      carries both label and reason
- [x] `Measured { "" }` → `export` refuses
      (`MeasuredWithoutReceipt`); honest receipt-bearing `Measured`
      exports with the receipt visible
- [x] Glossary rows (Evidence label, Synthetic fixture); allowlist +3
      + reseal; gates green; tests cite R-082/AT-082

## Comments

Grill: `.scratch/t051-illustrative-unmeasured/grill.md`.
Spec: `.scratch/t051-illustrative-unmeasured/spec.md`.
