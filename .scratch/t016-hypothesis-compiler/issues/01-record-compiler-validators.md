# 01: Hypothesis record + compiler + semantic validators (R-025/026)

**What to build:** `genesis/hypothesis.rs` + `genesis/validators.rs`:
`Hypothesis` with all 12 MASTER_SPEC:178 fields (mechanism claim and
engineering target SEPARATE, §9:180) and readiness `TestReady |
Exploratory | BlockedTestability`; `compile(mechanism) -> Hypothesis`;
`validate()` denying test-ready with structured reasons when falsifiers or
competing explanations are stripped, comparator missing, no operational
discriminator exists, or the hypothesis restates its own success metric
(AT-025 negative). Every threshold/effect bound carries provenance — an
unprovenanced target is returned for completion (AT-026 negative); the
compiler never invents numbers. No-discriminator hypotheses stay
`Exploratory` (never deleted, §9:182).

**Blocked by:** None (can start immediately).

**Status:** done

- [x] `Hypothesis` with all 12 §9:178 fields + separate mechanism claim / engineering target (spec AC 1, 6)
- [x] `compile(mechanism) -> Hypothesis`, red-first
- [x] `validate` denies test-ready on stripped falsifiers/competitors, missing comparator, no discriminator, self-fulfilling metric (AT-025) — structured reasons naming the §9:184 question
- [x] No-discriminator hypothesis → `Exploratory`, preserved (spec AC 3)
- [x] Unprovenanced threshold → returned for completion; compiler mints no numbers (AT-026)

## Comments

## Comments
Done 2026-10-01T06:48Z: 8/8. Denial reasons name the governing clause (AT-025/026, MASTER_SPEC:182/184/186). Readiness semantics: no-discriminator -> Exploratory (preserved); provenance-blocked -> BlockedTestability; substance denials -> Exploratory until completed.
