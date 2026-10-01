# Runtime implementation decisions

Architecture decisions for the Hephaestus runtime, recorded from T-001 onward.
Style follows `docs/DECISIONS.md`; rejected alternatives included.

## ADR-019 — Runtime decisions live in this file, not `docs/DECISIONS.md`

`docs/DECISIONS.md` (ADR-001–018) is covered by `MANIFEST.sha256` and is part
of the frozen v1.2 evidence envelope: appending to it would break package
integrity verification. Runtime-era decisions are therefore recorded here, in
the same one-paragraph style, and this file is deliberately outside the
manifest. Rejected alternative: editing `docs/DECISIONS.md` and regenerating
the manifest — that would rewrite reviewed v1.2 evidence.

## ADR-020 — The hidden evaluator's permission scope is the Cargo dependency graph

The synthetic-world evaluator (T-003) lives in `crates/synthetic-evaluator`,
which `crates/hephaestus` does not depend on; `tests/evaluator_access.rs`
denies manifest, lockfile, and source-level paths to it. Workers and the
control plane receive `{input, output}` observation projections only.
Rejected alternative: a visibility convention inside one crate (not
enforceable); process isolation (adopted later at T-010/T-022, where OS
mechanisms exist).

## ADR-021 — Manifest approvals are keyed HMAC; bare hashes never authorize

T-004 approvals use HMAC-SHA256 over canonical manifest bytes with keys
provisioned outside the record system; verification fails closed on wrong
key, tampered payload, malformed tag, or an unkeyed content hash
(`tests/approval_deny.rs`). Rejected alternative: treating SHA-256 digests as
signatures (explicitly forbidden by IMPLEMENTATION_PLAN T-004); asymmetric
signatures (deferred until a deployment boundary needs them).

## ADR-022 — Cross-language digest conformance is vector-pinned and fails closed

The Rust canonical-JSON digest profile is pinned by 48 vectors generated from
the Python reference (`tools/gen_digest_vectors.py`); both languages assert
against the same file. Known divergence: exponent-formatted floats render
differently, which produces mismatched digests and therefore *rejects*
bindings rather than accepting them. Rejected alternative: claiming
unconditional cross-language equivalence without vectors (the package README
requires these vectors before runtime use).

## ADR-023 — Traceability is enforced twice, from one source of truth

The 119 obligation mappings are validated by both the Python reference
(`tools/traceability.py`) and a Rust port (`crates/hephaestus/src/traceability.rs`)
with parity tests, including a byte-exact render of the three generated
documents. A mismatch in either implementation fails CI. Rejected
alternative: trusting only the reference tools (independent re-derivation
catches parser/render drift on the runtime side).

## ADR-024 — Documentation and tree freshness is enforced in tiers, inside the single gate registry

Documentation, folder, and file freshness is enforced as a four-layer model
(L1 integrity of frozen bytes, L2 derived-document freshness, L3 prose
semantics, L4 tree structure) whose checks are Makefile targets in the single
gate registry (Decision 6a): the pre-commit hook runs `make ci-fast`
(fast spec checks plus a staged conflict/whitespace check), `make ci` adds the
tree-level conflict grep, a manifest-coverage classifier asserting
`git ls-files ⊆ MANIFEST.sha256 ∪ tools/runtime_allowlist.txt` and the
reverse, an offline internal link/anchor checker, a README fence assertion
(unparseable = fail), and a runtime gate seal (`tools/gate_seal.sha256`
covering Makefile, `.github/workflows/`, `.githooks/`, `tools/*.py`), and
GitHub CI re-runs `make ci` on every push. A commit-msg hook requires an
ADR citation on any diff touching gate files. All of these are contract/spec
checks only — nothing here is runtime acceptance (AT-113). Serves R-014,
R-016, R-043, R-052, R-063, R-077, R-081, R-087. Fail-closed by construction:
any checker exception or parse failure fails the gate; suppression is only by
enumerated `path:line` baseline entries that may shrink but never grow via CI;
ignore-globs are forbidden (R-077, R-052).

L3 (prose matching reality) is explicitly *not fully mechanizable*: the
mechanizable slices (links, fences, structure) are gated, and the remainder is
covered by audit-as-artifact — `validation/<date>/AUDIT.md` written by the
same authoring assistant, with a cadence watchdog on the scheduled workflow
and an advisory age report in `make ci`. Per R-086 these audits are
author-led and are not independent reviews; per R-014 and R-016 they keep
evidence and exact sources outside ephemeral conversations.

Erratum (R-088): the raw commands at `README.md:19–20` and `HANDOFF.md:27`
(`python tools/verify_package.py`, `python -m unittest discover -s tests -v`)
are valid but superseded by the `make setup` / `make ci` flow, since
validation dependencies now live in `.venv`. The MANIFEST-covered bytes and
hashes are unchanged; this correction deliberately lives outside the frozen
envelope and must be re-validated by 2027-03-30 (either fold into the next
spec version's reseal or extend by explicit decision).

Gate files can never join `MANIFEST.sha256` (spec-package-only forever, Q9a /
ADR-017 / ADR-019), so the gate seal is the admissible tamper-detection
substitute; the honest residual trust is stated here: an agent holding write
access to gates, config, and history can still subvert every in-repo check,
and the only out-of-repo anchor (a GitHub ruleset requiring CI and alerting
on workflow changes for `main`) must be configured by the repository owner.
Rejected alternatives: adding gate files to MANIFEST (would rewrite reviewed
v1.2 evidence); Renovate/Dependabot (PR flow conflicts with direct commits on
`main`); running every check on every commit (mid-task transient link
failures create retry pressure; tiering keeps commits fast while `make ci`
backs stops pushes); glob-based ignore lists (silent suppression creep);
executing README fences as tests (illustrative commands, false-fire risk);
adopting the pre-commit framework (a second gate registry, contradicting the
single-registry doctrine).

Appendix (non-normative tier table): conflict-staged → ci-fast, fail;
gen-check / verify / test-package → ci-fast, fail; conflict-tree,
manifest-classifier, md-links, readme-fences, gate-seal, fmt, clippy,
test-rust, test-py, hooks-check → `make ci`, fail; commit-msg ADR citation →
hook, fail; unreferenced-file report → `make ci`, advisory; full `make ci` →
GitHub CI (SHA-pinned actions), fail; online link audit + audit-age watchdog →
scheduled workflow, fail-in-workflow; `cargo doc -D warnings` → separate CI
job, later; GitHub ruleset → repository owner, external.

## ADR-025 — The pre-commit tier runs the manifest classifier; rustdoc gets one named command

Two checks that only GitHub Actions ran became reachable locally: the
manifest-coverage classifier (ADR-024 layer 4) joined `ci-fast`, so the
pre-commit hook — which runs exactly `make ci-fast` — now blocks a commit
whose tracked files are neither manifest-covered nor allowlisted, with the
classifier's own hint, instead of letting a push go red (run 3 pushed twice
red through precisely this hole). `make doc-check` became the single
definition of `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`,
and the remote docs job calls it, so the command cannot drift between
registry and workflow. Serves the same obligations ADR-024 cites (R-014,
R-016, R-043) by making its layer-4 classifier reachable before a push;
introduces no new obligation. Measured `ci-fast` overhead: baseline pair
0.928 s / 0.950 s (spread 0.022 s), after 0.954 s — the delta sits inside
the baseline spread; timings in the run-4 state LOG. `manifest-check` stays
explicitly listed on `ci` as well — the target list is the human-readable
gate inventory, and the deterministic double-run is free at that scale.
This supersedes two ADR-024 appendix rows: "manifest-classifier → `make
ci`" becomes "`ci-fast` (hence every commit) and `make ci`"; "cargo doc →
separate CI job, later" becomes "local `make doc-check` target, called by
the separate CI job". Rejected alternatives: putting cargo doc into `ci-fast`
or `ci` (violates ADR-024 step 10's latency decision — a multi-second rustdoc
build does not belong on the commit hot path; the remote job stays the
backstop); moving `md-links` into `ci-fast` (ADR-024 already rejected
transient link failures on the commit path); editing ADR-024's appendix in
place (registers are append-only records — supersession is stated here).
