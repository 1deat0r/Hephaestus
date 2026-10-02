# 02: Two-stage investigation, claim charts, report, labels

**Status:** done

**What to build:** `investigate(claims, corpus, plan) -> PriorArtReport`:
stage-1 fast keyword/synonym rediscovery pass (`KNOWN`), stage-2 claim-level
comparison for survivors (`NEAR_MATCH`/`NO_MATCH_WITHIN_SEARCH_SCOPE`/
`CONFLICTING`/`UNRESOLVED`); `claim_chart` with per-claim rows + passage
refs (span-verified) + similarity/difference accounts; report metadata
(dates, scopes, query families, near-matches, missing access - R-029);
`validated_solution` vs `validated_candidate` labeling (R-112).

**Acceptance:**
- [x] Stage 1 flags rediscovery as `KNOWN` (known-mechanism fixture) (spec AC 1)
- [x] Stage 2 verdicts with difference accounts on near-match fixture (spec AC 2)
- [x] Chart refuses unverified spans (spec AC 3)
- [x] Report metadata complete (spec AC 5)
- [x] Labels: rediscovery → validated_solution; candidate requires chart (spec AC 6)
- [x] Twin-run byte-identical (spec AC 7)

## Comments
