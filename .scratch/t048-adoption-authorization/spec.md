# T-048 spec - R-045: separate authorization for adoption, publication, manufacturing

Status: ready-for-agent

## Problem Statement

R-045 (M3): adoption, publication, or manufacturing of a finished
candidate each require SEPARATE authorization — a positive prototype
result is evidence, never authorization (AT-045 negative: "finish a
positive prototype and request automatic public deployment" must not
self-authorize the external action). The codebase has generic HMAC
manifest approvals (R-060), result dossiers (T-023), and release
packets (T-028), but zero act-specific authorization surface (grep:
adoption/publication/manufacturing absent from src).

## Requirements trace

- R-045/AT-045 (OBLIGATIONS; enforcement: Invention Engine and dossier
  exporter; contract M0, runtime M3; owner: Realization).
  Source: MASTER_SPEC §15.
- Continuity: authority receipts compose with `security::approval`
  (R-060) later; release scope labels (T-028) are orthogonal.

## Design

New module `realization` — one seam, the gate:

- `ExternalAct { Adoption, Publication, Manufacturing }`.
- `Justification::PositivePrototypeResult { result_digest }` — the
  only caller-supplied justification, modeled precisely because it is
  NEVER sufficient.
- `ActAuthorization { act, subject, subject_digest, granted_by,
  authority_receipt }` — recorded via `record_authorization` from the
  standing authority (the module grants nothing itself).
- `request(&self, act, subject, subject_digest, justification)`:
  1. no authorization recorded for this subject at all (justification
     is a positive result) -> `ResultNeverAuthorizes { act,
     justification_digest }`;
  2. authorizations exist for this subject but only for OTHER acts ->
     `MissingSeparateAuthorization { act }`;
  3. same-act authorization with a different subject digest ->
     `AuthorizationMismatch`;
  4. matching same-act, same-digest authorization ->
     `Ok(ActReceipt { act, authorization_digest })` (version-bound
     runtime evidence retained).
- Typed rejection enum per the typed-rejection convention.

## Acceptance Criteria

1. **AT-045 negative**: positive prototype result alone -> Publication
   request refused with `ResultNeverAuthorizes`; no receipt exists.
2. **Separate-ness**: an Adoption authorization does not authorize
   Publication or Manufacturing (both refused with
   `MissingSeparateAuthorization`); each act needs its own record.
3. **Version-bound**: authorization for the wrong subject digest ->
   `AuthorizationMismatch`; correct digest -> `Ok(ActReceipt)` carrying
   the authorization digest.
4. Glossary rows: External act, Separate authorization.
5. Gates green (`make ci` + `make doc-check`), allowlist +6 + reseal,
   tests cite R-045/AT-045.

## Out of Scope

- Editing `dossier`/`release` files — the exporter half of the
  enforcement text is the composition contract (callers route external
  actions through the gate), recorded here, not wired.
- Grant-issuing APIs (the standing authority grants; the refusal
  taxonomy keeps granting out of agent-reachable code paths).
- HMAC binding of `authority_receipt` (R-060 surface wraps later).
- Committing or pushing (rule 5).

## Further Notes

Grill: `.scratch/t048-adoption-authorization/grill.md`.
