# T-061 spec - E2E mission driver

Status: ready-for-agent

## Problem Statement

Every subsystem is unit-tested but nothing composes them: no command
or test drives goal → mission → corpus → discovery → hypothesis →
experiment plan → evaluation → dossier (grep: zero partial chains;
CLI stops at the M1 fixture DAG; M2–M6 exit lines never assessed).
The user asked the distance to a real benchmark workflow; this is
the identified 1–3-run gap (Gap 1).

## Requirements trace

- Serves M2 exit (IMPLEMENTATION_PLAN:64 — grounded opportunities +
  test-ready hypotheses on hidden synthetic worlds), M3 exit
  (IMPLEMENTATION_PLAN:82 — outcome classes, honest negative export),
  and the receipt discipline of M0_M1_EXIT_ASSESSMENT.md. No new R
  obligation; R-033 (no fabricated numerics) binds ticket 02.
- Continuity: discovery span-citation discipline, mission auth
  fixtures, methods registry + interpret gates, dossier export gates
  (R-044/076/082), synthetic-evaluator stays a separate crate
  (dev-dependency only — hidden truth never becomes a runtime dep).

## Design

Three chained tickets (edges 01→02→03):

1. **M2 chain** (`tests/e2e_m2.rs`, DONE): fixture trace is the
   checked-in data file `tests/fixtures/e2e-trace.log`, generated
   inside the evaluator crate (`trace_fixture()`) and bound to
   `load_corpus()` worlds by that crate's regeneration test — no
   dependency edge either way (evaluator_access guard: control plane
   never links the hidden evaluator, even dev-deps) →
   `knowledge::ingest_bytes` → span-cited `TraceRecord`s →
   `discovery::analyze` → `assess_validity` → `genesis::registry::
   apply_all` → `hypothesis::compile` → `validators::validate` →
   `Readiness::TestReady`. Mission compiled from a fixture intake.
   Assertions structural only (no truth reads).
2. **M3 chain** (`tests/e2e_m3.rs`): methods-qualified interval
   computation over the SAME trace fixture durations (new registered
   MethodSpec + fn, unit-tested against hand-computed fixtures) →
   `experiment::compile` → `interpret` → TypedResult → Dossier with
   `EvidenceLabel::Measured` (receipt = sha256 of the fixture data) →
   `export` receipt; second scenario exports an honest negative
   (contradicted interval -> named negative kind). No evaluator linkage — hidden verdicts
   remain the evaluator suite's evidence (cited in ticket 03's exit
   assessment).
3. **CLI + exits**: `hephaestus fixture mission-run` printing the
   chain receipts (twin-run byte-identical, mirroring cli.rs
   precedent) + a fresh M2–M4 exit assessment document in the
   M0_M1 format (dispositions only where receipts exist — no
   fabrication of unrun clauses).

## Acceptance Criteria

1. `cargo test --test e2e_m2` green: grounded opportunity(s) +
   TestReady hypothesis from the hidden-world-derived fixture, zero
   truth reads.
2. `cargo test --test e2e_m3` green: supported-path typed result +
   exported dossier receipt (Measured label + fixture receipt);
   negative scenario exports honestly; interval fn matches
   hand-computed fixtures; register + interpret gates exercised; no
   evaluator crate referenced (guard suite stays green).
3. CLI subcommand green (exit 0, byte-identical twin runs) + M2–M4
   exit assessment with per-clause receipts or explicit NOT-RUN.
4. Gates green (`make ci` incl. req-coverage + doc-check); dev-dep
   added without touching the frozen envelope (Cargo.lock is
   allowlisted).

## Out of Scope

Model/baseline arms (consent-gated), campaign execution, T-033 live
wiring, ANY dependency on the hidden evaluator crate from the control
plane (evaluator_access guard — architectural, never widened), commit
(rule 5).

## Further Notes

Grill: `.scratch/t061-e2e-mission-driver/grill.md`.
