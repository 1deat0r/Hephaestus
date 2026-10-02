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

Work task by task. Commit every completed small task locally, then
push it to GitHub, before the next task starts. Local hooks and CI are
the gates. A red gate blocks the next task until the fix lands.
Never force-push. Never rewrite pushed history.

## Ticket status (standing user rule, 2026-10-02)

Flip the ticket Status in the same commit that lands its work. Every
open ticket declares a `Verify:` command and a `Covers:` list of spec
acceptance-criterion numbers. Keep open tickets small: at most 16
scope IDs and at most 8 unchecked boxes per ticket. A feature with any
open ticket must cover every spec AC through its tickets' Covers
lists. `make ticket-status` runs in `make ci` and checks all of this.
The scheduled sweep (`ticket-status-sweep`) fails any open ticket
whose Verify is green. A red gate blocks the next task.

## Task decomposition (standing user rule, 2026-10-02)

Every small task nests micro tasks. Every micro task nests nano
tasks.
- Small task: one ticket, one unit of value, one push. The size caps
  apply (16 scope IDs, 8 unchecked boxes).
- Micro task: one seam or one red-to-green cycle. It ends in one
  commit and one push.
- Nano task: one file, one command, or one measurement. Minutes of
  work; it never ships alone.
Open tickets declare the nesting under `**Micro-tasks:**`, with nano
steps as indented bullets. `make ticket-status` checks the structure.
