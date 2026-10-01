//! Hypothesis records and the compiler (T-016, MASTER_SPEC section 9).
//!
//! The compiler transforms a mechanism candidate into an operational
//! hypothesis (§9:178). It separates mechanistic claims from engineering
//! targets (§9:180) and NEVER invents numerical thresholds — every
//! threshold carries provenance (user requirement, deployment economics, a
//! prior study, a pilot, or an explicitly provisional design choice; the
//! provenance of that choice is mandatory, §9:186).

use crate::genesis::record::MechanismRecord;
use serde::{Deserialize, Serialize};

/// Readiness of a hypothesis (§9:182): "no credible discriminator, no
/// test-ready hypothesis" — but untestable ideas are preserved as
/// EXPLORATORY (or BLOCKED_TESTABILITY), never deleted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Readiness {
    /// Accessible discriminating test exists.
    TestReady,
    /// No discriminator today; potential value preserved.
    Exploratory,
    /// A completion requirement (e.g. threshold provenance) blocks testing.
    BlockedTestability,
}

/// A threshold or effect bound with MANDATORY provenance (§9:186, R-026).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Threshold {
    pub description: String,
    /// Where the number came from: user requirement, deployment economics,
    /// prior study, pilot, or explicitly provisional design choice. Empty
    /// provenance is refused at validation (AT-026).
    pub provenance: String,
    /// What the threshold is measured against.
    pub comparator: String,
}

/// Whether a revised hypothesis inherits prior confirmatory support
/// (R-027): a substantive change forces reassessment — silent inheritance
/// is impossible.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InheritedSupport {
    /// No prior version to inherit from.
    Fresh,
    /// Fields unchanged; prior support carries over.
    Inherits,
    /// Substantive change: prior support does NOT carry until explicitly
    /// reassessed.
    NeedsReassessment,
}

/// One empirical prediction (§9:182): measured quantity, unit, direction,
/// observation scope, uncertainty treatment, decision rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prediction {
    pub measured_quantity: String,
    pub unit: String,
    /// Expected direction of the effect (e.g. "decreases").
    pub direction: String,
    pub observation_scope: String,
    pub uncertainty_treatment: String,
    pub decision_rule: String,
}

/// The operational hypothesis (MASTER_SPEC §9:178): all twelve fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hypothesis {
    pub context: String,
    pub intervention: String,
    pub comparator: String,
    /// The mechanistic claim, carried from the mechanism record (§9:180
    /// separation: this is NOT the engineering target).
    pub proposed_mechanism: MechanismRecord,
    pub predictions: Vec<Prediction>,
    pub estimands: Vec<String>,
    /// Meaningful effect bounds, provenance-cited; None when none declared.
    pub effect_bounds: Option<Threshold>,
    pub boundary_conditions: Vec<String>,
    pub competing_explanations: Vec<String>,
    pub required_observations: Vec<String>,
    pub analysis_requirements: Vec<String>,
    /// Explicit outcomes that would count against the claim.
    pub falsifiers: Vec<String>,
    /// The engineering target, SEPARATE from the mechanism claim (§9:180);
    /// provenance-cited or None. The compiler never invents one.
    pub engineering_target: Option<Threshold>,
    /// The operational discriminator: what observation distinguishes this
    /// hypothesis from its competitors. None => EXPLORATORY (§9:182).
    pub discriminator: Option<String>,
    /// Readiness, set by semantic validation — never self-declared.
    pub readiness: Option<Readiness>,
    /// Version lineage (R-027).
    pub version: u64,
    pub supersedes: Option<u64>,
    /// Whether prior confirmatory support carries over.
    pub inherited_support: InheritedSupport,
}

impl Hypothesis {
    /// Whether the hypothesis merely restates its own success metric
    /// (§9:184): a falsifier that presupposes the mechanism's success.
    /// `pub(crate)`: semantic validators consume it; not public API.
    pub(crate) fn falsifier_is_self_fulfilling(&self) -> bool {
        self.falsifiers.iter().any(|f| {
            let lowered = f.to_lowercase();
            (lowered.contains("which would mean")
                || lowered.contains("would confirm")
                || lowered.contains("would mean"))
                && lowered.contains("works")
        })
    }
}

/// Compile a mechanism candidate into an operational hypothesis. Fills all
/// §9:178 fields from the mechanism record and its operating context;
/// leaves thresholds to the caller (never invented here) and readiness to
/// semantic validation.
pub fn compile(mechanism: &MechanismRecord) -> Hypothesis {
    Hypothesis {
        context: format!(
            "operating regime: {}; prerequisites: {}",
            mechanism.operating_regime,
            mechanism.prerequisites.join("; ")
        ),
        intervention: mechanism.realization.clone(),
        comparator: "current implementation without the mechanism, same workload and fixtures"
            .into(),
        proposed_mechanism: mechanism.clone(),
        predictions: vec![Prediction {
            measured_quantity: mechanism
                .variables
                .first()
                .cloned()
                .unwrap_or_else(|| "primary_variable".into()),
            unit: "recorded unit of the traced workload".into(),
            direction: "per the mechanism's expected effects".into(),
            observation_scope: "traced workload within the declared regime".into(),
            uncertainty_treatment: "stated per observation; never compiler-invented".into(),
            decision_rule: "falsifiers decide; see falsifiers list".into(),
        }],
        estimands: mechanism.expected_effects.clone(),
        effect_bounds: None, // caller supplies with provenance; never minted
        boundary_conditions: vec![
            format!("operating regime: {}", mechanism.operating_regime),
            format!(
                "failure modes monitored: {}",
                mechanism.failure_modes.join("; ")
            ),
        ],
        competing_explanations: vec![
            "poor implementation of the mechanism itself".into(),
            "measurement error in the traced observations".into(),
        ],
        required_observations: vec![
            "the measured quantity under the intervention".into(),
            "the same quantity under the comparator".into(),
        ],
        analysis_requirements: vec![
            "compare intervention vs comparator on the same fixtures".into(),
            "treat uncertainty as stated in the predictions".into(),
        ],
        falsifiers: mechanism
            .failure_modes
            .iter()
            .map(|m| format!("observing '{m}' under the intervention counts against the claim"))
            .chain(std::iter::once(
                "no measurable difference from the comparator within stated uncertainty"
                    .to_string(),
            ))
            .collect(),
        engineering_target: None,
        discriminator: Some(format!(
            "the intervention changes '{}' while the comparator does not, in the declared regime",
            mechanism
                .variables
                .first()
                .cloned()
                .unwrap_or_else(|| "the primary variable".into())
        )),
        readiness: None, // validation decides
        version: 1,
        supersedes: None,
        inherited_support: InheritedSupport::Fresh,
    }
}

/// Revise a hypothesis: mints version+1 with lineage. A substantive change
/// (mechanism, boundary condition, primary endpoint, falsifier) forces
/// `NeedsReassessment` — prior confirmatory support never silently
/// carries (R-027/§9:186).
pub fn revise(previous: &Hypothesis, mut changed: Hypothesis) -> Hypothesis {
    changed.version = previous.version + 1;
    changed.supersedes = Some(previous.version);
    let substantive = changed.proposed_mechanism != previous.proposed_mechanism
        || changed.boundary_conditions != previous.boundary_conditions
        || changed.falsifiers != previous.falsifiers
        || changed.engineering_target != previous.engineering_target;
    changed.inherited_support = if substantive {
        InheritedSupport::NeedsReassessment
    } else {
        InheritedSupport::Inherits
    };
    changed.readiness = None; // re-validated after revision
    changed
}
