# T-028 grill - release dossier, scope label, reproducibility report

Goal: release dossier with security findings, scope label, reproducibility
report, cost/latency measurements, unresolved research questions; no
critical engineering failure open in the declared scope (IMPLEMENTATION_PLAN:94,
R-077, R-078, R-080).

## Q1 - What does "release" mean here?
**A:** The scoped release packet: scope label (EXPERIMENTAL vs
QUALIFIED_FOR_DECLARED_SCOPE - default EXPERIMENTAL unless evidence
supports more, campaign release dispositions), security findings list,
reproducibility report, cost/latency measurements, unresolved research
questions. The gate: critical unresolved failures in the declared scope
BLOCK the release (R-077). (agent-default)

## Q2 - Module placement + seam?
**A:** New `crates/hephaestus/src/release/` module. Seam:
`assemble_release(inputs) -> Result<ReleasePacket, ReleaseBlock>` +
`scope_label(evidence) -> ScopeLabel`. (agent-default)

## Q3 - Scope labels (campaign release dispositions)?
**A:** EXPERIMENTAL (default), QUALIFIED_FOR_DECLARED_SCOPE (requires the
frozen campaign + protected oracle/method qualification + error and
correctness guardrails + independent reproduction + scope-specific security
gates). "Faster/better" claims require measured baseline comparison +
uncertainty - encoded as claim fields that MUST carry their measurement
references or the packet refuses. (agent-default)

## Q4 - Critical-failure gate (R-077)?
**A:** Security findings and engineering failures carry severity +
resolved flag; a CRITICAL unresolved finding in the declared scope blocks
`assemble_release` (named block). Out-of-scope criticals are recorded but
do not block (scope honesty). (agent-default)

## Q5 - Finite-suite uncertainty (R-078)?
**A:** The packet's reliability statement is a struct: measured-on (suite
id, n, interval) - a claim of universal reliability is unrepresentable
(no such field); uncertainty MUST be present or assembly refuses.
(agent-default)

## Q6 - Outcome classes (R-080)?
**A:** The packet records end-to-end outcome counts for positive, negative,
inconclusive, invalid, blocked - all five present (possibly zero), no class
omitted. (agent-default)

## Q7 - Reproducibility report?
**A:** Environment pins + repro commands + per-artifact digest verification
results (link to T-023 semantics). (agent-default)

## Q8 - Cost/latency measurements?
**A:** Quantities only (R-103 continuity): per-kind quantity+unit; no
invented conversion. (agent-default)

## Q9 - Unresolved research questions?
**A:** Required field (may be non-empty - honesty); assembled packet
REFUSES an empty-list claim of "no unresolved questions" only if the
evidence says otherwise... actually simpler: the field must be PRESENT
(Vec, possibly empty but explicitly recorded). (agent-default)

## Q10 - Vocabulary?
**A:** GLOSSARY rows FIRST: Release packet, Scope label, Security finding.
Decision row before edit. (agent-default)

## Q11 - TDD seams?
**A:** Red-first per ticket: findings+labels (01), assembly+gate (02).
(agent-default)
