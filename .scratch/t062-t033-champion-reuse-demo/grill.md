# T-062 grill - T-033 champion-reuse demonstration (the M3 NOT-RUN clause)

Goal: discharge `docs/M2_M3_M4_EXIT_ASSESSMENT.md` clause 3.5 — R-119's
end-to-end controlled loop: broad mission reveals a non-seeded failure →
service proposes a challenger → protected fresh evaluation qualifies →
persisted champion → **a later mission uses it** → restart preserves
state → injected regression triggers rollback → false/confounded/
overbudget/permission-expanding/inconclusive challengers stay
undeployed.

## Facts (probes)

- selfimprove has: `evaluate_candidate`, `promote`, `rollback`
  (returns RollbackReceipt, does NOT append), `ImprovementLedger`
  append/entries/to_json/from_json, learning_gate. **No proposal fn,
  no champion lookup fn, no consumer anywhere** (tests only round-trip
  from_json at self_improvement.rs:212).
- LedgerEntry carries target + outcome strings ("deployed"/"rejected"
  precedent) + challenger/incumbent ids — enough to resolve state, but
  NO value field: the tunable's value lives in the challenger payload
  whose digest the candidate pins (content-addressing, same pattern as
  promote's verified-artifacts fixture).
- ImprovementCandidate requires observation ids, fresh partition,
  budget_from, rollout/rollback digests, manifest — R-117 shape is
  already enforced by promote().
- The M2 slice's tunable exists today: `apply_all(..., max_per_operator
  = 8)` (hardcoded in e2e_m2). That is the honest championable target:
  `discovery.generation_bound`.
- missionrun chain (M3) composes after the M2 slice in the demo's
  mission runner.

## Q1 - What does the "service propose" step look like in code?
**A:** `selfimprove::propose_from_observations(...)` — a bounded,
deterministic constructor: requires non-empty supporting_observation_ids
(the non-seeded evidence), a target, incumbent/challenger content
digests, fresh partition, budget_from (≠ challenger_id — R-115 no
self-budgeting, checked here as well as in evaluate), guardrails,
manifest. No model, no guessing: trigger (mission completion) →
candidate with evidence attached; proposal with NO observations is
refused (typed error). (agent-default)

## Q2 - Champion resolution?
**A:** `selfimprove::active_champion(&ledger, target) ->
Option<Champion { id, digest }>`: walk entries for the target in
order — last `deployed` → challenger; a later `rolled_back` entry →
that deployment's incumbent; nothing → None (caller uses its default).
LedgerEntry gains NO fields (outcome strings "deployed"/"rolled_back"
follow the existing convention; the new outcome string is documented in
the fn + test). (agent-default)

## Q3 - How does a later mission USE it?
**A:** The demo's mission runner (test-local, composing the proven
slices): resolve active champion → map id→payload fixture (digest
verified against the candidate's challenger_digest before use —
content-addressed receipt) → run the M2 slice with
`max_per_operator = bound` → run missionrun M3 chain → emit a receipt
JSON {bound_used, source: "champion"|"default", mechanisms_found,
typed verdict, dossier receipt}. Mission A uses default8; mission B
(bound16 champion) shows a strictly wider sweep receipt; post-rollback
missions return to8. (agent-default)

## Q4 - Persistence + restart?
**A:** ledger.to_json → written to a test temp dir file → read back →
from_json → resolution identical; receipts assert byte-equal state
(R-116 restart). (agent-default)

## Q5 - Rollback + undeployed challengers?
**A:** injected regression: `rollback(&deployment1, base8_digest,
"injected latency-guardrail regression observed in mission B",
observations)` → receipt retained → resolution flips to incumbent →
mission C/D use8. Undeployed set: in the SAME test file, four
`promote()` attempts with Unsupported/confounded/overbudget/
permission-expanding assessments each return a named rejection (some
overlap existing suite coverage — R-119 requires them IN the
demonstration, so they are re-asserted here cheaply). (agent-default)

## Q6 - Files/scope?
**A:** src: `selfimprove` gains `propose_from_observations` +
`active_champion` (+ typed errors); test: `tests/champion_reuse.rs`
(one file, phased: A→propose→assess→promote→persist→B→restart→
regression→rollback→C/D + undeployed×4). allowlist +2 (test + scratch).
No live canary/drift services (still typed records — the exit doc
keeps that limitation line; R-119's "injected regression" is the
in-bounds demonstration). No commit (rule 5). (agent-default)
