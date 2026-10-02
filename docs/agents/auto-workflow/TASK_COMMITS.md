# Task-to-commit index

Every completed task of runs 15–36 (session 2026-10-02) has its own
local commit, pushed to `origin/main`. Tip CI: green. Tasks T-001 to
T-046 keep their own earlier commits (history below `1941b83`).

Rules commits:

| Task | Commit | Status |
|---|---|---|
| Standing rules: STE output + commit/push cadence (AGENTS.md) | `7d5d341` | pushed, CI green |
| Backfill audit trail, rotated reports, final allowlist/seal | `7f0abcb` | pushed, CI green |

Task commits (one per task unless noted):

| Task | Commit | Status |
|---|---|---|
| T-047 review authority service (R-034/035/036) | `d190c36` | done, pushed |
| T-048 external-act gate (R-045) | `8a87d4b` | done, pushed |
| T-049 provisional-target gate (R-076) | `1bc4516` | done, pushed |
| T-050 caching comparator review (R-083) | `879f048` | done, pushed |
| T-051 dossier evidence label (R-082) | `7e0b364` | done, pushed |
| T-052 benchmark admission gate (R-073) | `fc0e5a4` | done, pushed |
| T-053 mission completion gate (R-091) | `7b4c8bb` | done, pushed |
| T-054 classified bundle export (R-092) | `ebac2dd` | done, pushed (landed after T-064: backfill recovery) |
| T-055 held-out manifest gate (R-084) | `8573514` | done, pushed |
| T-056 citation parity (R-site retrofit) | `89f9f98` + fixup `070d680` | done, pushed |
| T-057 evidence kinds distinct (R-007) | `7a7fd0d` | done, pushed |
| T-058 inert retrieved directives (R-058) | `11b9343` | done, pushed |
| T-059 requirement-citation gate (ADR-027) | `879aa8b` | done, pushed |
| T-060 scratch trail completion | `902c313` | done, pushed |
| T-061 E2E mission driver — ticket 01 M2 chain, ticket 02 M3 chain, ticket 03 CLI + exits | `cc303f4` + fixup `055b766` | done, pushed (3 tickets share one commit) |
| T-062 champion-reuse demonstration (R-119) | `0339e2b` | done, pushed |
| T-063 crash reconciliation + wired canary (R-118) | `035476f` | done, pushed |
| T-064 triggers, cycles, monitor (R-115/R-118) | `7421dc0` | done, pushed |
| T-065 M5 + M6 exit assessment | `67898ff` | done, pushed |
| T-066 ticket 01 AT-citation report tool, ticket 02 amendment AT batch | `e662589` | done, pushed (2 tickets share one commit) |
| T-066 ticket 03 core AT batch (55 ATs) | — | pending; next task |

Notes:

- Pushed commits are never rewritten (no force-push rule). Ticket-level
  splits of T-061 and T-066 are not possible without rewriting; this
  file records the mapping instead.
- Forward cadence: one commit and one push per completed small task,
  then CI gates the next task.

## Pre-T-047 index (tasks T-001 to T-046)

All of this work was committed and pushed before the current rules
existed. Nothing was left in the working tree. Most tasks already have
their own commits. Two commits hold several tasks.

| Task | Commit(s) | Note |
|---|---|---|
| T-001 dev workspace | `25989ae` | one commit |
| T-002 typed contract records | `744cd86`, `e773eb2` | freshness gate included |
| T-003 fixture world + hidden evaluator | `8ef259e`, `d6437d0`, `219bdda`, `a83bd6a` | four small commits |
| T-004 security model pieces | `f2e2916`, `60a50e8`, `940d98a`, `40b1409`, `b12b6a3`, `1785230`, `16ad81a` | seven small commits (digests, approvals, trust, grants, receipts, sealed) |
| T-005 traceability validation | `43cebb3`, `9b27310`, `bdf870b` | three commits |
| T-006 event ledger | `47e4760` | one commit |
| T-007 capability grants + policy engine | `f54cfd2` (+ `4b85d73` CI fix) | one commit |
| T-008 to T-013 | `a4469a2` | SIX tasks in one commit — pushed before the per-task rule. A split needs a history rewrite. The rules forbid rewriting pushed history, so this stays as it is |
| T-014 pressure-point operators | `b8378dd` | one commit |
| T-015 mechanism registry | `d148480` | one commit |
| T-016 hypothesis compiler | `095dda4` | one commit |
| T-017 search orchestration | `dcb7ff2` | one commit |
| T-018 prior-art investigation | `e32343c` | one commit |
| T-019 experiment plan compiler | `0c8f751` | one commit |
| T-020 prototype worker | `0e03149` | one commit |
| T-021 method registry | `78ea9b8` | one commit |
| T-022 result interpreter | `b4f42cf` | one commit |
| T-023 dossier export | `f535bb5` | one commit |
| T-024 evidence lifecycle | `6799280` | one commit |
| T-025 workspace core | `29db77d` | one commit |
| T-026 eval suite | `9d345cc` | one commit |
| T-027 pilot campaigns | `13546a7` | one commit |
| T-028 release dossier packet | `12ce393` | one commit |
| T-029 advisory provider | `95bac04` | one commit |
| T-030 backend adapter | `f2a01a7` | one commit |
| T-031 acceleration evaluation | `d110bac` | one commit |
| T-032 domain packs | `08f2ddf` | one commit |
| T-033 + T-036 + T-037 | `3b37019` | three tasks folded by the user-ordered split (same-file interleave); the fold is recorded in the rotated decisions file |
| T-034 reproduction records | `0399b72` | one commit |
| T-035 amendment gates | `c445773` | one commit |
| T-038 revocation-race containment | `532ace3` | one commit |
| T-039 quarantine propagation | `c01123d` | one commit |
| T-040 yield accounting | `69ab041` | one commit |
| T-041 guardrail classes | `861c893` | one commit |
| T-042 import-closure oracle | `c72df72` | one commit |
| T-043 trust propagation | `c7da33c` | one commit |
| T-044 Pareto archive | `d34255e` | one commit |
| T-045 bounded task contracts | `af2417c` | one commit |
| T-046 domain verifier | `d26d10a` | one commit |

Chores and audits (not tasks): baseline `0f49466`, the ADR-024 gate
series, workflow audit records (`d63d553`, `ee7a081`, `d2a4d52`,
`dd3d8c1`, `62dd3a3`, `1941b83`), and the run-14 CI fixes
(`56d4267`, `9b52d52`, `f2af85b`, `1e88917`, `bee9cbf`). The full list
is `git log 0f49466..1941b83`.
