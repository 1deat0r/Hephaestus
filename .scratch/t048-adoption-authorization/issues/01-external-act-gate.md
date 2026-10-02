# 01: External-act gate — adoption, publication, manufacturing each separately authorized

**What to build:** The `realization` gate per spec: a positive
prototype result can never authorize an external action — requesting
publication (or adoption, or manufacturing) on the strength of a
positive result alone is refused by name; each act needs its OWN
subject-digest-bound authorization recorded from the standing
authority; an authorization for one act never covers another; a
mismatched subject digest is refused; a matching authorization yields
a version-bound receipt.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] AT-045: positive-result-alone publication request refused
      (`ResultNeverAuthorizes`), no receipt
- [x] Adoption authorization does not authorize publication or
      manufacturing (`MissingSeparateAuthorization` each)
- [x] Wrong subject digest -> `AuthorizationMismatch`; right digest ->
      `Ok(ActReceipt)` with authorization digest
- [x] Glossary rows (External act, Separate authorization); allowlist
      entries for every new file (+5) + reseal; gates green; tests cite R-045/AT-045

## Comments

Grill: `.scratch/t048-adoption-authorization/grill.md`.
Spec: `.scratch/t048-adoption-authorization/spec.md`.
