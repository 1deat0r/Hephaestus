# 02: Intervention controls + resumable authenticated gateway

**Status:** done

**What to build:** typed commands (Pause/Resume/Cancel/Steer) with
AuthToken, shared `authorize_control` used by BOTH local and gateway
paths, event-cursor handling (stale/duplicate -> replay recommendation),
resume-from-persisted-state.

**Acceptance:**
- [x] Shared authorization both paths (spec AC 3/4)
- [x] Cursor handling + resume (spec AC 5)
- [x] Twin-run byte-identical (spec AC 6)

## Comments
