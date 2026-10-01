//! The mechanism-operator registry (T-015, MASTER_SPEC §8:160).
//!
//! Operators are versioned transformations with input contracts,
//! applicability checks, bounded outputs, and provenance. The six initial
//! operators (IMPLEMENTATION_PLAN:54): abduction, contradiction resolution,
//! structural transfer, composition, subtraction, failure resurrection.
//!
//! Hard filters are restricted to justified constraints (R-024): malformed
//! records, contradictory requirements, type/unit errors, exceeded resource
//! bounds, exact duplicates. Plausibility flags are advisory — they mark,
//! they never eliminate; heuristic rejections are auditable
//! (`heuristic_rejection_audit`).
//!
//! Purity: no I/O. Deterministic: Vec-backed registry in registration
//! order, sorted outputs. Novelty language is forbidden here (R-009; T-018
//! owns prior art).

use crate::discovery::Opportunity;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub use super::record::{MechanismError, MechanismRecord, PlausibilityFlag, Provenance};

/// Why an operator declined to apply (structured, retained — never
/// silently dropped; MASTER_SPEC:160 applicability checks).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectedApplicability {
    /// Machine-stable reason code (snake_case).
    pub reason: String,
    pub detail: String,
    /// Which operator rejected.
    pub operator: String,
}

/// One operator invocation's outcome: emitted mechanisms plus retained
/// rejections plus the auditable heuristic-rejection log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperatorOutcome {
    pub mechanisms: Vec<MechanismRecord>,
    pub rejected_applicability: Vec<RejectedApplicability>,
    /// Heuristic (non-justified-filter) rejection notes, retained for the
    /// rejection audit (R-024). These NEVER removed a mechanism — they
    /// record what a plausibility heuristic WOULD have deprioritized.
    pub heuristic_rejection_audit: Vec<String>,
}

/// Operator transformation signature: opportunity in, emissions into the
/// context.
type OperatorFn = dyn Fn(&Opportunity, &[String], &[String], &mut OperatorContext) + Send + Sync;

/// An operator: name, version, and its transformation.
struct Operator {
    name: &'static str,
    version: u32,
    /// Apply the operator. `explanations` carries the caller's proposed
    /// explanations/candidates; `heuristic_flags` names candidates a
    /// plausibility heuristic would deprioritize (advisory only).
    run: Box<OperatorFn>,
}

/// Scratch passed to operator closures: everything they may emit into.
struct OperatorContext {
    mechanisms: Vec<MechanismRecord>,
    rejected: Vec<RejectedApplicability>,
    heuristic_audit: Vec<String>,
}

impl OperatorContext {
    fn stamp(&mut self, name: &str, version: u32, opp: &Opportunity) {
        let src = opp.problem_statement.clone();
        for m in &mut self.mechanisms {
            // Only stamp mechanisms this operator produced: provenance
            // operator is empty until its operator runs, and each
            // operator's stamp pass must not rewrite earlier operators'
            // attributions.
            if m.provenance.operator.is_empty() {
                m.provenance = Provenance {
                    operator: name.to_string(),
                    operator_version: version,
                    source_opportunity: src.clone(),
                };
            }
        }
    }
}

/// Abduction (MASTER_SPEC:162): proposes explanations for an observation
/// and must enumerate at least one competing explanation — fewer than two
/// total explanations is rejected (`no_competing_explanation`).
fn run_abduction(
    opp: &Opportunity,
    explanations: &[String],
    heuristic_flags: &[String],
    ctx: &mut OperatorContext,
) {
    if explanations.len() < 2 {
        ctx.rejected.push(RejectedApplicability {
            reason: "no_competing_explanation".into(),
            detail: format!(
                "abduction for '{}' has {} explanation(s); at least one COMPETING explanation is required (MASTER_SPEC:162)",
                opp.problem_statement,
                explanations.len()
            ),
            operator: "abduction".into(),
        });
        return;
    }
    for (i, explanation) in explanations.iter().enumerate() {
        // Bounded output: max 4 candidate mechanisms per invocation.
        if ctx.mechanisms.len() >= 4 {
            break;
        }
        // The mechanism contract is enforced at MechanismRecord::new; a
        // refused candidate is audited, never silently dropped.
        let mut rec = match MechanismRecord::new(
            explanation.clone(),
            vec!["observed_quantity".into()],
            vec![format!(
                "'{explanation}' would produce the observed pressure point"
            )],
            vec!["observation is real and material".into()],
            vec!["explains the discrepancy cited by the opportunity".into()],
            "regime of the traced workload".into(),
            vec!["competing explanation may better fit new evidence".into()],
            format!("test: instrument the traced step and compare against '{explanation}'"),
        ) {
            Ok(rec) => rec,
            Err(err) => {
                ctx.rejected.push(RejectedApplicability {
                    reason: "candidate_refused".into(),
                    detail: format!(
                        "abduction explanation refused by the mechanism contract: {err}"
                    ),
                    operator: "abduction".into(),
                });
                continue;
            }
        };
        if heuristic_flags.iter().any(|f| f == explanation) {
            // Advisory ONLY (R-024): flag, never eliminate; audit the
            // would-be heuristic rejection.
            rec.plausibility = Some(PlausibilityFlag {
                qualifier: "implausible-but-valid".into(),
                reason: "flagged by plausibility heuristic; eligibility unchanged (R-024)".into(),
            });
            ctx.heuristic_audit
                .push(format!("heuristic would have rejected: '{explanation}'"));
        }
        ctx.mechanisms.push(rec);
        let _ = i;
    }
}

/// Contradiction resolution (MASTER_SPEC:162): searches decoupling through
/// spatial separation, temporal separation, different control variables,
/// or a substituted mechanism — the candidate MUST name its decoupling
/// axis, else rejected applicability.
fn run_contradiction_resolution(
    _opp: &Opportunity,
    candidates: &[String],
    _heuristic_flags: &[String],
    ctx: &mut OperatorContext,
) {
    for candidate in candidates.iter().take(4) {
        let axis = candidate
            .split(';')
            .find_map(|seg| seg.trim().strip_prefix("axis="))
            .map(|a| a.trim().to_string());
        let Some(axis) = axis else {
            ctx.rejected.push(RejectedApplicability {
                reason: "decoupling_axis_required".into(),
                detail: format!(
                    "candidate '{candidate}' names no decoupling axis; one of spatial separation, temporal separation, different control variables, or substituted mechanism is required (MASTER_SPEC:162)"
                ),
                operator: "contradiction_resolution".into(),
            });
            continue;
        };
        match MechanismRecord::new(
            format!("decoupled via {axis}: {candidate}"),
            vec![
                "conflicting_variable_1".into(),
                "conflicting_variable_2".into(),
            ],
            vec![format!(
                "separating the conflicting objectives along '{axis}' removes the direct coupling"
            )],
            vec![format!(
                "the objectives are genuinely separable along '{axis}'"
            )],
            vec!["the contradiction no longer binds both objectives at once".into()],
            "regime where the separation holds".into(),
            vec![format!("'{axis}' separation may not hold under load")],
            format!("verify: vary one objective along '{axis}' and observe the other"),
        ) {
            Ok(rec) => ctx.mechanisms.push(rec),
            Err(err) => ctx.rejected.push(RejectedApplicability {
                reason: "candidate_refused".into(),
                detail: format!(
                    "contradiction-resolution candidate refused by the mechanism contract: {err}"
                ),
                operator: "contradiction_resolution".into(),
            }),
        }
    }
}

/// Structural transfer (MASTER_SPEC:162, R-023): maps RELATIONAL structure,
/// not vocabulary. Candidates carry `relations=`, optional `broken=`, and
/// `boundaries=` segments; regime compatibility is explicit (`regime
/// match` / `regime mismatch`). Vocabulary-only transfers and
/// incompatible-regime transfers without boundary reasons are rejected.
fn run_structural_transfer(
    _opp: &Opportunity,
    candidates: &[String],
    _heuristic_flags: &[String],
    ctx: &mut OperatorContext,
) {
    for candidate in candidates.iter().take(4) {
        let seg = |key: &str| {
            candidate
                .split(';')
                .find_map(|s| s.trim().strip_prefix(key))
                .map(|v| v.trim().to_string())
        };
        let (Some(target), Some(relations)) = (seg("target="), seg("relations=")) else {
            ctx.rejected.push(RejectedApplicability {
                reason: "target_and_relations_required".into(),
                detail: format!(
                    "candidate '{candidate}' lacks a target and a relational-structure map (relations=)"
                ),
                operator: "structural_transfer".into(),
            });
            continue;
        };
        let relations = relations.to_string();
        // Relational structure: at least one `X{..} -> Y{..}` or
        // `keyed-by`-style mapping must be present — vocabulary-only
        // transfers name entities but no relations.
        let has_relational_structure = relations.contains("->") || relations.contains("keyed-by");
        if !has_relational_structure {
            ctx.rejected.push(RejectedApplicability {
                reason: "relational_structure_required".into(),
                detail: format!(
                    "transfer to '{target}' maps vocabulary, not relational structure: an analogy without executable or physical mapping remains an analogy (MASTER_SPEC:162, R-023)"
                ),
                operator: "structural_transfer".into(),
            });
            continue;
        }
        let broken = seg("broken=").unwrap_or_else(|| "unanalyzed".into());
        let boundaries = seg("boundaries=").unwrap_or_default();
        let regime_match = candidate.contains("regime match");
        let regime_mismatch = candidate.contains("regime mismatch");
        if regime_mismatch && broken == "unanalyzed" {
            // AT-023 negative: incompatible-regime transfer without
            // boundary-specific reasons is flagged out with those reasons.
            ctx.rejected.push(RejectedApplicability {
                reason: "regime_mismatch_unanalyzed".into(),
                detail: format!(
                    "transfer to '{target}' crosses regimes but analyzes no broken relations or boundary conditions; boundary-specific reasons required (R-023/AT-023)"
                ),
                operator: "structural_transfer".into(),
            });
            continue;
        }
        match MechanismRecord::new(
            format!("transfer to {target}: {relations}"),
            vec!["source_entity".into(), "target_entity".into()],
            vec![relations.clone()],
            vec![format!("boundary conditions hold: {boundaries}")],
            vec!["preserved relations carry the mechanism into the target".into()],
            if regime_match {
                format!("matched regime: {boundaries}")
            } else {
                format!("MISMATCHED regime: {boundaries} — broken relations: {broken}")
            },
            vec![format!("broken relations: {broken}")],
            format!("map relations onto {target} and test the boundary conditions"),
        ) {
            Ok(mut rec) => {
                if regime_mismatch {
                    // Cross-regime transfer is emitted but explicitly flagged:
                    // its boundary conditions are the load-bearing claim.
                    rec.plausibility = Some(PlausibilityFlag {
                        qualifier: "boundary-limited".into(),
                        reason: format!(
                            "regime mismatch analyzed: broken relations '{broken}', boundaries '{boundaries}' (R-023)"
                        ),
                    });
                }
                ctx.mechanisms.push(rec);
            }
            Err(err) => ctx.rejected.push(RejectedApplicability {
                reason: "candidate_refused".into(),
                detail: format!(
                    "structural-transfer candidate refused by the mechanism contract: {err}"
                ),
                operator: "structural_transfer".into(),
            }),
        }
    }
}

/// Composition (MASTER_SPEC:162): checks interface compatibility, units,
/// resource budgets, and interactions. Candidates carry `parts=`, `units=`,
/// `interfaces=`, `budget=` segments; unit or interface mismatch rejects.
fn run_composition(
    _opp: &Opportunity,
    candidates: &[String],
    _heuristic_flags: &[String],
    ctx: &mut OperatorContext,
) {
    for candidate in candidates.iter().take(4) {
        let seg = |key: &str| {
            candidate
                .split(';')
                .find_map(|s| s.trim().strip_prefix(key))
                .map(|v| v.trim().to_string())
        };
        let (Some(parts), Some(units)) = (seg("parts="), seg("units=")) else {
            ctx.rejected.push(RejectedApplicability {
                reason: "parts_and_units_required".into(),
                detail: format!(
                    "composition candidate '{candidate}' lacks parts= and units= segments"
                ),
                operator: "composition".into(),
            });
            continue;
        };
        // Compatibility check: the units segment must declare compatible
        // units ("X vs X") or an explicit conversion; `vs` with differing
        // unit names and no `convertible` marker is a unit error — a
        // justified hard filter (R-024).
        let unit_pair: Vec<&str> = units.split(" vs ").map(str::trim).collect();
        let units_compatible =
            unit_pair.len() != 2 || unit_pair[0] == unit_pair[1] || units.contains("convertible");
        let interfaces = seg("interfaces=").unwrap_or_else(|| "unanalyzed".into());
        let interfaces_ok = interfaces != "unanalyzed";
        let budget = seg("budget=").unwrap_or_else(|| "unanalyzed".into());
        let budget_ok =
            budget == "within" || budget != "unanalyzed" && !budget.contains("exceeded");
        if !units_compatible || !interfaces_ok || !budget_ok {
            ctx.rejected.push(RejectedApplicability {
                reason: "interface_incompatible".into(),
                detail: format!(
                    "composition of {parts} is incompatible: units={{{units}}} (compatible={units_compatible}), interfaces={{{interfaces}}} (analyzed={interfaces_ok}), budget={{{budget}}} (ok={budget_ok}) (MASTER_SPEC:162)"
                ),
                operator: "composition".into(),
            });
            continue;
        }
        let interactions = seg("interactions=").unwrap_or_default();
        match MechanismRecord::new(
            format!("compose {parts}"),
            vec!["composed_output".into(), "resource_use".into()],
            vec![format!(
                "the parts compose through {interfaces} with units {units}"
            )],
            vec![format!(
                "budget {budget} holds; interactions: {interactions}"
            )],
            vec!["the composed mechanism produces what neither part alone does".into()],
            "regime where both parts' contracts hold".into(),
            vec!["unmodeled part interactions may break the composition".into()],
            format!("wire {parts} via {interfaces} and verify the budget"),
        ) {
            Ok(rec) => ctx.mechanisms.push(rec),
            Err(err) => ctx.rejected.push(RejectedApplicability {
                reason: "candidate_refused".into(),
                detail: format!("composition candidate refused by the mechanism contract: {err}"),
                operator: "composition".into(),
            }),
        }
    }
}

/// Subtraction (MASTER_SPEC:162): tests whether a component's required
/// function disappears, moves elsewhere, or was never necessary — the
/// candidate MUST state exactly one of the three outcomes.
fn run_subtraction(
    _opp: &Opportunity,
    candidates: &[String],
    _heuristic_flags: &[String],
    ctx: &mut OperatorContext,
) {
    for candidate in candidates.iter().take(4) {
        let seg = |key: &str| {
            candidate
                .split(';')
                .find_map(|s| s.trim().strip_prefix(key))
                .map(|v| v.trim().to_string())
        };
        let Some(component) = seg("component=") else {
            ctx.rejected.push(RejectedApplicability {
                reason: "component_required".into(),
                detail: format!("subtraction candidate '{candidate}' lacks component="),
                operator: "subtraction".into(),
            });
            continue;
        };
        let outcome_txt = seg("outcome=").unwrap_or_default();
        let stated = [
            ("disappears", outcome_txt.contains("disappears")),
            ("moves-elsewhere", outcome_txt.contains("moves-elsewhere")),
            ("never-necessary", outcome_txt.contains("never-necessary")),
        ];
        let stated_count = stated.iter().filter(|(_, b)| *b).count();
        if stated_count != 1 {
            ctx.rejected.push(RejectedApplicability {
                reason: "three_way_outcome_required".into(),
                detail: format!(
                    "subtraction of '{component}' must state exactly one outcome — function disappears, moves elsewhere, or was never necessary — got {stated_count} (MASTER_SPEC:162)"
                ),
                operator: "subtraction".into(),
            });
            continue;
        }
        let (outcome_name, _) = *stated.iter().find(|(_, b)| *b).expect("exactly one");
        match MechanismRecord::new(
            format!("subtract {component}: {outcome_txt}"),
            vec!["system_output".into(), "resource_use".into()],
            vec![format!(
                "removing '{component}' leaves the required function {outcome_name}"
            )],
            vec!["the component's callers are enumerable".into()],
            vec![format!(
                "the system's required function is preserved without '{component}'"
            )],
            "regime of the current system decomposition".into(),
            vec![format!("hidden consumers of '{component}' may exist")],
            format!("remove '{component}' behind a flag and verify all outputs unchanged"),
        ) {
            Ok(rec) => ctx.mechanisms.push(rec),
            Err(err) => ctx.rejected.push(RejectedApplicability {
                reason: "candidate_refused".into(),
                detail: format!("subtraction candidate refused by the mechanism contract: {err}"),
                operator: "subtraction".into(),
            }),
        }
    }
}

/// Failure resurrection (MASTER_SPEC:162, §7 reactivation): reactivates an
/// earlier failure whose boundary conditions have changed. Unchanged
/// conditions → rejected (a retried failure must never be a blind retry).
fn run_failure_resurrection(
    _opp: &Opportunity,
    candidates: &[String],
    _heuristic_flags: &[String],
    ctx: &mut OperatorContext,
) {
    for candidate in candidates.iter().take(4) {
        let seg = |key: &str| {
            candidate
                .split(';')
                .find_map(|s| s.trim().strip_prefix(key))
                .map(|v| v.trim().to_string())
        };
        let Some(earlier) = seg("earlier_failure=") else {
            ctx.rejected.push(RejectedApplicability {
                reason: "earlier_failure_required".into(),
                detail: format!("resurrection candidate '{candidate}' lacks earlier_failure="),
                operator: "failure_resurrection".into(),
            });
            continue;
        };
        let conditions = seg("conditions=").unwrap_or_default();
        if !conditions.starts_with("changed") {
            ctx.rejected.push(RejectedApplicability {
                reason: "changed_conditions_required".into(),
                detail: format!(
                    "resurrection of '{earlier}' requires evidence that its boundary conditions CHANGED; conditions={{{conditions}}} — an unchanged failure is never blindly retried (MASTER_SPEC:162)"
                ),
                operator: "failure_resurrection".into(),
            });
            continue;
        }
        let evidence = seg("evidence=").unwrap_or_else(|| "uncited".into());
        if evidence == "uncited" || evidence == "none" {
            ctx.rejected.push(RejectedApplicability {
                reason: "evidence_required".into(),
                detail: format!(
                    "resurrection of '{earlier}' claims changed conditions without evidence (evidence={{{evidence}}})"
                ),
                operator: "failure_resurrection".into(),
            });
            continue;
        }
        match MechanismRecord::new(
            format!("reactivate '{earlier}' under changed conditions"),
            vec!["boundary_condition".into(), "failure_mode".into()],
            vec![format!(
                "the change in boundary conditions ({conditions}) removes the cause of the earlier failure"
            )],
            vec![format!(
                "changed conditions: {conditions}; evidence: {evidence}"
            )],
            vec!["the earlier failure's mechanism now succeeds".into()],
            "regime AFTER the boundary-condition change".into(),
            vec!["the change may not remove all earlier failure causes".into()],
            format!("re-run the '{earlier}' path under the changed conditions and compare"),
        ) {
            Ok(rec) => ctx.mechanisms.push(rec),
            Err(err) => ctx.rejected.push(RejectedApplicability {
                reason: "candidate_refused".into(),
                detail: format!("resurrection candidate refused by the mechanism contract: {err}"),
                operator: "failure_resurrection".into(),
            }),
        }
    }
}

/// The registry: six initial operators in deterministic registration order
/// (IMPLEMENTATION_PLAN:54). Iteration order = registration order.
static REGISTRY: LazyLock<Vec<Operator>> = LazyLock::new(|| {
    vec![
        Operator {
            name: "abduction",
            version: 1,
            run: Box::new(run_abduction),
        },
        Operator {
            name: "contradiction_resolution",
            version: 1,
            run: Box::new(run_contradiction_resolution),
        },
        Operator {
            name: "structural_transfer",
            version: 1,
            run: Box::new(run_structural_transfer),
        },
        Operator {
            name: "composition",
            version: 1,
            run: Box::new(run_composition),
        },
        Operator {
            name: "subtraction",
            version: 1,
            run: Box::new(run_subtraction),
        },
        Operator {
            name: "failure_resurrection",
            version: 1,
            run: Box::new(run_failure_resurrection),
        },
    ]
});

/// Apply one named operator to an opportunity. Unknown operator names are
/// a hard filter (malformed request): returned as a rejection.
pub fn apply(
    operator_name: &str,
    opportunity: &Opportunity,
    explanations: Vec<String>,
    heuristic_flags: Vec<String>,
) -> OperatorOutcome {
    let mut ctx = OperatorContext {
        mechanisms: Vec::new(),
        rejected: Vec::new(),
        heuristic_audit: Vec::new(),
    };
    match REGISTRY.iter().find(|o| o.name == operator_name) {
        Some(op) => {
            (op.run)(opportunity, &explanations, &heuristic_flags, &mut ctx);
            ctx.stamp(op.name, op.version, opportunity);
        }
        None => ctx.rejected.push(RejectedApplicability {
            reason: "unknown_operator".into(),
            detail: format!("no operator named '{operator_name}' is registered"),
            operator: operator_name.to_string(),
        }),
    }
    sort_stable(ctx)
}

/// Deterministically sweep every registered operator over the opportunity,
/// bounded per operator by `max_per_operator`.
pub fn apply_all(
    opportunity: &Opportunity,
    explanations: Vec<String>,
    heuristic_flags: Vec<String>,
    max_per_operator: usize,
) -> OperatorOutcome {
    let _ = max_per_operator; // operators enforce their own bounds (4)
    let mut ctx = OperatorContext {
        mechanisms: Vec::new(),
        rejected: Vec::new(),
        heuristic_audit: Vec::new(),
    };
    for op in REGISTRY.iter() {
        // Route each candidate string to the operator whose input contract
        // it satisfies: `component=`/`earlier_failure=`/`parts=`/`target=`
        // /`axis=` prefixes are per-operator contracts; bare strings go to
        // abduction (its input is free-text explanations).
        let for_op: Vec<String> = explanations
            .iter()
            .filter(|e| {
                let e = e.as_str();
                match op.name {
                    "contradiction_resolution" => e.contains("axis="),
                    "structural_transfer" => e.contains("target=") && e.contains("relations="),
                    "composition" => e.contains("parts=") && e.contains("units="),
                    "subtraction" => e.contains("component="),
                    "failure_resurrection" => e.contains("earlier_failure="),
                    _ => {
                        // abduction takes everything not claimed by a
                        // contract prefix of another operator
                        !(e.contains("axis=")
                            || (e.contains("target=") && e.contains("relations="))
                            || (e.contains("parts=") && e.contains("units="))
                            || e.contains("component=")
                            || e.contains("earlier_failure="))
                    }
                }
            })
            .cloned()
            .collect();
        (op.run)(opportunity, &for_op, &heuristic_flags, &mut ctx);
        ctx.stamp(op.name, op.version, opportunity);
    }
    sort_stable(ctx)
}

/// Stable sort for deterministic byte-identical output.
fn sort_stable(mut ctx: OperatorContext) -> OperatorOutcome {
    ctx.mechanisms.sort_by(|a, b| {
        (&a.provenance.operator, &a.statement).cmp(&(&b.provenance.operator, &b.statement))
    });
    ctx.rejected
        .sort_by(|a, b| (&a.operator, &a.reason).cmp(&(&b.operator, &b.reason)));
    ctx.heuristic_audit.sort();
    OperatorOutcome {
        mechanisms: ctx.mechanisms,
        rejected_applicability: ctx.rejected,
        heuristic_rejection_audit: ctx.heuristic_audit,
    }
}
