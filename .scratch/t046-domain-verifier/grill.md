# T-046 grill - R-061/R-062/R-063: domain contract + adapter verifier gates

Goal: R-061/R-062/R-063 (Domain contract and adapter verifier,
runtime M0) — semantic rejection of contradictory records, provider
leak-freedom, and typed failure/quarantine for unknown versions or
missing references.

Existing surfaces:
- R-061: `semantic.rs` ports the reference cross-record validator and
  semantic_parity.rs asserts parity — but no test NAMES R-061 or
  exercises its AT: "schema-valid but semantically contradictory
  records" -> "semantic validation rejects the invalid relationship".
  The validator DOES reject contradictory relationships (duplicate /
  unresolvable reference / kind mismatch); coverage gap is the
  R-061-named AT scenario.
- R-062: backend adapter boundary exists (ExecutionBackend trait,
  LocalProcess, TachyonStatus blocked); no provider SDK types in core
  is structurally true, but AT-062 "swap a model and retrieval adapter
  in contract tests; unsupported capabilities fail explicitly" has no
  test.
- R-063: migrate.rs fails-closed on unknown schema versions with
  stable reasons (NotRegistered etc.) — but "missing reference ->
  stable reason code" for domain records and the R-063-named test are
  absent.

## Q1 - What to build?
**A:** A verifier gate module `domainver` that composes the three ATs
as one named contract surface (semantic-reject, provider-leak,
typed-failure) plus tests wiring EXISTING surfaces into the named
ATs. Minimal new logic; the point is the R-named contract, not a
second validator. (agent-default)

## Q2 - New logic needed?
**A:** Small:
1. `verify_cross_record(bundle) -> Result<(), TypedRejection>` — wraps
   `semantic::` port with typed, stable reason codes (R-061/AT).
2. `verify_no_provider_leak(core_symbols) -> Result` — scan core
   module symbol names for provider SDK markers (openai/anthropic/
   http-client types), fail with `PROVIDER_LEAK` (R-062/AT).
3. `TypedRejection` with stable codes incl. UNKNOWN_SCHEMA_VERSION,
   MISSING_REFERENCE (R-063/AT). (agent-default)

## Q3 - Why not just tests against existing modules?
**A:** The ATs require a named enforcement surface ("Domain contract
and adapter verifier"); a pure-function verifier the tests call IS
that surface, kept thin over proven ports. (agent-default)

## Q4 - TDD?
**A:** Red-first: contradictory-record rejection; adapter-swap no-leak;
unknown-version/missing-reference stable codes. (agent-default)

## Q5 - Vocabulary?
**A:** GLOSSARY row: Typed rejection. (agent-default)
