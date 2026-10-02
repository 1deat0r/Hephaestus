# 02: Rollback + improvement ledger

**Status:** done

**What to build:** `rollback` (verified incumbent restore, reason+observations
retained, same-digest retry rejected), append-only `ImprovementLedger`
(deployed + rejected candidates with provenance; restart = re-read).

**Acceptance:**
- [x] Rollback semantics (spec AC 4)
- [x] Ledger append-only + restart reconstruction (spec AC 6)
- [x] Twin-run byte-identical (spec AC 7)

## Comments
