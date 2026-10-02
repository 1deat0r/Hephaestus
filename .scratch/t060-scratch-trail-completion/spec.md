# T-060 spec - scratch trail completion

Status: ready-for-agent

## Problem Statement

Archived STATE blocks reference .scratch/t014..t040 spec paths that
exist only untracked on this machine; t041+ are tracked+allowlisted.
The trail must be in-repo (staged) and manifest-covered (allowlisted)
without altering a single historical byte.

## Requirements trace

Workflow trail integrity (rule 11: disk is the durable truth);
no R obligation. Precedent: t041+ allowlist entries.

## Design

`git add` all untracked `.scratch/t01*..t040` files + t060's own
artifacts; append per-file runtime_allowlist entries; `make seal`.

## Acceptance Criteria

1. Zero untracked `.scratch/` paths remain (excluding none — all
   scratch staged).
2. Allowlist contains an entry for every newly tracked file;
   manifest-check green with matching counts.
3. `make ci` + `make doc-check` green; no file content changed
   (stage-only; staged blob == worktree bytes).
4. This run's grill/spec/issue tracked too.

## Out of Scope

MANIFEST envelope changes, content edits, commit (rule 5).

## Further Notes

Grill: `.scratch/t060-scratch-trail-completion/grill.md`.
