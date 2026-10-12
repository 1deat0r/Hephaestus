# 01: Run advisory, yank, and license scans

**Status:** done
**Verify:** test -f docs/SUPPLY_CHAIN_AUDIT.md
**Covers:** 1, 2, 3, 4
**Blocked by:** None (can start immediately)

## Scope

Run cargo audit and cargo deny against the frozen lockfile.
Record advisories, yanked crates with paths, license state,
and the direct-dep inventory. Upgrade nothing here.

**Small tasks:**

1. [x] **S1** Scan and record
   **Status:** done
   **Verify:** test -f docs/SUPPLY_CHAIN_AUDIT.md
   Run both tools, write the audit with limits explicit.
   **Micro-tasks:**
   1. [x] **M1** Scan, record, and write
      **Verify:** test -f docs/SUPPLY_CHAIN_AUDIT.md
      atomic

## Comments

Upgrades need a separate ticket with their own tests.
