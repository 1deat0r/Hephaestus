# 03: Release packet — assemble the v1 dossier

**Status:** done
**Verify:** test -f docs/RELEASE_DOSSIER_V1.md
**Covers:** 5, 6
**Blocked by:** 01, 02 (needs campaign and comparison receipts)
**Requirements:** R-077, R-078, R-080

## Scope

Assemble the typed release packet through the T-028 machinery:
scope label, security findings gate, finite-suite uncertainty,
all five outcome classes, quantity-only cost and latency, and a
reproducibility report. Critical in-scope failures block
assembly with a named block.

**Small tasks:**

1. [x] **S1** Assemble packet from campaign receipts
   **Status:** done
   **Verify:** test -f docs/RELEASE_DOSSIER_V1.md
   Feed pilot and comparison receipts into the assembler; resolve
   or scope every critical finding first.
   **Micro-tasks:**
   1. [x] **M1** Assemble and gate the packet
      **Verify:** test -f docs/RELEASE_DOSSIER_V1.md
      atomic

2. [x] **S2** Verify deterministic sections replay
   **Status:** done
   **Verify:** test -f docs/RELEASE_DOSSIER_V1.md
   Re-run the assembly; deterministic sections match exactly
   while measured sections stay labeled as varying (02 precedent).
   **Micro-tasks:**
   1. [x] **M1** Re-run and compare deterministic sections
      **Verify:** test -f docs/RELEASE_DOSSIER_V1.md
      atomic

## Comments

Plan track for the v0.0.1 to v1 road. Blocked until 01 and 02
produce receipts.
