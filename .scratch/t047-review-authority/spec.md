# T-047 spec - R-034/R-035/R-036: review authority service

Status: ready-for-agent

## Problem Statement

R-034 (M3): review votes must be recorded as judgments and can never
substitute empirical validation — an authoritative deterministic check
blocks promotion even when every model reviewer approves. R-035 (M3):
generator, analyzer, and promotion authorities are separated — an
evaluator-edit attempted from a candidate workspace is denied with an
audit event. R-036 (M3): concrete objections are retained with their
dispositions; resolving one objection must not hide another, and open
blocking objections prevent advancement. No vote, objection, or
review-authority concept exists in the codebase today (grep-verified),
and no named Review/Verification service surface carries these ATs.

## Requirements trace

- R-034/AT-034, R-035/AT-035, R-036/AT-036 (OBLIGATIONS; enforcement:
  Review and Verification services; contract M0, runtime M3).
  Source: MASTER_SPEC §12.
- Continuity: `selfimprove::promote` and `evaluation::interpret` stay
  the deterministic authorities (T-033/T-022) — the review service
  never reimplements or overrides them.

## Design

Single new seam — the `review` service (module with record + service
functions), operating on a version-bound `ReviewRecord` keyed by the
subject digest:

- `ReviewVote { reviewer_id, stance: Approve|Reject|Abstain, basis }`
  where `basis` is fixed to `Judgment` at construction; no numeric
  confidence fields; votes never convert into evidence records (R-034).
- `DeterministicOutcome::{Passed{receipt_digest}, Failed{reason},
  Absent}` — caller-supplied (from `selfimprove::evaluate_candidate`,
  `evaluation::interpret`, or equivalent); the service performs no
  computation of its own (R-034: the checker stays authoritative).
- `Authority::{Generator, Analyzer, Promoter, CandidateWorkspace}` and
  `request_evaluator_edit(record, authority, requested_digest)`:
  - every call appends an `AuditEvent { action:
    evaluator_edit_request, authority, requested_digest, outcome }`
    bound to the record's subject digest (version-bound evidence);
  - CandidateWorkspace / Generator / Promoter -> `AuthorityDenied`
    (AT-035 negative: candidate workspace denied + audit recorded);
  - Analyzer -> `Registered` as a proposal record only — the service
    never mutates any evaluator (analyzer evaluator-write power inside
    this service would collapse the separation; real evaluator
    replacement remains the protected out-of-band path).
- `Objection { id, statement, raised_by, blocking, disposition:
  Open|Resolved{note, resolved_by} }` — append-only list; `resolve`
  sets a disposition and never removes an entry (R-036).
- `advance(record, DeterministicOutcome)`:
  1. `Failed`/`Absent` -> `ReviewRejection::Deterministic{reason}`
     (votes are structurally not read on any path);
  2. any `Open && blocking` objection -> `ReviewRejection::
     OpenObjections{ids}`;
  3. else `Ok(AdvanceReceipt { subject, receipt_digest })`.
- `ReviewRejection` uses stable typed reason codes (typed-rejection
  convention, R-061 lineage).

## Acceptance Criteria

1. **AT-034**: votes are typed judgments (no empirical fields);
   unanimous Approve + deterministic `Failed` -> `advance` Denied;
   deterministic `Absent` -> Denied; deterministic `Passed` + no open
   blocking objections -> Ok — tallies never consulted.
2. **AT-035**: evaluator-edit from CandidateWorkspace (and Generator,
   Promoter) -> `AuthorityDenied` AND an audit event recorded on the
   record (action, authority, requested digest, outcome); Analyzer ->
   `Registered` proposal, evaluator bytes untouched.
3. **AT-036**: two objections raised, one resolved -> both remain
   visible with distinct dispositions and `advance` Denied on the open
   blocking one; all blocking resolved + deterministic Passed ->
   `advance` Ok.
4. Glossary rows: Review vote, Objection, Review authority.
5. Gates green: `make ci` + `make doc-check`; allowlist entries for
   every new tracked file; `make seal` after allowlist edit; citation
   of R-034/035/036 + AT-034/035/036 in tests.

## Out of scope

- Changing `selfimprove::promote` or `evaluation::interpret`
  signatures; wiring `advance` into callers (composition is the
  integration; a forced receipt parameter would churn T-033 tests).
- Any actual evaluator mutation/rotation machinery (stays protected
  out-of-band, T-019/T-033).
- New principal contract schemas (module records follow the T-024
  pattern; verify_package's 18-schema count is unchanged).
- Persistence/ledger wiring for records (T-006 integration later).
- Committing or pushing (rule 5: invocation authorizes neither).

## Further Notes

Grill Q&A: `.scratch/t047-review-authority/grill.md`. Decision rows:
`docs/agents/auto-workflow/decisions.md` (phase 2/3).
