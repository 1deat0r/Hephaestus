# T-060 grill - scratch trail completion (t014..t040)

Goal: the run artifacts of T-014..T-040 (27 dirs: grill/spec/issues,
103 files) are untracked AND unallowlisted while t041+ artifacts are
tracked+allowlisted — and archived STATE blocks reference those spec
paths (e.g. spec: .scratch/t014-pressure-points/spec.md). The trail
hole dates from the original 4207825 split (that commit's tree
carried t041+ scratch only — verified at session start).

## Q1 - What to do?
**A:** Stage every untracked .scratch/t014..t040 file byte-preserved
(historical records — zero content edits) plus this run's own t060
artifacts; append per-file `tools/runtime_allowlist.txt` entries
following the t041+ precedent; `make seal`; gates. (agent-default)

## Q2 - Why not fold into the MANIFEST envelope?
**A:** .scratch is runtime history, deliberately NOT frozen
spec-package content (manifest hint: runtime files belong on the
allowlist; envelope changes need versioned reseal — out of scope).
(agent-default)

## Q3 - Verification?
**A:** git status shows zero untracked .scratch dirs; manifest-check
green with tracked/allowlisted counts up by the exact file count;
`make ci` + doc-check green; byte-preservation spot-check (diff
against pre-stage copies is impossible — but nothing is edited: only
`git add`, verified by staged-vs-worktree identity). (agent-default)
