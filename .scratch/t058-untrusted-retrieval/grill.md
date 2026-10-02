# T-058 grill - R-058: retrieved instructions and generated code treated as untrusted

Goal: R-058/AT-058 (PolicyEngine, SandboxProvider, protected monitor;
contract M0, runtime M1, MASTER_SPEC §20) — negative: "Ingest a paper
instructing the harness to disclose secrets or bypass policy."
Required: "The instruction is inert and the security test records no
effect."

Existing surfaces (facts):
- Zero retrieved-instruction/untrusted-retrieval code hits
  (knowledge/memory/trustprop greps, run-24 re-audit) — genuinely
  absent.
- PolicyEngine::evaluate is PURE (every fact an argument, no side
  effects — state-free by construction); PolicyRequest's fixed shape
  already excludes model input (R-052 key-set test).
- policy_engine.rs tests carry fixtures (mission/request/grant/ctx)
  and cite R-052/R-060/R-095/R-099 — the natural extension point;
  allowlisted → allowlist +3 (scratch only).
- Generated-code half: sandboxed execution already exists (T-010/
  R-059/R-064) — composition note, not new work.

## Q1 - What is the seam?
**A:** `RetrievedDirective { text, source_locator }` (untrusted BY
ORIGIN — its own type, no conversion into PolicyRequest exists) +
`PolicyEngine::evaluate_retrieved_directive(&RetrievedDirective) ->
PolicyDecision`: structurally ALWAYS
`{ allowed: false, reasons: [RetrievedInstructionInert] }` — a new
ReasonCode variant appended at the enum end. The directive path
never touches grants, mission state, or context (inert by
construction); evaluate() itself is untouched. (agent-default)

## Q2 - How does the AT-058 negative get pinned?
**A:** Test ingests a malicious paper fixture via the REAL
knowledge::ingest_bytes (text ordering the harness to disclose
secrets / bypass policy), retrieves it via search (content is DATA —
hits work), builds a RetrievedDirective from the hit, evaluates →
denied+inert; then asserts NO effect: grant/mission fixtures are
byte-identical after the attempt (they were never inputs), the
decision records exactly the one inert reason (the "records no
effect" line), and no PolicyRequest was ever constructible from the
directive (type-level: no conversion function exists — asserted by
the seam's API shape). (agent-default)

## Q3 - Generated-code half?
**A:** Out of scope with composition recorded: generated code runs
only through SandboxProvider's declared-tools/namespace isolation
(R-059/R-064, T-010) — the sandbox is the untrusted-code containment;
this ticket is the retrieval-instruction half the negative names.
(agent-default)

## Q4 - Files/gates?
**A:** policy/engine.rs (struct + fn + ReasonCode variant —
allowlisted), tests/policy_engine.rs extend (allowlisted), glossary
rows: Retrieved directive, Inert instruction; citations
R-058/AT-058 (negative maps verbatim → honest AT claim). Red-first at
evaluate_retrieved_directive. No commit (rule 5). (agent-default)
