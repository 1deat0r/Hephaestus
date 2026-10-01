# 03: Concurrency proof, glossary terms, and gate evidence

**What to build:** The AT-055 concurrency half demonstrated under real
contention (many threads, small shared budget, invariant holds), the domain
vocabulary recorded, and the full gate suite green with evidence — closing
the run.

**Blocked by:** 01 (core budget ledger).

**Status:** ready-for-agent

- [ ] Multi-thread test: N threads × M `reserve` attempts through a
      `std::sync::Mutex<BudgetLedger>` against a small limit; final assert
      `spent + reserved + unresolved <= limit` and every success/failure is
      accounted (AT-055 / R-055)
- [ ] Determinism: identical call sequences on equal inputs yield equal
      totals (no clock/randomness in the module)
- [ ] GLOSSARY gains: budget ledger, reservation, unresolved charge
      (decision row written **before** the edit, per domain.md)
- [ ] `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace`,
      `make doc-check`, `make ci` all green; evidence lines in the state
      LOG; test headers cite R-055/R-056/AT-055/AT-056
- [ ] Known limitations recorded (in-process-only guarantee, no durability
      by design) in the spec/report, not hidden
