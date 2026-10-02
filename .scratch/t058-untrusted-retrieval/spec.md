# T-058 spec - R-058: retrieved instructions and generated code untrusted

Status: ready-for-agent

## Problem Statement

R-058 (M1): retrieved instructions and generated code are untrusted —
a paper ingested with instructions to disclose secrets or bypass
policy must be inert, and the security test records no effect
(AT-058 negative → required outcome). Zero retrieved-instruction code
exists (run-24 re-audit): content is retrieved as data but no
directive path exists to be inert ON.

## Requirements trace

- R-058/AT-058 (OBLIGATIONS; enforcement: PolicyEngine,
  SandboxProvider, protected monitor; contract M0, runtime M1).
  Source: MASTER_SPEC §20.
- Continuity: PolicyEngine stays pure/state-free; PolicyRequest's
  fixed shape (R-052) untouched; generated-code containment is
  SandboxProvider's (R-059/R-064) — composition, recorded.

## Design

Extend `policy::engine`:

- `RetrievedDirective { text, source_locator }` — untrusted by
  origin; no conversion into PolicyRequest exists.
- `ReasonCode::RetrievedInstructionInert` (appended at enum end).
- `PolicyEngine::evaluate_retrieved_directive(&RetrievedDirective)
  -> PolicyDecision`: always `{ allowed: false, reasons:
  [RetrievedInstructionInert] }`; pure, takes no mission/grant/context
  — inert by construction.

## Acceptance Criteria

1. **AT-058**: malicious paper ingested (real
   `knowledge::ingest_bytes`), retrieved (search hit = data), directive
   evaluated → denied with exactly `RetrievedInstructionInert`.
2. No effect: mission/grant fixtures byte-identical after the
   attempt (never inputs); decision records the single inert reason
   (the security test's "no effect" record).
3. `evaluate()` and all existing reason flows unchanged (existing
   policy tests stay green).
4. Tests cite R-058/AT-058 (negative maps verbatim).
5. Gates green; allowlist +3 + reseal; glossary rows: Retrieved
   directive, Inert instruction.

## Out of Scope

Generated-code containment (sandbox composition, R-059/R-064);
PolicyRequest changes; monitor/event-ledger wiring beyond the pure
decision record. Committing (rule 5).

## Further Notes

Grill: `.scratch/t058-untrusted-retrieval/grill.md`.
