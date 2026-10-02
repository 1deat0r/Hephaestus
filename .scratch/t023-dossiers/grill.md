# T-023 grill - raw-data capture, cost receipts, clean-environment reproduction, dossiers

Goal: raw-data capture, complete cost receipts, clean-environment
reproduction, exportable dossiers including failed attempts and deviations
(IMPLEMENTATION_PLAN:74, MASTER_SPEC §15:299-300, R-044, R-003, R-080).

## Q1 - Module placement + seam?
**A:** New `crates/hephaestus/src/dossier/` module. Seam:
`capture(run) -> RunRecord`, `reproduce(record, env) ->
ReproductionOutcome`, `export(mission) -> Dossier` (+ `to_json`).
(agent-default)

## Q2 - What does a RunRecord capture (R-023-adjacent raw data)?
**A:** measurements (raw, unsummarized), environment declaration (OS/tool
versions pinned - R-065 continuity), deviations from the frozen plan
(retained, never silently dropped), cost receipts (R-103: acquisition,
generation, failed/invalid/blocked runs, evaluator+reproduction work,
human intervention - quantities recorded, NO invented conversion price).
(agent-default)

## Q3 - Clean-environment reproduction (§15:298)?
**A:** `reproduce` compares the record's environment declaration to the
reproduction environment: mismatch on pinned versions -> `EnvironmentMismatch`
(a typed outcome, not a silent pass); same environment + same artifact
digests -> `Reproduced`; measurement disagreement beyond the declared
precision -> `Disagrees`. All three are first-class outcomes (R-080).
(agent-default)

## Q4 - Dossier contents (§15:299)?
**A:** problem+beneficiary, prior-art reference (link to T-018 report
inputs), mechanism explanation, complete hypothesis versions, predeclared
tests (frozen plan), artifacts+environment, raw data, analysis, results
(typed results from T-022), counterevidence, failure history, uncertainty,
scope limits, cost ledger, reproducibility commands, unresolved risks, next
justified action. (agent-default)

## Q5 - Negative results (R-003, §15:300, R-044)?
**A:** DossierKind::Negative is first-class. §15:300 requires
distinguishing: invalid test / failed implementation / unsupported
mechanism / uncompetitive engineering realization / resource-limited
investigation - encoded as `NegativeResultKind` enum. Both histories
(failed attempts + valid negative test) present with SEPARATE conclusions
(R-044 negative case: "Both histories and reproduction material are
present with separate conclusions"). (agent-default)

## Q6 - Failed attempts + deviations?
**A:** `failure_history: Vec<FailureEntry>` (what failed, where, when,
digest of the failed artifact) and `deviations: Vec<Deviation>` (frozen-plan
reference, what changed, reason) - export REQUIRES both fields present
(empty allowed, None refused). (agent-default)

## Q7 - Reproducibility commands (§15:299)?
**A:** `repro_commands: Vec<String>` on the dossier - refused empty at
export (a result lacking reproduction material cannot pass a
reproducibility gate, §14:282 continuity). (agent-default)

## Q8 - Export gate?
**A:** `export` refuses: missing raw data, missing failure history field,
missing deviation field, empty repro commands, unpriced conversion of human
time (cost entries must be quantities; a "price" field on human time is
refused - campaign: "record both provider charges and resource/human-time
quantities rather than inventing a conversion price"). (agent-default)

## Q9 - Integration?
**A:** Dossier carries the TypedResult (T-022) verbatim + hypothesis
version chain (R-097 lineage): mission -> opportunity -> mechanism ->
hypothesis -> plan -> result IDs recorded as a lineage vector.
(agent-default)

## Q10 - Vocabulary?
**A:** GLOSSARY rows FIRST: Dossier, Run record, Reproduction outcome.
Decision row before edit. (agent-default)

## Q11 - TDD seams?
**A:** Red-first per ticket at `capture`/`reproduce` (01) and `export`
(02). (agent-default)
