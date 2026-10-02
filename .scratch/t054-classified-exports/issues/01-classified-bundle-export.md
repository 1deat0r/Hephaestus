# 01: Classified bundle export — mixed states only with per-record status, evidence, scope, reproduction

**What to build:** The R-092 bundle-export gate: exploratory,
test-ready, and validated records exported together are refused unless
each record carries its own status (typed), evidence ids (non-empty),
scope (non-empty), and reproduction state (typed) — the refused
bundle names the records that lack classification, and the accepted
bundle serializes each record's four facts without collapsing them
into one label.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] AT-092: unclassified mixed bundle → `MissingEvidence`/
      `MissingScope` naming the record
- [x] Classified mixed bundle → `Ok`, JSON shows distinct
      per-record statuses
- [x] Status/reproduction typed-present (compile-enforced), asserted
      in test
- [x] Glossary rows (Classified record, Reproduction state); allowlist
      +3 + reseal; gates green; tests cite R-092/AT-092

## Comments

Grill: `.scratch/t054-classified-exports/grill.md`.
Spec: `.scratch/t054-classified-exports/spec.md`.
