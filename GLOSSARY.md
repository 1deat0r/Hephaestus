# Hephaestus

Shared language for the Autonomous Invention Harness project: the specification package and the runtime that will be built from it.

## Language

**Spec package**:
The reviewed, versioned contract at this repo's root (`MASTER_SPEC.md`, `requirements.json`, `schemas/`, `tools/`) — the product contract, frozen per version.
_Avoid_: the app, the product

**Runtime**:
The not-yet-built Rust control plane and Python workers that implement the spec package.
_Avoid_: the app, the product, Hephaestus (ambiguous between the two)

**Obligation**:
A stable requirement ID (`R-nnn`) in `requirements.json` that a mapped runtime test must eventually satisfy.
_Avoid_: requirement, ticket

**Task**:
A work item (`T-nnn`) in `IMPLEMENTATION_PLAN.md` with declared prerequisites and an exit gate.
_Avoid_: ticket, story

**Milestone**:
A release gate (`M0`–`M6`) grouping tasks behind a labeled exit condition.
_Avoid_: phase, sprint

**Event ledger**:
The append-only, sha256-chained history of contract events in the Rust control plane; the authoritative record that every other view is rebuilt from.
_Avoid_: log, event stream, journal

**Projection**:
A derived view (such as the timeline index) built from the event ledger after its events are durable; rebuildable at any time and never authoritative.
_Avoid_: cache, materialized view

**Staged artifact**:
Bytes written to the store's staging area under a temporary name; addressable only after the atomic commit rename places them at their digest.
_Avoid_: temp file (too broad), pending artifact

**Content addressing**:
Naming an object by the sha256 of its bytes, so the name attests the content and reads can re-verify them.
_Avoid_: hashing, checksum (weaker claim)
