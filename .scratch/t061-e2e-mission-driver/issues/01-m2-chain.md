# 01: M2 chain e2e — goal to test-ready hypothesis on hidden-world fixture

**What to build:** `tests/e2e_m2.rs` driving mission compile →
fixture trace source derived from the synthetic-evaluator corpus →
ingest → span-cited traces → discovery::analyze → grounded
opportunities → genesis apply_all → hypothesis compile → validate →
TestReady, with structural assertions only (the test never reads
World truth — M2 exit wording honored).

**Blocked by:** None (can start immediately).

**Status:** done

- [x] e2e_m2 green: ≥1 grounded opportunity, ≥1 TestReady
      hypothesis, zero truth reads
- [x] Fixture data demonstrably derived from load_corpus() worlds
- [x] Gates green; no frozen-envelope changes

## Comments

Grill: `.scratch/t061-e2e-mission-driver/grill.md`.
Spec: `.scratch/t061-e2e-mission-driver/spec.md`.
