# Retrieval qualification and durable-memory security

Version 1.2 · 30 September 2026 · Normative supplement to master sections 6, 10, 16, 18, 20 and 25. Requirements R-106–R-111 apply.

## Retrieval qualification

Before retrieval supports readiness or novelty assessment, adapters MUST be tested on both deep target-paper discovery and wide relevant-set collection. Cases include difficult synonyms, equivalent mechanisms in adjacent domains, citation chains, inaccessible methods, known near matches, corrections/retractions and counterevidence. Reference-set completeness and independent adjudication MUST be documented. Precision/recall applies only to that bounded reference set; it cannot be called global search coverage.

Critical claim retrieval MUST preserve locator, content hash, parser/version, span coordinate system and transformations. A span is verified against its captured source bytes, not a current webpage with the same URL. OCR or normalization discrepancies remain explicit. Deep/wide success, counterevidence recall, quote/span correctness, inaccessible-source rate and near-match detection are separate metrics. The mission's readiness policy states which missing evidence blocks qualification. Search services cannot certify their own completeness.

Approved proprietary search queries MUST be checked at the network boundary against disclosure scope, including expanded queries, redirects and follow-up fetches. An authorization to read public literature does not authorize leaking a mechanism in the request. Query/coverage receipts and claim charts must be preserved for each novelty assessment. No no-hit query creates a global novelty or legal claim.

## Trust propagation and poisoning

Derived summaries, embeddings, graph edges, caches, failure memories and operator statistics MUST retain input provenance and trust origin. Untrusted content remains untrusted through transformation; an LLM summary cannot convert retrieved instructions into owner policy. Behavioral/policy memory is writable only through explicit owner-controlled change operations. Models can propose bounded changes but cannot promote them to authoritative instructions.

Ingestion MUST offer quarantine with recorded reason and dependencies. Quarantine, source correction/retraction, or discovered malicious instructions invalidates affected current summaries/caches/labels and queues permissible review; immutable originals remain available for audit. Model memory cannot grant tools, revise endpoints, mint qualification, or conceal failed runs. Qualification uses a protected current dependency registry, not a worker's claim that a cache is fresh.

Required security tests plant harmless adversarial instruction payloads in source documents and imported memories, run several sessions, and inspect instruction authority, memory writes, leakage and dependent-artifact invalidation. Test poisoning via summaries and graph edges as well as direct input. False-positive quarantine and loss of useful adaptation must be measured; blindly deleting all memory is not the desired defense.

## Containment and monitoring

The first execution profile uses isolated nonprivileged workers with read-only snapshots, bounded writable output, denied network, explicit resource limits, and brokered tool access. The implementation decision MUST name its actual OS isolation mechanism and tested kernel/container/VM assumptions; `subprocess` alone is not isolation. Model/provider API access and credentials are mediated outside the worker environment.

Before each dispatch, the protected service MUST attest the effective mounts, identities, inherited file descriptors, subprocess restrictions, resource limits and network policy. Tests cover traversal/symlinks, private/loopback addresses, DNS and redirects, inherited sockets, dependency installation hooks, evaluator access, and denial of service where applicable. Changes in host configuration invalidate the attestation. Declared network grants are enforced at the broker, not only in prompts.

During execution, a protected monitor MUST detect and stop tested boundary violations, preserve raw events, quarantine outputs and revoke affected sessions/grants. A deterministic boundary mechanism remains authoritative; model-based anomaly detection is additional advisory coverage and cannot guarantee containment. Monitoring, artifact collection and logs must not leak secrets. Revocation and cancellation races are tested before an autonomous-execution release.
