# T-053 spec - R-091: complete a bounded mission with evidence rather than agent agreement

Status: ready-for-agent

## Problem Statement

R-091 (M4): a mission completes WITH EVIDENCE, not with agent
agreement — unanimous approval and no usable evidence must not yield
a completion that could claim a validated invention candidate
(AT-091). The mission module compiles and revises but has no
completion surface at all (grep-verified), so nothing currently
refuses agreement-only completion.

## Requirements trace

- R-091/AT-091 (OBLIGATIONS; enforcement: Mission completion and
  Promotion service; contract M0, runtime M4). Source: MASTER_SPEC
  §31.
- Continuity: review-service precedent (T-047) — agreement inputs are
  structurally never consulted for eligibility; compile-time
  tripwires (T-012) unchanged.

## Design

Extend `mission`:

- `AgentAgreement { reviewers: Vec<String> }` — opinions, recorded
  but never consulted by the gate.
- `EvidenceRef { evidence_id, version }` — version-bound evidence.
- `CompletionError::{ AgreementWithoutEvidence, UnusableEvidence {
  evidence_id } }`.
- `complete_mission(mission_id, &AgentAgreement, &[EvidenceRef]) ->
  Result<MissionCompletion, CompletionError>`:
  - empty evidence → `AgreementWithoutEvidence` (agreement unread);
  - empty id or version on any ref → `UnusableEvidence`;
  - else `MissionCompletion { mission_id, evidence }`.

## Acceptance Criteria

1. **AT-091 negative**: unanimous agreement + empty evidence →
   `AgreementWithoutEvidence`; no completion record exists.
2. Evidence refs with empty id/version → `UnusableEvidence` naming
   the ref.
3. Well-formed evidence → `MissionCompletion` carrying every ref
   (version-bound), reviewers list irrelevant to the outcome.
4. Glossary rows: Agent agreement, Completion evidence.
5. Gates green (`make ci` + `make doc-check`), allowlist +4 + reseal,
   tests cite R-091/AT-091.

## Out of Scope

- Budget/tripwire/completion-criteria checks (compile-time + promotion
  service composition).
- Any validated-candidate claim machinery (promotion half).
- Committing or pushing (rule 5).

## Further Notes

Grill: `.scratch/t053-mission-completion/grill.md`.
