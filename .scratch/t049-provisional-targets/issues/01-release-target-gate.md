# 01: Release-verifier gate for provisional targets vs measured benchmarks

**What to build:** The R-076 half of the protected release verifier:
performance claims are typed `ProvisionalTarget` or `MeasuredBenchmark`
(the latter pinned to receipt + reference machine), the verifier
refuses a forged measured claim with the named block, and the single
achieved-benchmark display funnel can never emit a provisional p95
target — so an unimplemented feature's target cannot be presented as
an achieved benchmark while honest targets and real measurements both
still record.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] AT-076: provisional p95 target for an unimplemented feature is
      recordable but absent from `achieved_benchmarks`
- [x] Empty-receipt / empty-machine measured claim →
      `UnmeasuredTargetDisplayed` from `assemble_release`
- [x] Fully-pinned measured claim passes and appears in the funnel
- [x] Glossary rows (Provisional target, Achieved benchmark); allowlist
      +3 + reseal; gates green; tests cite R-076/AT-076

## Comments

Grill: `.scratch/t049-provisional-targets/grill.md`.
Spec: `.scratch/t049-provisional-targets/spec.md`.
