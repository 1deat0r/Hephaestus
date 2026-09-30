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
