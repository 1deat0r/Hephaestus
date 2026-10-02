# Repository instructions for implementation agents

The authoritative product specification is `MASTER_SPEC.md`; stable obligations are in `requirements.json`. Architectural departures require an explicit decision record. Preserve user work and follow existing repository instructions when integrating.

Do not weaken tests, protected evaluators, source provenance, authorization, or scientific interpretation to make an experiment look successful. Generated claims remain proposals until independently qualified. Never manufacture citations, runtime receipts, experiments, benchmark scores, independent reviews, or measured performance.

Keep all model, retrieval, execution, graph, and decision providers behind capability contracts. Jev is a probabilistic advisory provider, not deterministic scientific logic. Tachyon is optional until its actual interface passes contract tests.

No seed hypothesis is required from the user. No global-novelty claim follows from no search hits. No statistical mechanism conclusion follows solely from missing a product target. No inconclusive observation becomes confirmation. No new artifact inherits old approval or confirmation without version checks.

Every implementation change must identify requirement IDs, run relevant checks, preserve evidence, and report limitations. Use isolated artifacts and deterministic merge/verification for parallel work. Do not spawn tiny workers when batching is more efficient.

The package contains specification/reference checks, not a working invention application. Keep this distinction visible until implementation changes it with evidence.

## Output style (standing user rule, 2026-10-02)

Write all agent output for this project in concise Simplified
Technical English (ASD-STE100). Rules: one idea per sentence; 20 words
or fewer per sentence; active voice; present tense; one meaning per
word; no optional words; no idioms or metaphors; every sentence has a
clear subject; short paragraphs; lists when order matters. Keep all
numbers, paths, receipts, and caveats exact. Simplicity must not remove
facts.

## Git cadence (standing user rule, 2026-10-02)

Work task by task. The commit unit is the SMALL TASK: after its
`Verify:` passes, commit it locally with the message
`<task-id>.<n>: <imperative summary>` (subject 72 chars or fewer),
flip its `**Status:**` in that same commit, and push to GitHub before
the next small task starts. Local hooks and CI are the gates. A red
gate blocks the next small task until the fix lands. Never force-push.
Never rewrite pushed history.

## Ticket status (standing user rule, 2026-10-02)

Flip the ticket `**Status:**` in the same commit that lands its
work. Every open ticket declares a `Verify:` command and a `Covers:`
list of spec acceptance-criterion numbers. Keep open tickets small:
at most 16 scope-ID occurrences per ticket. A feature with any open
ticket must cover every spec AC through its tickets' Covers lists.
Per-level child caps live in Task decomposition below. `make
ticket-status` runs in `make ci` and checks all of this. The
scheduled sweep (`ticket-status-sweep`) fails any open ticket whose
Verify is green. A red gate blocks the next task.

## Task decomposition (standing user rule, 2026-10-02)

One hierarchy, four levels. A ticket file is a TASK.
- TASK: `**Status:**`, `**Verify:**`, `**Covers:**` at file top.
- Small task: one commit unit. Child of a TASK. Cap: 8 per TASK.
  Own `**Status:**` and `**Verify:**` line and own checkbox.
- Micro task: one red-to-green cycle. Child of a small task.
  Cap: 6 per small task. Own `**Verify:**` line and own checkbox.
- Nano task: one file, one command, or one measurement. Child of a
  micro task. Cap: 4 per micro task. Indented `- [ ]` bullet.
Open tickets carry the nesting under `**Small tasks:**` and
`**Micro-tasks:**`. A unit with no children ends its branch with the
marker `atomic` — never invent padding children. Roll-up: all
children checked (or `atomic`) plus its Verify green makes the parent
done when its commit lands. `make ticket-status` checks caps and
structure.
