# T-029 grill - typed decision provider, bounded advisory evaluation

Goal: evaluate a typed decision provider (Jev or equivalent) for bounded
ADVISORY work - calibration, abstention, rejection false negatives, cost,
end-to-end quality; deterministic control-plane gates stay authoritative
(IMPLEMENTATION_PLAN:98, R-005, R-090, AGENTS.md "Jev is a probabilistic
advisory provider, not deterministic scientific logic").

## Q1 - The constraint reality?
**A:** No Jev/decision provider is installed, and AGENTS.md says Jev is a
probabilistic ADVISORY provider kept behind capability contracts. This goal
implements the advisory CONTRACT LAYER: the typed interface a provider must
pass, calibration/abstention/rejection measurement records, and the
rule that control-plane gates are deterministic and authoritative
(regardless of any advisory input). No provider integration, no fabricated
calibration numbers. (agent-default)

## Q2 - Module placement + seam?
**A:** New `crates/hephaestus/src/advisory/` module. Seam:
`AdvisoryProvider` trait (propose_decision + abstain + confidence),
`evaluate_provider(proposer, fixtures) -> ProviderReport`,
`assert_control_authority() -> ControlAttestation`. (agent-default)

## Q3 - What does an advisory decision look like?
**A:** `AdvisoryDecision { decision_id, recommendation: Recommendation,
confidence: Confidence, rationale_ref }` where Recommendation ∈
{Prioritize(OpportunityId), Deprioritize, Abstain} and Confidence is
explicitly a MODEL JUDGMENT (§16 continuity: "uncalibrated model
probabilities are stored as model judgments, not scientific posterior
probabilities") - a labeled enum, never merged into evidence. (agent-default)

## Q4 - Calibration, abstention, rejection false negatives?
**A:** Typed measurement records computed from FIXTURES with KNOWN
outcomes: calibration buckets (predicted-confidence bucket vs observed
correct rate), abstention rate, rejection-false-negative count (provider
rejected a case that was actually resolvable). Computed only where
outcomes are known; elsewhere NotMeasured. (agent-default)

## Q5 - Control-plane authority (R-005, deterministic gates)?
**A:** `assert_control_authority` returns an attestation that ALL control
decisions (budget, promotion, authorization) flow through the existing
deterministic modules (policy/budget-ledger/selfimprove gates), and that
advisory input CANNOT: (a) change a gate outcome, (b) mint budget,
(c) qualify a method. Structurally: the advisory module has no mutable
handles to those modules. (agent-default)

## Q6 - Core works without the provider (R-005)?
**A:** The advisory module is OPTIONAL by construction: no other module
imports it (checked by the module graph); the trait has an
`NullProvider` (always abstains) proving the core runs without any
provider. (agent-default)

## Q7 - Official vs third-party (R-090)?
**A:** The trait is provider-neutral; a provider records its
implementation identity (name + digest) and the report labels whether it
is the official Jev model or a third-party implementation - never
conflated. (agent-default)

## Q8 - Cost + end-to-end quality?
**A:** Cost receipts as quantities (R-103 continuity); end-to-end quality
measured ONLY as the delta on fixtures with known outcomes, else
NotMeasured. (agent-default)

## Q9 - Vocabulary?
**A:** GLOSSARY rows FIRST: Advisory decision, Calibration record,
Control-plane authority. Decision row before edit. (agent-default)

## Q10 - TDD seams?
**A:** Red-first per ticket: trait+decisions (01), evaluation+authority
(02). (agent-default)
