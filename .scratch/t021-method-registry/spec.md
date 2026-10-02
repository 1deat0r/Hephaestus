# T-021 spec - fixed-sample method family + protected method registry

Status: ready-for-agent

## Problem Statement

The campaign's M3 slice needs the fixed-sample method family implemented as
typed, registered, preregistered structures: paired repository-level
comparisons, bounded-outcome concentration intervals, binary yield
differences, fixed Bonferroni error allocation, explicit missingness
policies, estimands, and a preregistration gate that blocks n=null,
unqualified methods, and unresolved oracles from entering confirmation.
No fabricated sample sizes; sequential methods out of scope.

## Requirements trace

- R-100: typed experiment families, selection lineage, method
  qualification, explicit error allocation.
- R-103: preregistered sampling, tuned baselines, denominators, all
  failure/human/resource costs recorded (denominator fields on the
  estimand; cost fields on the registration).
- Campaign M3 paragraph: bounded-outcome intervals, fixed Bonferroni
  family, clipping frozen before confirmation, missingness rules,
  preregistration gate, zero-failure planning bound.

## Acceptance Criteria

1. `register_method` binds name/version, estimand, assumptions, error
   allocation, missingness policy, clipping policy, identity digest;
   duplicates rejected; entries immutable (new version = new entry).
2. `bonferroni_alpha(k, total)` splits equally, sums to <= total.
3. `zero_failure_bound(299, 0.05) <= 0.01` (campaign's own example,
   planning-only label in docs).
4. `preregister` refuses n=None / unqualified method / unresolved oracle /
   missing calculation-reference, alpha allocation, reference distribution,
   power/precision target, or sensitivity assumptions - each a named error.
5. Missingness policy enum with strictest-default semantics.
6. Twin-run byte-identical registry serialization.

## Risks

- Fabricating a sample size: explicitly forbidden - the package supplies
  none and none is invented here.
- Over-engineering: no live-data statistics in this module.
