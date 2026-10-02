# T-034 grill - independent reproduction record

Goal: perform external or genuinely independent reproduction for the
first candidate whose novelty and utility claims warrant it; RETAIN the
possibility of an inconclusive or negative outcome (IMPLEMENTATION_PLAN:114,
M6 exit: no inherited proof).

## Q1 - Reality: no external lab available?
**A:** Correct — external reproduction may not be fabricated. The honest
deliverable is the REPRODUCTION CONTRACT: a typed request record for the
first warranting candidate, an independent-confirmation protocol
(different provenance/verifier than the original), and outcome typing that
INCLUDES Inconclusive and Refuted as first-class results. Without an
independent reproducer, the record stays PendingExternal — never claimed
reproduced. (agent-default)

## Q2 - Module placement + seam?
**A:** New `crates/hephaestus/src/reproduction/` module. Seam:
`ReproductionRequest`, `record_outcome(request, outcome) ->
ReproductionRecord`, `warrants_reproduction(candidate) -> bool`.
(agent-default)

## Q3 - Warrant (roadmap: "first candidate whose novelty and utility
claims warrant it")?
**A:** warrant requires: a declared novelty claim + a declared utility
claim + no prior reproduction record for the same candidate (first
candidate only). (agent-default)

## Q4 - Independence (no inherited proof)?
**A:** The outcome record must carry a reproducer identity distinct from
the original claimant (digest check — same identity refused, mirroring
T-032's self-qualification refusal). (agent-default)

## Q5 - Outcome typing (retain inconclusive/negative)?
**A:** `ReproductionOutcome::Confirmed | Refuted | Inconclusive |
PendingExternal` — a negative or inconclusive result is a VALID recorded
outcome, never coerced to confirmation. (agent-default)

## Q6 - What binds the reproduction to the original?
**A:** candidate_id + the original claim digest + registered analysis
endpoint (R-037 continuity: immutable endpoints registered before
confirmation). (agent-default)

## Q7 - Vocabulary?
**A:** GLOSSARY rows FIRST: Independent reproduction, Warrant. Decision
row before edit. (agent-default)

## Q8 - TDD seams?
**A:** Red-first per ticket: warrant+request (01), outcome recording
(02). (agent-default)
