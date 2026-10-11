# AE-04 S2 receipt — invalidate affected derivatives

Date: 2026-10-11 UTC.
Requirement IDs: R-015, R-017, R-107, R-108, R-111. Spec AC: 11.
No architectural departure: no RUNTIME_DECISIONS entry needed.

## Red (missing behavior)

Command:
cargo test -p hephaestus --test architecture_incremental_invalidation s2_invalidation

Result: compile failure (E0432): `InvalidationCause`,
`InvalidationEvent`, `InvalidationTracker`, and `QualificationError`
are absent from `hephaestus::trustprop`.

## Green (after implementation)

Command: same as above.
Result: 5 passed, 0 failed —
s2_invalidation_correction_marks_target_and_dependents_stale,
s2_invalidation_quarantine_and_retraction_propagate,
s2_invalidation_preserves_audit_originals,
s2_invalidation_blocks_stale_qualification_while_review_pending,
s2_invalidation_allows_qualification_after_review_resolves.
Runner: `python3 .scratch/architecture-efficiency/workflow.py verify
AE-04 S2` → PASS, exit 0.
Receipt: ae04-s2-workflow-verify.log.

## Implementation

`crates/hephaestus/src/trustprop/mod.rs` — new `InvalidationTracker`
over the S1 `DependencyIndex`: `apply` marks the target plus its
transitive dependents stale and appends the event to audit; audit
offers no removal API so originals survive; `can_qualify` refuses a
stale derivative while its review stays pending and passes after
`resolve_review` or when no review pends. All three causes share one
version-scoped traversal; causes stay distinct in audit.

## Regressions

`trust_propagation` 4 passed; `memory_quarantine` 4 passed;
`evidence_lifecycle` 7 passed. `make ci` and `make doc-check` run
before landing.

## Limitations

The tracker lives in memory; persistence uses index links plus audit
replay (not yet wired). Review pending state is explicit caller
state, not an automatic review-service hook. S3 builds exact reuse
on this stale set.
Developer model is Muse Spark per user direction; the Pi protocol
names MiMo. Contract code carries no model dependency.
