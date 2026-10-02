# T-035 grill - amendment row: domain validity/migration gates + qualified holdouts

Goal: amendment row T-032/T-033 (IMPLEMENTATION_PLAN:138): (a) new-domain
validity/migration gates, (b) fresh or separately qualified holdouts for
challenger evaluation. Requirements R-100 (experiment families with
selection lineage + method qualification + error allocation), R-101
(sealed-data access tracking, no holdout exposure resets), R-114
(explicit state transitions + schema/policy migration with immutable
historical identities).

## Q1 - What exists already?
**A:** domainpack module (T-032): qualification gate, units, validity,
actuation separation. selfimprove module (T-033): champion updates,
canary/rollback. This goal adds the AMENDMENT layer: migration gates
between pack/schema versions and holdout discipline for challenger
evaluation. (agent-default)

## Q2 - Module placement + seam?
**A:** New `crates/hephaestus/src/amendment/` module. Seam:
`check_migration(old_version, new_version, gates) -> Result<MigrationReceipt,
MigrationError>`, `allocate_holdout(challenger, pool) -> Result<Holdout,
HoldoutError>`. (agent-default)

## Q3 - Migration gate (R-114)?
**A:** Explicit transition: old version -> new version requires declared
schema gates (schema compatible, policy migration declared) and produces
an immutable receipt (old identity preserved verbatim in the receipt —
historical identities immutable). Undeclared transitions refused.
(agent-default)

## Q4 - Holdout discipline (R-101)?
**A:** `allocate_holdout` records sealed-data access per campaign with
authenticated identity; a holdout EXPOSED in a previous campaign cannot
be reset to sealed (exposure is monotonic — prevent holdout exposure
resets). A challenger must be evaluated on a FRESH holdout or a
separately qualified one (never on data whose seal was broken by the
selection process). (agent-default)

## Q5 - Experiment families (R-100)?
**A:** AmendmentHoldout binds: family id, selection lineage (which
campaigns selected this challenger), method qualification reference,
explicit family-level error allocation (declared before evaluation —
reuse t019 lineage continuity). (agent-default)

## Q6 - Vocabulary?
**A:** GLOSSARY rows FIRST: Migration receipt, Holdout exposure,
Error allocation. Decision row before edit. (agent-default)

## Q7 - TDD seams?
**A:** Red-first per ticket: migration receipts (01), holdout discipline
(02). (agent-default)
