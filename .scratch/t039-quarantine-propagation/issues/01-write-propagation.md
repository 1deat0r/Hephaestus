# 01: Memory write gate + quarantine propagation

**What to build:** `memory` module: MemoryArtifact/Label records,
`authorize_write` (scoped-grant gate), `propagate_quarantine`
(invalidate current dependents, append-only).

**Status:** done

**Acceptance:**
- [x] Write denial (spec AC 1)
- [x] Propagation + audit survival (spec AC 2/3)
- [x] Twin-run byte-identical (spec AC 4)

## Comments
