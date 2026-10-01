# 02: Ambiguity taxonomy, value frame, authorization requests

**What to build:** Reversible ambiguities compile through into explicit
assumptions while spending/disclosure/irreversible/risk ambiguities come back
as authorization requests; goals without a value frame fail closed instead of
inventing values (R-011).

**Blocked by:** 01 (record + compile seam must exist first).

**Status:** done

- [x] Terminology/vocabulary/corpus/subdomain ambiguity → Mission with explicit assumption entries + provenance
- [x] Unapproved spending, external disclosure, irreversible op, ambiguous risk → `NeedsAuthorization` with exact reason codes (deny-matrix style)
- [x] "Invent something useful" with no priorities/profile → `MissingValueFrame`, no Mission
- [x] Sensitive-application goal → authorization request, never a silent sensitive Mission
- [x] Caller-supplied hypothesis recorded as provenance marker only — hypothesis text never stored, never validated

## Comments
Done 2026-10-01: genuine red→green (irreversible + unsupervised-risk failed first, 5 passed); fix added the two documented tripwire lists. 7/7 in `mission_auth.rs`.
