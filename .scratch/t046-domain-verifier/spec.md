# T-046 spec - R-061/R-062/R-063: domain contract + adapter verifier gates

Status: ready-for-agent

## Problem Statement

R-061 (M0): semantic validation rejects schema-valid but semantically
contradictory records. R-062 (M0): provider SDK types stay outside
core interfaces; unsupported capabilities fail explicitly under
adapter swap. R-063 (M0): unknown schema versions / missing references
quarantine or reject with stable reason codes. The machinery exists
(semantic.rs port, backend adapter boundary, migrate fail-closed) but
no named verifier surface or R-named tests bind the three ATs.

## Requirements trace

- R-061/AT-061, R-062/AT-062, R-063/AT-063 (OBLIGATIONS; runtime M0,
  contract M0). Source: MASTER_SPEC §21.
- Continuity: semantic.rs is the ported reference validator (T-002);
  do not duplicate its logic.

## Design

`domainver` module — thin, pure verifier over existing surfaces:

- `TypedRejection { code, detail }` with STABLE codes:
  `SEMANTIC_CONTRADICTION`, `PROVIDER_LEAK`, `UNKNOWN_SCHEMA_VERSION`,
  `MISSING_REFERENCE`, `UNSUPPORTED_CAPABILITY`.
- `verify_cross_record(bundle: &Value) -> Result<(), Vec<TypedRejection>>`:
  runs the existing `semantic::` port; maps its findings onto stable
  codes (R-061).
- `verify_no_provider_leak(symbol_names: &[String]) -> Result`:
  fails with `PROVIDER_LEAK` when a core symbol name references a
  provider SDK marker (R-062); supported-capability check for adapter
  swap lives in the test via the backend contract.
- `verify_version_and_refs(schema_version, refs) -> Result`:
  unknown version -> `UNKNOWN_SCHEMA_VERSION`; unresolved ref ->
  `MISSING_REFERENCE` (R-063).

## Acceptance Criteria

1. AT-061: a schema-valid bundle with a contradictory relationship is
   rejected with `SEMANTIC_CONTRADICTION`.
2. AT-062: swapping adapters in the contract test surfaces no provider
   SDK type in core; unsupported capability fails with
   `UNSUPPORTED_CAPABILITY`.
3. AT-063: unknown version and missing reference produce stable codes.
4. Twin-run byte-identical rejection serializations.

## Risks

- Keep the verifier thin; semantic.rs remains the validator of record.
