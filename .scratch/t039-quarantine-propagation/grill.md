# T-039 grill - R-109: quarantine propagation to dependent artifacts/labels

Goal: R-109 negative case (OBLIGATIONS:1745-1748): "Attempt a
source-induced policy-memory write then quarantine its origin" ->
"Unauthorized write is denied; dependent current artifacts/labels
invalidate while immutable audit history survives."

Existing: knowledge service excludes quarantined SOURCES from retrieval.
Missing: PROPAGATION — when a source is quarantined, artifacts/labels
derived from it must invalidate; behavioral-memory writes stay
policy-gated.

## Q1 - What is missing exactly?
**A:** Two pieces: (a) derived-artifact dependency graph — artifact
records name their source; (b) `propagate_quarantine(source_id,
artifacts)` which invalidates dependent CURRENT artifacts/labels while
audit history survives (append-only invalidation, same pattern as
T-036). (agent-default)

## Q2 - Module placement + seam?
**A:** New `crates/hephaestus/src/memory/` module (behavioral-memory +
labels). Seam: `MemoryArtifact` (source-derived, current flag), `label`,
`propagate_quarantine`, `authorize_write` (policy-gated — unauthorized
source-induced writes denied). (agent-default)

## Q3 - Unauthorized write denial (negative case)?
**A:** `authorize_write(writer_grant, target)` — a write to the policy
memory without a scoped grant covering it is denied (reuse
policy::grant capabilities_subset semantics conceptually; typed refusal
named). (agent-default)

## Q4 - Propagation semantics?
**A:** Quarantining a source marks every CURRENT artifact derived from
it not-current (invalidated); labels referencing those artifacts
invalidated too. History: the artifact records remain (append-only
invalidation records, no deletion) — audit survives. (agent-default)

## Q5 - Vocabulary?
**A:** GLOSSARY rows FIRST: Behavioral memory, Quarantine propagation.
Decision row before edit. (agent-default)

## Q6 - TDD seams?
**A:** Red-first: write authorization (01), propagation (02)? Or single
ticket? Single cohesive module, one acceptance set -> 1 ticket.
(agent-default)
