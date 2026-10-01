//! Semantic validators for hypotheses (T-016, MASTER_SPEC §9:184, R-025).
//!
//! Validation asks the compiler-check questions: could plausible data
//! support, contradict, or leave this claim unresolved? Are the test's
//! outcomes meaningfully different under the proposed and competing
//! mechanisms? Are key variables measurable? Is the comparator relevant?
//! Are the preconditions realizable? Does the hypothesis merely restate
//! its own success metric?
//!
//! Denial is structured and auditable: the hypothesis is NEVER deleted —
//! readiness is downgraded to Exploratory (no discriminator: §9:182) or
//! BlockedTestability (completion required, e.g. threshold provenance).

use super::hypothesis::{Hypothesis, InheritedSupport, Readiness};
use serde::{Deserialize, Serialize};

/// The verdict: readiness plus structured denial reasons, each naming the
/// §9:184 question it answers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Verdict {
    pub readiness: Readiness,
    pub denials: Vec<String>,
}

/// Validate a hypothesis and assign readiness. The hypothesis is never
/// deleted: TestReady requires everything; a missing discriminator yields
/// Exploratory (§9:182); a completion requirement (threshold provenance)
/// yields BlockedTestability.
pub fn validate(h: &Hypothesis) -> Verdict {
    let mut denials: Vec<String> = Vec::new();

    // §9:184: "explicit outcomes that would count against the claim".
    if h.falsifiers.is_empty() {
        denials.push(
            "falsifiers required: a hypothesis without outcomes that count against the claim cannot be tested (AT-025)"
                .into(),
        );
    }
    // §9:178: competing explanations; AT-025 strips them.
    if h.competing_explanations.is_empty() {
        denials.push(
            "competing explanations required: without them the test cannot distinguish mechanisms (AT-025)"
                .into(),
        );
    }
    // §9:178: comparator; AT-026 negative case.
    if h.comparator.trim().is_empty() {
        denials.push(
            "comparator required: an arbitrary target without a reason or comparator is returned for completion, never accepted as justified (AT-026)"
                .into(),
        );
    }
    // §9:182: operational discriminator.
    let has_discriminator = h
        .discriminator
        .as_ref()
        .map(|d| !d.trim().is_empty())
        .unwrap_or(false);
    if !has_discriminator {
        denials.push(
            "operational discriminator required: no credible discriminator, no test-ready hypothesis (MASTER_SPEC:182)"
                .into(),
        );
    }
    // §9:184: "Does the hypothesis merely restate its own success metric?"
    if h.falsifier_is_self_fulfilling() {
        denials.push(
            "self-fulfilling falsifier: a falsifier that restates the hypothesis's own success metric decides nothing (MASTER_SPEC:184)"
                .into(),
        );
    }
    // §9:186 / AT-026: thresholds and effect bounds carry provenance; the
    // compiler never invents numbers.
    for t in [&h.effect_bounds, &h.engineering_target]
        .into_iter()
        .flatten()
    {
        if t.provenance.trim().is_empty() || t.comparator.trim().is_empty() {
            denials.push(
                "threshold provenance required: every threshold/effect bound cites its source (user requirement, prior study, pilot, or explicitly provisional choice) and its comparator (AT-026)"
                    .into(),
            );
            break;
        }
    }
    // R-027: substantively changed hypotheses cannot promote on inherited
    // support that has not been reassessed.
    if h.inherited_support == InheritedSupport::NeedsReassessment {
        denials.push(
            "inherited support needs reassessment: a substantive change (mechanism, boundary, endpoint, falsifier) invalidates silent inheritance (R-027)"
                .into(),
        );
    }

    let readiness = if !has_discriminator {
        // §9:182: preserved as EXPLORATORY, never deleted.
        Readiness::Exploratory
    } else if denials.iter().any(|d| d.contains("provenance")) {
        Readiness::BlockedTestability
    } else if denials.is_empty() {
        Readiness::TestReady
    } else {
        // Denied for substance (falsifiers/competitors/self-fulfilling):
        // stays Exploratory until completed — preserved, not deleted.
        Readiness::Exploratory
    };
    Verdict { readiness, denials }
}
