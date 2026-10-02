# T-047 grill - R-034/R-035/R-036: review authority (votes, authorities, objections)

Goal: R-034/R-035/R-036 (Review and Verification services; contract M0,
runtime M3) — votes are judgments never empirical validation;
generator/analyzer/promotion authority separated; objections retained
with dispositions.

Existing surfaces (facts):
- No vote/objection/authority concept exists anywhere in crates+python
  (grep: zero hits; `judge_score: Option<String>` in evalsuite is a
  score, not a vote; knowledge's "single vote" is span dedup).
- `selfimprove::promote` is already a deterministic authority gate
  (self-attestation, holdout, confounds, artifacts, benefit, guardrails,
  budget, scoped grant) — it never reads opinions. R-034's negative
  case ("all model reviewers approve a candidate with a deterministic
  error -> authoritative checker blocks") is composition: review votes
  must exist WITHOUT ever feeding that checker.
- `evaluation::interpret` returns typed validity (Invalid/NotAssessed
  ...) — an existing deterministic outcome to link, not reimplement.
- Evaluator protection exists at plan level (evaluator digest mismatch
  -> Invalid) but no operation-level "edit evaluator from candidate
  workspace -> denied + audit event" surface.
- Record pattern precedent: T-024 lifecycle/{mod,record}.rs (serde
  structs + fns, no frozen-contract schema addition; verify_package's
  18 principal schemas unchanged).
- File-shape precedent: T-046 = grill + spec + issues/01 + module +
  lib.rs line + one tests file + runtime_allowlist entries + reseal.

## Q1 - What object is reviewed, and where does review live?
**A:** Standalone module `src/review/` operating on a version-bound
`ReviewRecord` keyed by subject digest (promotion candidates,
hypotheses, artifacts — generic; callers decide the subject). Votes and
objections attach to the record; `advance` is the gate surface.
Enforcement wording in OBLIGATIONS is "Review and Verification
services" — a service module is the named surface, mirroring T-046's
named-verifier precedent. (agent-default)

## Q2 - How do votes avoid becoming empirical validation (R-034)?
**A:** `ReviewVote { reviewer_id, stance, basis: Judgment }` — `basis`
is FIXED to `Judgment` at construction; the struct carries no numeric
confidence/posterior fields and records never convert into evidence
records. Eligibility logic (`advance`) structurally never reads votes —
only the caller-supplied deterministic outcome and open blocking
objections. Negative test: unanimous Approve votes + deterministic
Failed -> Denied, tallies irrelevant. (agent-default)

## Q3 - What is the deterministic input?
**A:** `DeterministicOutcome::{Passed{receipt_digest}, Failed{reason},
Absent}` supplied by the caller (selfimprove::evaluate_candidate
result, evaluation::interpret validity, ...). Review never reimplements
determinism; `Absent` or `Failed` denies. Keeps the deterministic
checker authoritative by construction (R-034: "authoritative checker
blocks promotion despite agreement"). (agent-default)

## Q4 - Authority separation API (R-035)?
**A:** `Authority::{Generator, Analyzer, Promoter, CandidateWorkspace}`
and `request_evaluator_edit(record, authority, requested_digest)`:
every call appends an `AuditEvent{action: evaluator_edit_request, ...}`
(version-bound: subject digest + requested digest). CandidateWorkspace,
Generator, Promoter -> `Err(AuthorityDenied)` + audit (negative case:
candidate workspace edit denied WITH audit event). Analyzer ->
`Ok(Registered)` recorded as a proposal ONLY — the service never
mutates any evaluator (analyzer holding evaluator-write power inside
this service would collapse the separation; actual evaluator
replacement stays the protected out-of-band path of T-019/T-033).
(agent-default)

## Q5 - Objection lifecycle (R-036)?
**A:** `Objection { id, statement, raised_by, blocking, disposition:
Open | Resolved{note, resolved_by} }` — append-only list; `resolve`
sets a disposition, never deletes. `advance` denies while any
Open && blocking exists (reason lists their ids). Negative test:
resolve one of two -> BOTH remain visible (one Resolved, one Open) and
advance Denied. Positive: all blocking resolved + deterministic Passed
-> Ok(AdvanceReceipt). (agent-default)

## Q6 - Persistence shape?
**A:** T-024 pattern: serde records in `review/record.rs`, service
functions in `review/mod.rs`; in-memory + serializable, no new
principal contract schema (T-024/T-046 precedent; the 18 schema count
stays). (agent-default)

## Q7 - TDD seams and files?
**A:** Public entry points: `record_vote`, `request_evaluator_edit`,
`raise_objection`, `resolve_objection`, `advance`. One tests file
`tests/review_service.rs` (deny-first cases + positives, both — the
T-024/T-046 single-file precedent). Tests name R-034/035/036 and
AT-034/035/036 verbatim (citation scan). Red-first: unanimous-approve
still denied; candidate-workspace edit denied+audited; resolve-one
keeps both visible and blocks. (agent-default)

## Q8 - Vocabulary (domain-modeling, inline)?
**A:** GLOSSARY rows: **Review vote** (judgment, not validation),
**Objection** (blocking or not; disposition retained), **Review
authority** (one of generator/analyzer/promoter; scope-bound).
_Avoid_ lines per format. (agent-default)

## Q9 - Integration with `selfimprove::promote`?
**A:** Non-invasive: `promote` stays untouched (already authoritative);
`review::advance` is a composable pre-gate. Forcing a review receipt
into `promote`'s signature would exceed R-034's requirement (the
negative case is satisfied compositionally) and would churn T-033's
tests. Logged as scope cut: wiring `advance` into callers is a later
ticket if the spec demands it. (agent-default)

## Q10 - Gates/allowlist?
**A:** +6 `tools/runtime_allowlist.txt` entries (.scratch/t047 grill,
spec, issues/01, review/mod.rs, review/record.rs,
tests/review_service.rs) then `make seal` (allowlist is sealed). No
commit this run (rule 5); a later authorized commit of gate files
cites an ADR per commit-msg hook — noted for the committing run.
(agent-default)
