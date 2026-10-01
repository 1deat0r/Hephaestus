# T-042 grill - R-102: import-closure oracle disagreement handling

Goal: R-102 (amendment row T-003/T-026-T-028): "Define the first domain
and independently qualify its bounded context-correctness oracle."
Negative case: "Exercise import cycles, aliases, relative imports,
deletion, rename, dynamic imports and disagreement between oracle
implementations." Required outcome: "The declared closure matches the
qualified oracle; unsupported resolution falls back conservatively or
blocks without omission."

## Q1 - What is the closure oracle domain?
**A:** The FIRST domain is the workspace's own Rust import graph: given
a declared module closure, an independently qualified oracle checks the
declared set matches actual imports. Existing `importclosure` work may
exist — check first. (agent-default)

## Q2 - What must the oracle handle (negative case)?
**A:** Import cycles (detect, not loop), aliases (`use x as y`),
relative imports (`super::`/`crate::`), deletion/renames (stale declared
entries flagged), dynamic imports (conservative fallback), and
DISAGREEMENT between two oracle implementations (declare mismatch -> the
declared closure is NOT accepted; conservative block). (agent-default)

## Q3 - What exists already?
**A:** To verify: grep for importclosure modules. If a closure module
exists with resolution, this goal adds the DISAGREEMENT + conservative
fallback gates. (agent-default)

## Q4 - Conservative fallback semantics?
**A:** Unsupported resolution -> the closure check BLOCKS (returns a
named unresolved list) rather than silently omitting the import.
(agent-default)

## Q5 - Vocabulary?
**A:** GLOSSARY rows FIRST: Import closure, Oracle disagreement.
Decision row before edit. (agent-default)

## Q6 - TDD seams?
**A:** Red-first per gap found. (agent-default)
