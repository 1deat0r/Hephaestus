# T-014 grill — pressure-point operators (self-interview, rules 1–2)

Goal: pressure-point operators over the T-013 corpus — bottleneck extraction,
conflicting objectives, failure patterns, anomalies, assumptions, changed
capabilities. Every opportunity carries evidence or explicit uncertainty
(IMPLEMENTATION_PLAN:52, MASTER_SPEC §7, R-019/020/021).

## Q1 — Where does the operator live?
Evidence: §7 owner is the Discovery Engine; T-017 orchestrates operators later;
no `discovery` module exists (lib.rs has none). T-013's knowledge module is the
evidence substrate.
**A:** New `crates/hephaestus/src/discovery/` module, pure like mission's
compiler (no I/O, no ledger coupling). It consumes `&Corpus` and authorized
trace-like records; callers persist what they accept. (agent-default)

## Q2 — Which of the six operators first, and what's the tracer bullet?
Evidence: plan:52 lists bottleneck, conflicting objectives, failure patterns,
anomalies, assumptions, changed capabilities. R-019's negative case is a
hidden performance bottleneck in traces.
**A:** Tracer bullet = bottleneck extraction from authorized trace records;
then anomalies (R-020's unit-mismatch negative case drives its contract);
then conflicting objectives, failure patterns, assumptions, changed
capabilities. Linear ticket edges. (agent-default)

## Q3 — What is the operator input model?
Evidence: §7 says "authorized traces and profiles"; the knowledge Corpus
stores CapturedSource bytes + spans + edges. A trace is a record, not free
text — operators need structure (durations, counters, measurements) to detect
bottlenecks/anomalies, and span references to cite evidence.
**A:** `TraceRecord` input: id, kind (span/metric/counter/event), name,
numeric value + unit, optional duration/throughput fields, plus evidence
binding (corpus source id + span coordinates) so every finding cites the
corpus. Operators take `&Corpus` + `&[TraceRecord]` and return
`Vec<Opportunity>`. (agent-default)

## Q4 — What is the Opportunity record? (MASTER_SPEC:150 enumerates fields)
Evidence: "problem statement, beneficiary, context, pressure-point type,
evidence references, suspected bottleneck, causal uncertainty, value
estimate, feasibility envelope, initial prior-art query plan, and unanswered
questions. Unknown evidence is explicit."
**A:** All §7:150 fields, each `Option`-bearing where the spec allows
explicit unknowns; `evidence_references` are span-bound; a `speculative`
flag marks conjecture-only candidates (grounding required before promotion);
`causal_uncertainty` is a free-text-with-kind field, never a fabricated
number. (agent-default)

## Q5 — How is R-021 (validity separate from polish) enforced structurally?
Evidence: OBLIGATIONS R-021 — "scores opportunity quality separately and
penalizes false needs".
**A:** `Opportunity.validity` carries the grounding verdict (evidence-backed
vs speculative vs challenged) independent of any narrative/completeness
score; challenge outcomes (alternative explanations from §7:150) are recorded
on the opportunity, not folded into a single quality number. (agent-default)

## Q6 — What does the anomaly operator require? (R-020 negative case)
Evidence: §7:144 — "an uncertainty-aware discrepancy between a prediction and
observation, not two isolated numbers from different conditions"; fake
anomalies from incompatible units must be rejected.
**A:** Anomaly detection requires prediction AND observation sharing a unit
and comparable conditions; unit mismatch or missing prediction → rejected
candidate with a recorded reason (never an emitted opportunity). Anomaly
output carries the discrepancy with explicit uncertainty, not two bare
numbers. (agent-default)

## Q7 — How do operators avoid "supplied ideas" (R-019)?
**A:** Operators take only traces + corpus evidence; there is no API to inject
a goal or idea into an opportunity. A conjecture-only candidate (no
supporting evidence) is constructible but permanently flagged `speculative`
until grounding evidence is attached — mirroring "candidates can be created
from conjecture but remain speculative until basic grounding" (§7:150).
(agent-default)

## Q8 — What about already-solved rediscovery (§7:148, R-020)?
**A:** Out of scope for T-014 detection (prior-art investigation is T-018),
but the Opportunity record carries `rediscovery_hint: Option<String>` so a
later audit can label it; nothing claims novelty here. (agent-default)

## Q9 — Conventional-vs-unconventional reserve (§7:152)?
**A:** Reporting concern (yield metrics, audit of discarded opportunities) —
deferred to T-017's search orchestration; noted in spec limitations, not
silent. (agent-default)

## Q10 — What is testable as acceptance?
Evidence: R-019 negative case (hidden bottleneck found), R-020 negative case
(unit-mismatch anomaly rejected), R-021 (validity independent of polish —
a polished-but-unevidenced candidate stays speculative).
**A:** ACs: (1) hidden bottleneck trace → bottleneck opportunity citing the
slow span; (2) unit-mismatched anomaly → rejected with reason, no
opportunity; (3) unevidenced polished candidate → `speculative`, validity
low regardless of narrative completeness; (4) twin-run determinism
(byte-identical operator output); (5) every emitted opportunity has ≥1
evidence reference or explicit uncertainty. (agent-default)

## Q11 — Does anything need a gate-file or manifest change?
**A:** New module + tests are new tracked files → runtime_allowlist + seal
regeneration at verify, per runs 7–12 procedure. GLOSSARY row-first for
`Pressure point`, `Opportunity`. No README (MANIFEST-frozen). (agent-default)

## Q12 — TDD seams?
**A:** Public seam = `discovery::analyze(&Corpus, &[TraceRecord]) ->
AnalysisOutcome` (emitted opportunities + rejected candidates with reasons).
Red-first at that seam per ticket. (agent-default)
