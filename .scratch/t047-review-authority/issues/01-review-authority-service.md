# 01: Review authority service — judgment votes, separated authorities, retained objections

**What to build:** The `review` service per spec: a version-bound
`ReviewRecord` that records review votes as judgments (never consulted
for eligibility), denies evaluator-edit requests from candidate/generator
and promotion scopes with a recorded audit event while registering
analyzer proposals without mutating any evaluator, retains every
objection with its disposition, and gates `advance` on the caller's
deterministic outcome plus open blocking objections — so unanimous
approval can never bypass the authoritative checker and an unresolved
blocking objection always stops advancement.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] AT-034: unanimous-approve + deterministic Failed/Absent denied;
      vote records typed judgment with no empirical fields; deterministic
      Passed + no open blocking objections -> advance Ok
- [x] AT-035: candidate-workspace (and generator/promoter)
      evaluator-edit denied with audit event recorded; analyzer ->
      registered proposal, evaluator bytes untouched
- [x] AT-036: resolve one of two objections -> both visible with
      distinct dispositions and advance Denied; all blocking resolved +
      deterministic Passed -> advance Ok
- [x] Glossary rows (Review vote, Objection, Review authority);
      allowlist entries for every new file; `make seal`; gates green
      (`make ci` + `make doc-check`); tests cite R-034/035/036 +
      AT-034/035/036

## Comments

Grill: `.scratch/t047-review-authority/grill.md`.
Spec: `.scratch/t047-review-authority/spec.md` (Status: ready-for-agent).
