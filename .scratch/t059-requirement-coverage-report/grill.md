# T-059 grill - requirement coverage report (productized coverage scan)

Goal: every derivation this session re-grepped R-ids with an ad-hoc
scope; the scope choice manufactured phantom gaps (crates+python only
→ 25 false "uncited") and the data-inclusive alternative is
tautological (requirements.json cites itself). Productize the scan as
a gated repo tool with an honest code scope and a reasoned allowlist.

## Facts (scope experiments)

- crates+python only: 25 uncited (phantom — most verified as
  implemented elsewhere over runs 17–26).
- + requirements/validation DATA: 0 uncited — tautology (self-cites).
- Honest code scope (crates/**.rs, python/**.py, tests/**(rs|py),
  tools/*.py, validation/*.py, .githooks/*): 7 uncited — R-054,
  R-079, R-081, R-085, R-087, R-088, R-089.
- Of those, honest code cites EXIST or are addable without strain:
  R-081 (verify_package.py validates owner/milestone/acceptance_tests
  — lines 36–38), R-087 (commit-msg hook is the ADR-citation
  enforcement), R-085 (release record's unresolved_questions doc),
  R-088 (verify_span source identity + `correct` in corpus tests),
  R-089 (method registry assumptions binding — tested).
  R-054 (no cache exists; views rebuildable by construction —
  N/A-until-cache) and R-079 (build-order process obligation) go to
  the reasoned allowlist.

## Q1 - What does the tool do?
**A:** `tools/check_requirement_citations.py`: scan the code scope,
diff against requirements.json, consult
`tools/requirement_citations.txt` (`R-XXX # reason` lines); FAIL on
an uncited-unallowlisted id AND on a stale allowlist entry (now
cited) — fail-closed both directions, mirroring
check_manifest_coverage's stale detection. PASS prints counts.
(agent-default)

## Q2 - Where does it wire in?
**A:** Makefile `req-coverage` target, added to the `ci` registry
list (a new R-gap fails every commit/CI until triaged: cite or
allowlist-with-reason). Makefile + gate_seal.py are gate files →
ADR-027 minted in RUNTIME_DECISIONS.md (verified NOT in MANIFEST);
`tools/requirement_citations.txt` joins gate_seal EXTRA_DATA_FILES so
the allowlist is sealed (silent gap-hiding impossible). Commits cite
ADR-027 later per the commit-msg hook. (agent-default)

## Q3 - Cite retrofits in the same ticket?
**A:** Yes — five comment-only honest cites (R-081 in
verify_package.py, R-087 in .githooks/commit-msg, R-085 in
release/record.rs doc, R-088 in corpus_ingest + corpus_review
headers, R-089 in method_registry header). Allowlist ends at exactly
2 entries. (agent-default)

## Q4 - Tests?
**A:** Tools in this repo are exercised by running them (no unit-test
precedent for tools/*); verification = `make req-coverage` green,
stale-detection demonstrated once (temporarily cite an allowlisted id
→ expect FAIL → revert), plus full `make ci` + doc-check.
(agent-default)

## Q5 - Scope?
**A:** No glossary rows (infrastructure tool, not domain vocabulary);
no scan of requirements data (tautology guard documented in the tool
docstring); no commit (rule 5). (agent-default)
