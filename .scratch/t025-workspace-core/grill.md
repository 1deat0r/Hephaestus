# T-025 grill - conversational TUI + resumable authenticated gateway

Goal: conversational TUI with truthful progress, hypothesis comparison,
evidence drill-down, intervention controls, and a resumable authenticated
gateway (IMPLEMENTATION_PLAN:88, MASTER_SPEC §2:42, R-067, R-068, R-069).

## Q1 - Scope reality check?
**A:** A full interactive TUI is a UI-surface product. The M4-verifiable
core that requirements actually pin: (a) a typed UI state model grounded
in persisted state (R-067 - progress/outcomes READ from state, never
narrated), (b) hypothesis comparison + evidence drill-down queries over
existing modules, (c) intervention controls as typed commands with
policy+authentication applied identically to gateway and local paths
(R-069), (d) pause/resume/cancel/steering/event-cursor recovery (R-068) as
a resumable authenticated gateway protocol. Rendering (ratatui etc.) is
presentation, later. (agent-default)

## Q2 - Module placement + seam?
**A:** New `crates/hephaestus/src/workspace/` module (the interactive
research workspace core). Seam: `Workspace::open(state) -> Workspace`,
`progress() -> ProgressView`, `compare(h1, h2) -> ComparisonView`,
`drill_down(evidence_id) -> EvidenceView`, `command(cmd, auth) ->
Result<Ack, ControlError>`, `gateway_event(event) -> Result<Ack>`.
(agent-default)

## Q3 - Truthful progress (R-067, §2:42)?
**A:** ProgressView fields are read from persisted records: entity counts
per lifecycle state, last event id, running/paused flag. NO fabricated
percentages, NO "agent working" narrative - unknown is represented as
unknown. (agent-default)

## Q4 - Hypothesis comparison?
**A:** compare(h1, h2) returns per-field diffs (version, state, claim ids,
discriminator presence) + shared-evidence overlap - read-only over
genesis/hypothesis records. (agent-default)

## Q5 - Evidence drill-down?
**A:** drill_down(id) returns the record + its lineage chain (R-097 ids)
+ stale flags from the lifecycle graph - grounded in persisted state.
(agent-default)

## Q6 - Intervention controls + identical policy (R-068, R-069)?
**A:** Typed commands: Pause, Resume, Cancel(mission), Steer(mission,
directive). EVERY command carries an AuthToken; the SAME
`authorize_control(token, command, policy)` function gates both the local
and gateway paths - one code path, no gateway bypass. (agent-default)

## Q7 - Event-cursor recovery (R-068)?
**A:** The gateway records an event cursor (last processed event id);
`gateway_event` with a stale/duplicate cursor returns the replay
recommendation; resume = re-open workspace from persisted state + cursor.
(agent-default)

## Q8 - What is auth here?
**A:** Minimal: a token is a digest string checked against the policy's
authorized digests (existing policy module continuity). NOT a crypto
system - the contract layer. (agent-default)

## Q9 - Vocabulary?
**A:** GLOSSARY rows FIRST: Workspace, Event cursor, Intervention control.
Decision row before edit. (agent-default)

## Q10 - TDD seams?
**A:** Red-first per ticket: progress+views (01), controls+gateway (02).
(agent-default)
