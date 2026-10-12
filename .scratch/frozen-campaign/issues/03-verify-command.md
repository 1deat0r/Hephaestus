# 03: Verify manifests from the CLI

**Status:** done
**Verify:** cargo test --test cli verify_manifest_cmd
**Covers:** 1, 2, 3
**Blocked by:** None (can start immediately)

## Scope

Add `fixture verify-manifest` reading a manifest file plus
live inputs and printing the typed verdict. Exit 0 on Valid,
exit 1 otherwise. Stale files refuse end to end, not just in
library tests.

**Small tasks:**

1. [x] **S1** Surface verification in the CLI
   **Status:** done
   **Verify:** cargo test --test cli verify_manifest_cmd
   Implement the command with valid and stale paths tested.
   **Micro-tasks:**
   1. [x] **M1** Red-to-green command cycle
      **Verify:** cargo test --test cli verify_manifest_cmd
      atomic

## Comments

Closes the prior review note: no CLI surfaced the verdict.
