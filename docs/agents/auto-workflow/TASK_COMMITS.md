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
