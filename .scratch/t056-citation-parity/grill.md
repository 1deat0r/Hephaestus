# T-056 grill - citation parity for audited false-positive obligations

Goal: the coverage scan (the derivation tool) reports 25 uncited
R-ids: 24 formally-audited false positives (implementation exists,
citation missing) + R-054 (deferred). Retofit honest citations at the
exact sites that already verify each obligation, so the scan's signal
becomes true — without fabricating a single claim.

## Q1 - Which obligations get code cites, and where?
**A:** The 15 code-verified ones, each at its exercising site (map
below), R-IDs only — AT claims added only where the test mirrors the
AT scenario, and none of these do verbatim, so ATs stay uncited:
R-004 (sandbox_deny header), R-006 (domain_packs header),
R-008 (hypothesis_compiler header), R-028+R-030 (prior_art header) +
R-030 (priorart/mod doc), R-038+R-039 (experiment_compiler header),
R-041 (method_registry header — bonferroni family split fn exists),
R-042 (result_interpreter header — noninferiority fn), R-043
(prototype_worker header — assemble/invariants), R-047
(evidence_lifecycle header — invalidate traversal), R-048
(advisory/record ModelJudgment doc — verbatim match), R-053
(advisory_provider header — calibration fn), R-064 (backend_adapter
header — LocalProcess), R-093 (release_packet header —
scope_label EXPERIMENTAL default). (agent-default)

## Q2 - Which do NOT get code cites?
**A:** (1) Doc-process FPs R-079/081/085-089: their enforcement lives
in docs/process (IMPLEMENTATION_PLAN order, requirements register
fields, RUNTIME_DECISIONS ADRs, MASTER_SPEC honesty clauses) —
citing them in code would FABRICATE the signal; they stay documented
in the run-17..23 decisions ledger as audited FPs. (2) R-007 and
R-058 re-audited this run: NOT false positives at all — R-007's
distinctness exists only as the schema/generated enum with no
R-named test; R-058 (retrieved instructions/generated code untrusted)
has zero code hits in knowledge/memory/trustprop — both RECLASSIFIED
open and become future goals, excluded from the retrofit.
(3) R-054 stays deferred (no cache feature). (agent-default)

## Q3 - Churn/gates?
**A:** Comment-only edits (14 files) — no behavior, no tests change;
no allowlist/seal impact (all files already allowlisted); scan must
still show only the non-code-cited set afterward (verification
criterion). (agent-default)

## Q4 - Scope?
**A:** No schema/generator edits (frozen contract envelope — out of
bounds for a citation retrofit), no new tests (that is R-007/R-058's
future ticket), no commit (rule 5). (agent-default)
