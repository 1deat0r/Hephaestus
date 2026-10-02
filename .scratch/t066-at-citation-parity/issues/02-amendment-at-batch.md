# 02: Triage batch A — amendment ATs (AT-094..AT-119)

**What to build:** audit each AT-094..AT-119 negative case against
existing tests (run-24 method: read BOTH sides); cite at true mirrors
(comment-only); record every non-mirror as a goal candidate in this
ticket + a decisions row; re-scan shows the batch fully dispositioned.

**Blocked by:** 01 (the report is the worklist).

**Status:** done
**Covers:** 2

- [x] All AT-094..119 dispositioned: cited (verified mirror) or
      recorded goal candidate — none silently skipped
- [x] Re-scan: batch absent from the silent-gap set; gates green

## Comments

Grill: `.scratch/t066-at-citation-parity/grill.md`.

**Batch A dispositions (2026-10-02):** 22 ATs cited at verified mirrors
(read both sides — experiment_compiler AT-094/096, self_improvement
AT-095/117, evidence_lifecycle AT-098, method_registry AT-100,
sealed_deny AT-101, import_closure AT-102, pilot AT-103/104,
release_packet AT-105, corpus_search AT-106, corpus_ingest AT-107,
trust_propagation AT-108, memory_quarantine AT-109, containment_races
AT-111, prior_art AT-112, traceability AT-113, migrate AT-114,
trigger_canary AT-115, learning_reuse AT-116, crash_reconciliation
AT-118, champion_reuse AT-119).

**Goal candidates (NOT mirrored — never silently skipped):**
- **AT-097**: cross-record substitution into a dossier with otherwise
  valid results has no denying test (eval_suite lineage checks and
  semantic cross-record validation are neighbouring properties, not
  the mirror) → future goal.
- **AT-106 neg-facets**: hard-synonym / cross-domain-equivalent /
  citation-chain retrieval fixtures are untested (the statement's
  counterevidence+near-match facets ARE cited at prior_art/corpus
  search) → facet-level goal candidate.

Re-scan: 55 remaining (core batch = ticket 03 + AT-097).
