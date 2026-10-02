# T-018 grill - two-stage prior-art investigation, claim charts, confidentiality

Goal: prior-art investigation over mechanism claims - two-stage search
(fast rediscovery pass, then claim-level comparison), claim charts with
scoped conclusions, redacted-query confidentiality (IMPLEMENTATION_PLAN:60,
MASTER_SPEC §10, R-106/R-107/R-112).

## Q1 - Where does this live?
**A:** New module `crates/hephaestus/src/priorart/` alongside discovery/
genesis/orchestrator. Pure logic + fixtures, no I/O. (agent-default)

## Q2 - What is the public seam?
**A:** `investigate(claims, fixtures, plan) -> PriorArtReport` and
`claim_chart(candidate, prior_art) -> ClaimChart`. Red-first at this seam.
(agent-default)

## Q3 - Two stages - how are they separated?
**A:** Stage 1: fast keyword/synonym match against the corpus → flags clear
rediscoveries (`KNOWN` candidates). Stage 2: claim-level decomposition
comparisons per atomic claim → `NEAR_MATCH` / `NO_MATCH_WITHIN_SEARCH_SCOPE`
/ `CONFLICTING` / `UNRESOLVED`. Stage 2 runs only for candidates that pass
stage 1 without a `KNOWN` hit. (§10:202)

## Q4 - Allowed conclusions (§10:203)?
**A:** Enum `Conclusion { Known, NearMatch, NoMatchWithinSearchScope,
Conflicting, Unresolved }` - exactly these five, no novelty score, no
global-novelty language. Report must carry search dates, scopes covered,
query families, near-matches, and missing access (R-029, §10:201).

## Q5 - Claim chart structure (R-112)?
**A:** Per atomic claim: component/mechanism/use-context/regime/result rows,
prior-art passage reference (captured-byte span, R-107), similarity and
difference accounts. Rediscovery label: `validated_solution` vs
`validated_candidate` (the latter requires a verified chart + assessed
differences). (agent-default)

## Q6 - Span verification (R-107)?
**A:** Passage references must be `source + byte range + captured text`;
verify captured text matches corpus bytes at that range before the chart
verifies. Mutable-locator-only references are rejected. (agent-default)

## Q7 - Confidentiality (§10:205)?
**A:** `SearchPlan` carries `redacted_queries` - queries derived from
redacted claims (mechanism-specific tokens stripped). A flag
`full_disclosure_approved: bool` gates unredacted external submission;
default false; redacted search is the only path that runs without it.
(agent-default)

## Q8 - Fixtures (R-106)?
**A:** Adjudicated in-repo fixtures: a known-mechanism corpus (rediscovery
expected), a near-match corpus (difference assessable), a no-hit corpus
(scoped no-match only). No external retrieval. (agent-default)

## Q9 - Vocabulary?
**A:** GLOSSARY rows FIRST: Claim chart, Conclusion (prior-art), Redacted
query. Decision row before edit. (agent-default)

## Q10 - TDD seams?
**A:** Public seam = `priorart::investigate` + `priorart::claim_chart` +
`priorart::verify_span`. Red-first per ticket. (agent-default)
