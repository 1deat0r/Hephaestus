# T-043 grill - R-108: trust-origin propagation gate

Goal: R-108 (amendment row T-007/T-010/T-011): "Preserve trust origin
through summaries, caches, graph edges and cross-session memory."

Existing: `TrustOrigin` is typed on mission provenance (contracts) and
used in operations recorder/replay. But derived surfaces — summaries,
caches, knowledge records, memory artifacts, graph edges — carry no
trust origin: a summary of an owner-trusted record or an untrusted one
is indistinguishable downstream.

## Q1 - What exactly is missing?
**A:** A propagation contract: any derived record (summary, cache
entry, knowledge/memory artifact, graph edge) must INHERIT the trust
origin of its source(s), with a downgrade rule — the derived trust is
the MINIMUM of its inputs. Untrusted sources propagate untrusted
forever; no laundering. (agent-default)

## Q2 - Seam?
**A:** New `trustprop` module: `derive_origin(inputs: &[TrustOrigin])
-> TrustOrigin` (min-of-inputs) + a `DerivedTrust` stamp helper for
records; integrated at the knowledge/memory seams. (agent-default)

## Q3 - Why minimum, not maximum?
**A:** Trust is a safety property, not an average: one untrusted input
contaminates the derivation. Max would launder untrusted evidence
through a trusted co-citation. (agent-default)

## Q4 - Ordering?
**A:** Contract enum order is authoritative: UntrustedSource <
UntrustedModel < ObservedTool < Owner < ProtectedService <
SyntheticFixture? NO — synthetic fixtures are NOT trust; ordering by
data reliability: untrusted_source < untrusted_model < synthetic_fixture
< observed_tool < owner < protected_service. (agent-default)

## Q5 - Vocabulary?
**A:** GLOSSARY row: Trust propagation. Decision row before edit.
(agent-default)

## Q6 - TDD seams?
**A:** Red-first: min-of-inputs + downgrade-on-derive. (agent-default)
