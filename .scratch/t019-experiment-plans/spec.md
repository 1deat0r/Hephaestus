# T-019 spec - Experiment Compiler

Status: ready-for-agent

## Problem Statement

Hypotheses (T-016) have no path to experiments. MASTER_SPEC §13 requires an
Experiment Compiler that compiles the cheapest authorized experiment able to
resolve the relevant uncertainty, binds every plan field immutably
(hypothesis version, claim IDs, conditions, comparator, artifact, procedure,
units, primary endpoints, guardrails, sampling unit, analysis spec, stopping
rule, evaluator hash, resource limit, authorization scope), builds a
discrimination matrix against named alternatives, and returns precise
blockers instead of imagined outcomes. R-040 (predeclared analysis + valid
stopping rule) and R-096 (complete endpoint/control/unit/guardrail coverage)
gate it.

## Requirements trace

- R-040 / AT-040: analysis spec + stopping rule required at compile time.
- R-096 / AT-096: validate denies qualification until endpoint, control,
  sampling unit, guardrail coverage complete - named denials.
- R-094/R-095: evaluator hash from protected context; digest mismatch
  rejected.
- R-097: plan binds frozen hypothesis version.
- §13:255: all fields bound, registered immutably before confirmatory
  results.
- §13:257: discrimination matrix rows (mechanism, alternative,
  artifact/null); `cannot_discriminate` flag; no invented probabilities.
- §13:262: blocker enum, precise, never a substituted imagined outcome.

## Acceptance Criteria

1. `compile` produces a plan binding ALL §13:255 fields; missing analysis
   spec or stopping rule fails compilation (R-040).
2. `validate` returns named denials for missing endpoint/control/unit/
   guardrail; complete plan passes (R-096).
3. Evaluator digest carried; validate rejects digest mismatch (R-094/095).
4. Discrimination matrix rows cover mechanism/alternative/artifact;
   indistinguishable predictions flagged `cannot_discriminate` (§13:257).
5. Blockers are the precise enum; compile returns blockers, never imagined
   results (§13:262).
6. Plan binds frozen hypothesis version string (R-097).
7. Twin-run byte-identical plans.

## Risks

- Vocabulary drift: GLOSSARY rows FIRST (decision row before edit).
- Over-engineering: statistical computation stays out (M3 scope, T-021);
  this module is plan structure + gates only.
