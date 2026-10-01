# T-043 spec - R-108: trust-origin propagation gate

Status: ready-for-agent

## Problem Statement

R-108 requires preserving trust origin through summaries, caches, graph
edges and cross-session memory. `TrustOrigin` exists on mission
provenance, but derived records carry no origin: trust can be laundered
by summarizing/caching untrusted input.

## Requirements trace

- R-108 (OBLIGATIONS; runtime M1).
- Existing: `contracts::TrustOrigin` (6 variants), operations
  recorder/replay.

## Acceptance Criteria

1. `derive_origin(inputs)` returns the MINIMUM trust of its inputs
   (one untrusted input contaminates the derivation; no laundering).
2. Derived records can be stamped (`DerivedTrust`) with source refs +
   inherited origin, serializable.
3. Ranking: UntrustedSource < UntrustedModel < SyntheticFixture <
   ObservedTool < Owner < ProtectedService (synthetic is weaker than
   observed tooling — it proves construction, not observation).
4. Twin-run byte-identical.

## Risks

- Vocabulary drift: GLOSSARY row FIRST (decision row before edit).
