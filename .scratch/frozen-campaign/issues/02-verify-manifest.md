# 02: Verify the manifest against live files

**Status:** done
**Verify:** cargo test -p hephaestus --test campaign_manifest verify_manifest
**Covers:** 1, 2, 3
**Blocked by:** None (can start immediately)

## Scope

Add manifest re-verification: check internal consistency,
bind the frozen plan to code, recompute trace and report
digests from live bytes, and return a typed verdict (Valid,
DigestMismatch, VersionMismatch). Byte-slice inputs cannot
fail reads, so no UnreadableInput variant exists. A manifest
that no longer matches refuses as stale, never as valid.
Cover with positive, refusal, and regression tests.

**Small tasks:**

1. [x] **S1** Verify digests with typed verdicts
   **Status:** done
   **Verify:** cargo test -p hephaestus --test campaign_manifest verify_manifest
   Implement verification plus the three test classes.
   **Micro-tasks:**
   1. [x] **M1** Red-to-green verification cycle
      **Verify:** cargo test -p hephaestus --test campaign_manifest verify_manifest
      atomic

## Comments

Makes the frozen artifact checkable. Supports the future
frozen_campaign qualification input; is not one.
