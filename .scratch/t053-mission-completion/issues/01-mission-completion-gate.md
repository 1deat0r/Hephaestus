# 01: Mission completion gate — evidence or refusal, agreement never counts

**What to build:** The R-091 completion seam: unanimous agent approval
without usable evidence cannot produce a mission completion record;
malformed evidence refs are refused by name; well-formed version-bound
evidence completes the mission carrying every ref — so no validated
candidate claim can rest on agreement alone.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] AT-091: unanimous `AgentAgreement` + empty evidence →
      `AgreementWithoutEvidence`, no completion
- [x] Empty evidence_id/version → `UnusableEvidence` naming it
- [x] Well-formed refs → `MissionCompletion` with every ref carried;
      reviewer list does not affect the outcome
- [x] Glossary rows (Agent agreement, Completion evidence); allowlist
      +4 + reseal; gates green; tests cite R-091/AT-091

## Comments

Grill: `.scratch/t053-mission-completion/grill.md`.
Spec: `.scratch/t053-mission-completion/spec.md`.
