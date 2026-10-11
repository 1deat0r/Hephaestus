# 01: Freeze the fixture campaign manifest

**Status:** done
**Verify:** test -f docs/CAMPAIGN_MANIFEST.json
**Covers:** 1, 2, 3
**Blocked by:** None (can start immediately)

## Scope

Add `fixture freeze-campaign` writing the manifest. Frozen
section replays exactly; provenance stays labeled run-local.
Fixture scope marked on the face of the file.

**Small tasks:**

1. [x] **S1** Freeze and prove replay
   **Status:** done
   **Verify:** test -f docs/CAMPAIGN_MANIFEST.json
   Implement the command, freeze the manifest, twin-check it.
   **Micro-tasks:**
   1. [x] **M1** Red-to-green freeze cycle
      **Verify:** test -f docs/CAMPAIGN_MANIFEST.json
      atomic

## Comments

Supports the future frozen_campaign input; is not one.
