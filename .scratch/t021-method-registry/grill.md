# T-021 grill - fixed-sample method family, method registry (campaign M3 slice)

Goal: the fixed-sample repository-level method family from
`docs/CONTEXT_ASSEMBLY_CAMPAIGN.md` - bounded-outcome intervals, fixed-family
error allocation (Bonferroni), estimands, missingness handling, power/
precision inputs, protected method registry binding assumptions + identity
(IMPLEMENTATION_PLAN:72, campaign M3 paragraph, R-100, R-103).

## Q1 - Scope: what is IN and what is NOT?
**A:** IN: typed method registry, fixed-sample method definitions (paired
repository-level difference; bounded-outcome concentration interval; binary
yield difference; exact Bernoulli bound), error allocation from a fixed
family, missingness rules (failed/missing runs stay failures; missing timing
cannot become favorable), estimand records, frozen-before-confirmation
clipping policy, preregistration completeness gate (no n=null/unqualified
method/unresolved oracle into confirmation). NOT: actual statistical
computation over live data (that is the evaluator/result-interpreter path,
T-022), sequential methods (explicitly out of scope), and any fabricated
sample-size estimate - the package deliberately supplies none. (campaign
"Metrics and numerical decisions", agent-default)

## Q2 - Module placement?
**A:** New `crates/hephaestus/src/methods/` module. Pure registration +
validation logic. (agent-default)

## Q3 - Public seam?
**A:** `register_method(registry, spec) -> Result<MethodId, RegistryError>`
+ `preregister(registry, plan) -> Result<Preregistration, PreregError>` +
`bonferroni_alpha(family_size, alpha_total) -> Vec<f64>` +
`zero_failure_bound(n, alpha) -> f64` (the campaign's `1 - 0.05**(1/n)`
planning calculation, documented as planning-only). Red-first. (agent-default)

## Q4 - What binds a MethodSpec (R-100)?
**A:** name + version, estimand (what is being estimated, on which units),
assumptions (independence, distribution), error allocation, missingness
policy, clipping/bounding policy (frozen before confirmation),
implementation identity digest. Registry entries are immutable once
qualified - a change is a new version. (agent-default)

## Q5 - Missingness rules (campaign M3 paragraph)?
**A:** Encoded as an enum the spec must declare: FailedOrMissingIsFailure
(for useful-outcome yield), TimingMissingIsUnfavorable (missing timing
cannot become a favorable latency estimate), ExcludeWithReason (only where
the qualified method allows, reason recorded). Default is the strictest
applicable. (agent-default)

## Q6 - Error allocation (R-100, campaign "fixed Bonferroni family")?
**A:** `bonferroni_alpha(k, total)` splits the family alpha equally;
allocation recorded on the method spec. Sum of allocations <= total alpha.
(agent-default)

## Q7 - Preregistration gate (campaign "No runtime manifest with n=null,
unqualified method, or unresolved oracle can enter confirmation")?
**A:** `preregister` refuses: n=None, method not in the qualified registry,
oracle unresolved. Each refusal is a named PreregError. The manifest MUST
record the n-calculation reference, alpha allocation, reference
distribution, power/precision target, sensitivity assumptions (campaign
 MUST-include list) - absent entries are named errors. (agent-default)

## Q8 - Zero-failure bound?
**A:** `zero_failure_bound(n, alpha) = 1 - alpha^(1/n)` exactly as the
campaign states (IID Bernoulli, zero failures); doc-comment labels it a
planning calculation, not a prescribed sample size or universal safety
claim. n=299 with alpha=0.05 gives <= 0.01 (the campaign's own example -
assert it). (agent-default)

## Q9 - Protected registry semantics (R-095-adjacent)?
**A:** Methods are registered with implementation identity digest; the
registry rejects duplicate (name, version) and treats any change as a new
version - "protected" here means immutable-once-registered, structurally
enforced. (agent-default)

## Q10 - Vocabulary?
**A:** GLOSSARY rows FIRST: Method registry, Error allocation, Estimand.
Decision row before edit. (agent-default)

## Q11 - TDD seams?
**A:** Red-first per ticket at `register_method` + `preregister`. (agent-default)
