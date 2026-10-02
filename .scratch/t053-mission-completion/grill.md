# T-053 grill - R-091: complete a bounded mission with evidence rather than agent agreement

Goal: R-091/AT-091 (Mission completion and Promotion service; contract
M0, runtime M4, MASTER_SPEC §31) — negative: "Return unanimous
approval but no usable evidence." Required: "The mission cannot claim
a validated invention candidate."

Existing surfaces (facts):
- mission module: `compile`, `revise`, intake records — ZERO
  completion surface (grep `complete|completion`: none);
  mission_review.rs is T-012 fix-cycle compile tests, not completion.
- Pattern precedent: T-047's review gate — agreement input is
  structurally never consulted for eligibility (R-034's shape);
  here agreement must likewise never substitute evidence (R-091).
- "Validated invention candidate" claims live downstream (promotion);
  this gate refuses to produce the completion record that such claims
  would rest on.

## Q1 - What is the seam?
**A:** `mission::complete_mission(mission_id, &AgentAgreement,
&[EvidenceRef])` — the completion gate. `AgentAgreement { reviewers }`
(opinions), `EvidenceRef { evidence_id, version }` (version-bound
evidence, §31 receipts lineage). (agent-default)

## Q2 - Refusals?
**A:** Empty evidence slice → `CompletionError::
AgreementWithoutEvidence` (AT-091 negative verbatim — the agreement
param is structurally unread on every path, exactly as review votes
are in R-034). Any ref with an empty id or version →
`UnusableEvidence { evidence_id }` (usable = well-formed,
version-bound). Success → `MissionCompletion { mission_id, evidence }`
carrying the accepted refs (the record a validated-candidate claim
would need — without it, no claim). (agent-default)

## Q3 - Bounded/mission-side checks?
**A:** Scope cut: budget/tripwire/completion-criteria checks belong to
compile-time tripwires and the promotion service (enforcement names
BOTH services; promotion half composes later). This ticket is the
evidence-vs-agreement gate only — matching AT-091's negative exactly.
(agent-default)

## Q4 - Files/gates?
**A:** mission/record.rs (types), mission/mod.rs (gate + re-exports),
NEW tests/mission_completion.rs (clear seam name; +1 allowlist) +
.scratch×3 → allowlist +4; reseal; glossary rows: Agent agreement,
Completion evidence. Citations R-091/AT-091. Red-first at
complete_mission. No commit (rule 5). (agent-default)
