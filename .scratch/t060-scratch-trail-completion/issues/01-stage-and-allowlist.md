# 01: Stage t014..t040 artifacts and allowlist them

**What to build:** every historical .scratch/t014..t040 run artifact
(and this ticket's own files) staged byte-preserved with per-file
runtime_allowlist entries and a fresh seal — archived STATE references
resolve in-repo again, manifest-check counts reconcile.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] Zero untracked .scratch paths; allowlist covers every newly
      tracked file; manifest-check green
- [x] `make ci` + `make doc-check` green; staged blobs byte-identical
      to worktree (stage-only)

## Comments

Grill: `.scratch/t060-scratch-trail-completion/grill.md`.
Spec: `.scratch/t060-scratch-trail-completion/spec.md`.
