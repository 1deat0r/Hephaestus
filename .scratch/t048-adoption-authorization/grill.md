# T-048 grill - R-045: separate authorization for adoption, publication, or manufacturing

Goal: R-045/AT-045 (Invention Engine + dossier exporter; contract M0,
runtime M3, owner Realization, MASTER_SPEC §15) — a positive prototype
result never authorizes an external action; adoption, publication, and
manufacturing each need their OWN authorization.

Existing surfaces (facts):
- `security/approval.rs` = HMAC manifest approvals (R-060 lineage,
  generic byte-level approval) — not act-specific, carries no act
  semantics.
- `dossier` (T-023) exports results; `release` (T-028) gates release
  packets (R-077/078/080) — release scope ≠ external-act authorization.
- grep of src for adoption/publication/manufacturing: ZERO hits —
  genuine gap (R-044 dossier and R-046 orthogonal statuses are cited
  elsewhere).
- Pattern precedents: deny-first typed gates with named rejections
  (domainver T-046, release::ReleaseBlock T-028, review T-047);
  externally-supplied authority material (ApprovalKey "supplied from
  outside the system ... never from a candidate record").

## Q1 - What is the named surface?
**A:** New module `realization` (owner field: Realization) — an
external-act gate: typed `ExternalAct { Adoption, Publication,
Manufacturing }`, externally-recorded per-act authorizations, and
`request(...)` returning a version-bound receipt or a typed refusal.
The dossier exporter enforces R-045 by composition (its callers route
external actions through this gate); wiring export itself stays out of
scope, exactly as T-047 left `promote()` untouched. (agent-default)

## Q2 - How is "separate" modeled?
**A:** One `ActAuthorization` per act, bound to `subject` +
`subject_digest` + `granted_by` + `authority_receipt`. `request` for
act X requires an authorization FOR X on the same subject digest;
adoption-auth never satisfies publication, publication never satisfies
manufacturing (three-way deny test). Authorization records are input
from the standing authority — the module never grants (granting
decisions are the standing authority's, per the refusal taxonomy; the
API only records + enforces). (agent-default)

## Q3 - How does the negative case land?
**A:** AT-045 negative: "finish a positive prototype and request
automatic public deployment" — `request(Publication, ..., Justification:
PositivePrototypeResult{digest})` with no authorization ->
`Err(ResultNeverAuthorizes { act, justification_digest })` naming WHY:
the result is evidence, not authorization. With an authorization for a
DIFFERENT act -> `Err(MissingSeparateAuthorization)`. With an
authorization whose subject digest differs -> `Err(AuthorizationMismatch)`
(version-bound). Matching authorization -> `Ok(ActReceipt { act,
authorization_digest })`. (agent-default)

## Q4 - HMAC/ApprovalKey integration?
**A:** Out of scope: the gate composes with caller-supplied authority
receipts (opaque digest strings); binding the receipt to HMAC-tagged
bytes is R-060's existing surface and can wrap these records later
without API change (the field exists precisely for that). Keeps this
ticket single-purpose. (agent-default)

## Q5 - Files, tests, TDD?
**A:** `src/realization/mod.rs` (records + gate in one module — small
surface, single-file precedent exists), `tests/realization_gates.rs`
(deny-first: result-never-authorizes; three-way separate-ness; digest
mismatch; authorized path Ok), citation of R-045/AT-045 in module docs
and test names, GLOSSARY rows: External act, Separate authorization.
Allowlist +6 entries (mod.rs, tests, grill/spec/issue) + reseal.
Red-first at the gate seam (`request`). (agent-default)

## Q6 - Scope cuts?
**A:** (1) No dossier/release file changes — enforcement text's
"exporter" half is the composition contract recorded in the spec's out
-of-scope, not a T-048 edit; (2) no grant-issuing API (refusal
taxonomy: the module records, the standing authority grants);
(3) no commit/push (rule 5). (agent-default)
