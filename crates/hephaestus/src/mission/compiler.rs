//! Intent-to-Mission compiler (T-012): `compile` turns a broad authorized
//! goal into a versioned `Mission` or into authorization requests.
//!
//! The function is pure and deterministic: the same intake always yields a
//! byte-identical result. Sensitivity/actuation/disclosure detection below
//! is a documented **tripwire, not a classifier** — it over-triggers into
//! authorization requests by design (fail-closed), and its hint lists are
//! part of the reviewed surface, not learned behavior.

use super::record::{
    Assumption, AssumptionKind, AuthReason, AuthorizationRequest, AutonomyProfile, CompileError,
    Compiled, ImpactReport, Intake, Mission, MissionChange, ResourceEnvelope, ReviseError,
    default_guardrails, forbidden_actions,
};

/// Substring hints that a goal requires physical actuation. No autonomy
/// profile grants actuation, so any hit fails closed to an authorization
/// request (MASTER_SPEC section 4).
const ACTUATION_HINTS: &[&str] = &[
    "actuat",
    "robot",
    "drone",
    "physical device",
    "motor",
    "valve",
    "surgery",
];
/// Substring hints that a goal requires public disclosure.
const DISCLOSURE_HINTS: &[&str] = &[
    "publish",
    "disclos",
    "press release",
    "public post",
    "announce publicly",
];
/// Substring hints of a sensitive application the compiler must not silently
/// choose (MASTER_SPEC section 4). Over-triggers into authorization requests.
const SENSITIVE_HINTS: &[&str] = &[
    "weapon",
    "pathogen",
    "bioweapon",
    "surveillance of people",
    "targeting individuals",
];
/// Substring hints that a goal requires an irreversible operation
/// (mass deletion, destruction, wiping). Fails closed to an authorization
/// request: reversibility cannot be proven from goal text.
const IRREVERSIBLE_HINTS: &[&str] = &[
    "irreversib",
    "wipe all",
    "delete all",
    "destroy",
    "drop production",
];
/// Substring hints of materially ambiguous risk: unsupervised execution
/// with no human review. The compiler cannot bound the risk, so it asks
/// (R-011) instead of assuming oversight exists.
const RISK_HINTS: &[&str] = &["unsupervised", "no human review", "without oversight"];

fn hit(haystack: &str, hints: &[&str]) -> bool {
    let lower = haystack.to_lowercase();
    hints.iter().any(|h| lower.contains(h))
}

fn request(reason: AuthReason, detail: &str) -> AuthorizationRequest {
    AuthorizationRequest {
        reason,
        detail: detail.to_string(),
    }
}

/// Run every goal-text tripwire and collect each hit in fixed order.
/// Shared by `compile` and the `revise` Goal arm so a revised objective can
/// never smuggle in what fresh compilation refuses.
fn screen_goal(goal: &str) -> Vec<AuthorizationRequest> {
    let mut out = Vec::new();
    if hit(goal, ACTUATION_HINTS) {
        out.push(request(
            AuthReason::ActuationRefused,
            "goal appears to require physical actuation, which no autonomy profile grants",
        ));
    }
    if hit(goal, DISCLOSURE_HINTS) {
        out.push(request(
            AuthReason::ExternalDisclosure,
            "goal appears to require public disclosure, which no autonomy profile grants implicitly",
        ));
    }
    if hit(goal, SENSITIVE_HINTS) {
        out.push(request(
            AuthReason::SensitiveApplication,
            "goal touches a sensitive application; the compiler will not choose it silently",
        ));
    }
    if hit(goal, IRREVERSIBLE_HINTS) {
        out.push(request(
            AuthReason::IrreversibleOperation,
            "goal appears to require an irreversible operation; reversibility cannot be proven from goal text",
        ));
    }
    if hit(goal, RISK_HINTS) {
        out.push(request(
            AuthReason::AmbiguousRisk,
            "goal contemplates unsupervised execution; the compiler cannot bound the risk and will not assume oversight",
        ));
    }
    out
}

/// Build the assumption list from goal text. Terminology is always present;
/// vocabulary, corpus, and subdomain defaults are constructed only when the
/// goal asks for them — never canned unconditionally.
fn assumptions_for(goal: &str) -> Vec<Assumption> {
    let lower = goal.to_lowercase();
    let mut out = vec![Assumption {
        kind: AssumptionKind::Terminology,
        statement: "domain terms are taken from the goal text as written".to_string(),
        provenance: "compiler default; override via revise".to_string(),
    }];
    if lower.contains("vocab") || lower.contains("keyword") {
        out.push(Assumption {
            kind: AssumptionKind::SearchVocabulary,
            statement: "initial search vocabulary drawn from the goal's own terms".to_string(),
            provenance: "compiler default; override via revise".to_string(),
        });
    }
    if lower.contains("corpus") || lower.contains("literature") || lower.contains("papers") {
        out.push(Assumption {
            kind: AssumptionKind::InitialCorpus,
            statement: "initial corpus is the already-authorized local collection".to_string(),
            provenance: "compiler default; override via revise".to_string(),
        });
    }
    if lower.contains("subdomain") || lower.contains("sub-field") || lower.contains("subfield") {
        out.push(Assumption {
            kind: AssumptionKind::TentativeSubdomain,
            statement: "tentative subdomain taken from the goal phrasing".to_string(),
            provenance: "compiler default; override via revise".to_string(),
        });
    }
    out
}

fn permitted_tools(profile: AutonomyProfile) -> Vec<String> {
    match profile {
        // Plan-only: nothing executable.
        AutonomyProfile::PlanOnly => vec![],
        AutonomyProfile::LocalResearch => {
            vec!["corpus.read".to_string(), "retrieval.query".to_string()]
        }
        AutonomyProfile::SandboxExperiments => vec![
            "corpus.read".to_string(),
            "retrieval.query".to_string(),
            "sandbox.exec".to_string(),
        ],
        AutonomyProfile::SupervisedExternal => vec![
            "corpus.read".to_string(),
            "retrieval.query".to_string(),
            "sandbox.exec".to_string(),
            "external.research.supervised".to_string(),
        ],
    }
}

fn permitted_destinations(profile: AutonomyProfile) -> Vec<String> {
    match profile {
        AutonomyProfile::PlanOnly => vec![],
        AutonomyProfile::LocalResearch => vec!["local-corpus".to_string()],
        AutonomyProfile::SandboxExperiments => {
            vec!["local-corpus".to_string(), "sandbox".to_string()]
        }
        AutonomyProfile::SupervisedExternal => vec![
            "local-corpus".to_string(),
            "sandbox".to_string(),
            "approved-external".to_string(),
        ],
    }
}

/// Compile an intake into a Mission or authorization requests.
///
/// Order of checks is load-bearing and documented: structural defects error;
/// actuation/disclosure/sensitivity fail closed; value-frame absence fails
/// closed; only then does a Mission get built.
pub fn compile(intake: &Intake) -> Result<Compiled, CompileError> {
    if intake.goal.trim().is_empty() {
        return Err(CompileError::EmptyGoal);
    }
    if intake.beneficiary.trim().is_empty() {
        return Err(CompileError::EmptyBeneficiary);
    }

    // Goal-text screens accumulate every hit (fixed check order, so twin
    // runs agree): the owner grants the whole set in one round trip.
    let mut requests = screen_goal(&intake.goal);

    // No value frame: the compiler must not substitute its own values.
    if intake.standing_priorities.is_empty() {
        requests.push(request(
            AuthReason::MissingValueFrame,
            "broad goal arrived with no standing priorities or resource profile; supply the owner's value frame",
        ));
    }

    if !intake.resource.spending_approved {
        requests.push(request(
            AuthReason::UnapprovedSpending,
            "resource envelope carries priced work without approved spending",
        ));
    }
    if !requests.is_empty() {
        return Ok(Compiled::NeedsAuthorization(requests));
    }

    let envelope = ResourceEnvelope {
        limit: intake.resource.envelope.limit.clone(),
        pricing_unknown: intake.resource.envelope.pricing_unknown,
    };

    Ok(Compiled::Mission(Box::new(Mission {
        beneficiary: intake.beneficiary.clone(),
        objective: intake.goal.clone(),
        domain_boundaries: vec![format!(
            "as prioritized by: {}",
            intake.standing_priorities.join("; ")
        )],
        constraints: vec!["operate inside the approved workspace and policy".to_string()],
        envelope,
        permitted_tools: permitted_tools(intake.requested_profile),
        permitted_destinations: permitted_destinations(intake.requested_profile),
        forbidden_actions: forbidden_actions(),
        confidentiality: "owner-confidential by default; disclosure needs authorization"
            .to_string(),
        success_metrics: vec![format!(
            "advance the stated objective within guardrails: {}",
            intake.goal
        )],
        guardrails: default_guardrails(),
        evidence_standard:
            "recorded evidence with provenance; valid negative results are legitimate outcomes"
                .to_string(),
        stop_conditions: vec![
            "resource envelope exhausted".to_string(),
            "authorization revoked".to_string(),
            "owner requests stop".to_string(),
        ],
        assumptions: assumptions_for(&intake.goal),
        caller_hypothesis_provenance: intake
            .caller_hypothesis
            .as_ref()
            .map(|_| "caller-supplied; unvalidated; validation is T-016's job".to_string()),
        profile: intake.requested_profile,
        version: 1,
        supersedes: None,
    })))
}

/// Mint a new Mission version from an owner-requested change (R-012).
///
/// The returned `ImpactReport` names exactly the sections that moved, in
/// fixed order, so callers can invalidate dependent plans deterministically.
/// Overriding a requirement that matches nothing errors instead of guessing.
pub fn revise(
    previous: &Mission,
    change: &MissionChange,
) -> Result<(Mission, ImpactReport), ReviseError> {
    let mut next = previous.clone();
    let changed_sections = match change {
        MissionChange::Goal(goal) => {
            if goal.trim().is_empty() {
                return Err(ReviseError::EmptyValue("goal".to_string()));
            }
            // A revised objective must pass the same screens as a fresh
            // goal: otherwise revise could mint the authorization-grade
            // Mission that compile promises never to produce.
            let hits = screen_goal(goal);
            if !hits.is_empty() {
                return Err(ReviseError::NeedsAuthorization(hits));
            }
            next.objective.clone_from(goal);
            next.success_metrics = vec![format!(
                "advance the stated objective within guardrails: {goal}"
            )];
            next.assumptions = assumptions_for(goal);
            vec![
                "objective".to_string(),
                "success_metrics".to_string(),
                "assumptions".to_string(),
            ]
        }
        MissionChange::CostLimit(limit) => {
            next.envelope.limit.clone_from(limit);
            vec!["envelope".to_string()]
        }
        MissionChange::DatasetPolicy(policy) => {
            if policy.trim().is_empty() {
                return Err(ReviseError::EmptyValue("dataset policy".to_string()));
            }
            next.constraints.push(format!("dataset policy: {policy}"));
            vec!["constraints".to_string()]
        }
        MissionChange::QualityMargin(margin) => {
            if margin.trim().is_empty() {
                return Err(ReviseError::EmptyValue("quality margin".to_string()));
            }
            next.constraints.push(format!("quality margin: {margin}"));
            vec!["constraints".to_string()]
        }
        MissionChange::OverrideRequirement {
            requirement,
            replacement,
        } => {
            if replacement.trim().is_empty() {
                return Err(ReviseError::EmptyValue("replacement".to_string()));
            }
            let section =
                if let Some(slot) = next.constraints.iter_mut().find(|c| *c == requirement) {
                    slot.clone_from(replacement);
                    "constraints"
                } else if let Some(slot) = next.guardrails.iter_mut().find(|g| *g == requirement) {
                    slot.clone_from(replacement);
                    "guardrails"
                } else {
                    return Err(ReviseError::UnknownRequirement(requirement.clone()));
                };
            next.assumptions.push(Assumption {
                kind: AssumptionKind::OtherReversible,
                statement: format!("owner override of inferred requirement: {requirement}"),
                provenance: "revise override; supersedes compiler inference".to_string(),
            });
            vec![section.to_string()]
        }
    };
    next.supersedes = Some(previous.version);
    next.version = previous.version + 1;
    Ok((next, ImpactReport { changed_sections }))
}
