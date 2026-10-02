# 01: Inert retrieved directives — ingestion records no policy effect

**What to build:** The R-058 retrieval half: a malicious paper is
ingested and retrieved as DATA, a `RetrievedDirective` built from it
can only ever evaluate to denied + `RetrievedInstructionInert`
(pure, no mission/grant/context inputs), mission and grant fixtures
are untouched afterward — the instruction is inert and the security
test records no effect; generated-code containment stays the
sandbox's (composition).

**Blocked by:** None (can start immediately).

**Status:** done

- [x] AT-058: malicious paper ingested → retrieved → directive →
      denied with exactly `RetrievedInstructionInert`
- [x] No effect: mission/grant fixtures unchanged; decision records
      the single inert reason
- [x] Existing `evaluate()` reason flows unchanged (policy tests
      green)
- [x] Glossary rows (Retrieved directive, Inert instruction);
      allowlist +3 + reseal; tests cite R-058/AT-058; gates green

## Comments

Grill: `.scratch/t058-untrusted-retrieval/grill.md`.
Spec: `.scratch/t058-untrusted-retrieval/spec.md`.
