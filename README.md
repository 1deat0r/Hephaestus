# Hephaestus — Autonomous Invention Harness master-spec package
Version 1.2 · 30 September 2026

A specification for a self-improving harness that starts from a broad goal, discovers opportunities, originates mechanisms and falsifiable hypotheses, runs authorized experiments, and exports reproducible invention dossiers.

## Main documents

`MASTER_SPEC.md` is the full 31-section human-readable specification. `MASTER_SPEC.pdf` includes its reader edition and the four normative supplements. `HANDOFF.md` is the coding-agent entry point. `IMPLEMENTATION_PLAN.md` defines M0–M6 and T-001–T-034. `requirements.json`, `TRACEABILITY.md`, `docs/OBLIGATIONS.md`, and `ACCEPTANCE_TESTS.md` connect 119 obligations to future runtime tests. `release-scopes.json` separates contract readiness from runtime gates. `REVIEW_AND_RISKS.md` records corrections and remaining risks honestly.

The normative supplements define [qualification and lineage](docs/QUALIFICATION_CONTRACT.md), the [first domain/campaign protocol](docs/CONTEXT_ASSEMBLY_CAMPAIGN.md), [retrieval and durable-memory security](docs/RETRIEVAL_AND_MEMORY.md), and [mandatory autonomous self-improvement](docs/SELF_IMPROVEMENT.md). Numerical campaign targets remain provisional; no campaign was run.

## Contracts and checks

`schemas/contracts.schema.json` contains 18 principal record schemas. `examples/software-mission.json` is a synthetic, unexecuted context-assembly investigation fixture, not a real scientific result. The reference validator demonstrates selected cross-record and qualification guardrails; positive qualification fixtures use explicitly synthetic protected contexts. `tools/verify_package.py` checks reciprocal traceability, clause anchors, release scope, generated-document consistency and example conformance. Version 1.0 contracts, fixtures and validation evidence are preserved for compatibility and historical review.

Install the small validation dependencies in an isolated environment using `python -m pip install -r requirements-validation.txt`, then run:

```sh
python tools/verify_package.py
python -m unittest discover -s tests -v
```

`validation/REPORT.md` records what was actually checked. `MANIFEST.sha256` verifies packaged file contents. File hashes provide integrity checking, not signatures or legal proof.

Regenerate mapped documents with `python tools/render_contract_docs.py`. The optional PDF build and layout check use `requirements-pdf.txt`: `python tools/build_pdf.py`. Neither tool executes the invention runtime.

## Status

This is an implementation baseline, not a working invention application. No production runtime, physical experiment, independently reviewed invention, or product benchmark is included. The reference checks cannot establish scientific quality or production security.

Autonomous self-improvement is a required M3 capability: learn from mission outcomes, qualify challengers, deploy permitted changes, use them in later missions and roll back regressions. The runtime is not yet implemented.
