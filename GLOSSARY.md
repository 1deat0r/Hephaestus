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
