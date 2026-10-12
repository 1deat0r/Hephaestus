# 01: Add the license allowlist

**Status:** done
**Verify:** cargo deny check licenses
**Covers:** 1, 2, 3, 4
**Blocked by:** None (can start immediately)

## Scope

Write deny.toml with the explicit allowlist plus
clarifications. Green the licenses check. Record LGPL and
CDLA reasoning in the receipt.

**Small tasks:**

1. [x] **S1** Allowlist and green the check
   **Status:** done
   **Verify:** cargo deny check licenses
   Inventory done; write policy, clarify forms, go green.
   **Micro-tasks:**
   1. [x] **M1** Red-to-green policy cycle
      **Verify:** cargo deny check licenses
      atomic

## Comments

Policy only. No dependency changes.
