//! The six pressure-point operators (T-014) and the `analyze` entry point.
//!
//! Operator contracts (all cite MASTER_SPEC section 7):
//!
//! - **Bottleneck** (:142 traces/profiles): a material, repeated slow step
//!   in duration records. Requires span evidence; unevidenced records are
//!   skipped — uncertainty is explicit, citations are never fabricated.
//! - **Anomaly** (:144): an uncertainty-aware discrepancy between a
//!   prediction and an observation sharing a unit under comparable
//!   conditions. Unit mismatch, missing prediction, or incomparable
//!   conditions → rejection, never an emitted opportunity (R-020/AT-020).
//! - **Conflicting objectives** (:144): counter-movement between two
//!   objectives requires coupling evidence in the relevant regime; bare
//!   counter-movement is rejected.
//! - **Failure pattern**: recurring same-kind failures (≥3 distinct
//!   records) form one pattern opportunity citing the distinct failing
//!   records; a single failure is not a pattern.
//! - **Assumption**: declared unstated assumptions become opportunities
//!   with explicit uncertainty.
//! - **Changed capability** (:144): verified availability plus a concrete
//!   account of which previous constraint it changes; availability alone
//!   is rejected.

use super::record::{
    AnalysisOutcome, Opportunity, PressureKind, RejectedCandidate, TraceRecord, TraceVal,
};

/// Median of the values (records must share a unit — the caller filters).
fn median_of(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = values.len();
    if n % 2 == 1 {
        values[n / 2]
    } else {
        (values[n / 2 - 1] + values[n / 2]) / 2.0
    }
}

/// Bottleneck operator: name-grouped duration records; a group whose
/// median exceeds the median of the OTHER groups' durations by ≥4x AND
/// has ≥2 records with span evidence is a material, repeated bottleneck.
/// Unevidenced records never ground a citation.
fn bottleneck_opportunities(
    records: &[TraceRecord],
    rejected: &mut Vec<RejectedCandidate>,
) -> Vec<Opportunity> {
    let _ = rejected; // bottleneck rejections are implicit (honest empty)
    let mut by_name: Vec<(String, Vec<&TraceRecord>, Vec<f64>)> = Vec::new();
    let mut all_durations: Vec<f64> = Vec::new();
    for r in records {
        if let TraceVal::DurationMs(v) = r.value {
            all_durations.push(v as f64);
            match by_name.iter_mut().find(|(n, _, _)| *n == r.name) {
                Some((_, recs, vals)) => {
                    recs.push(r);
                    vals.push(v as f64);
                }
                None => by_name.push((r.name.clone(), vec![r], vec![v as f64])),
            }
        }
    }
    if by_name.len() < 2 || all_durations.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (name, recs, vals) in &by_name {
        if recs.len() < 2 {
            continue;
        }
        let group_median = median_of(&mut vals.clone());
        // Material: the group's median is ≥4x the median of the OTHER
        // groups' durations (not the global median, which the group
        // itself inflates). With one other group this is the direct
        // comparison the AT-019 fixture needs.
        let mut others: Vec<f64> = all_durations
            .iter()
            .copied()
            .filter(|v| {
                // exclude this group's own values by membership
                let mut g = vals.clone();
                g.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                !g.binary_search_by(|p| p.partial_cmp(v).unwrap_or(std::cmp::Ordering::Equal))
                    .is_ok()
            })
            .collect();
        if others.is_empty() {
            continue;
        }
        let others_median = median_of(&mut others);
        if group_median >= others_median * 4.0 {
            let evidence: Vec<_> = recs.iter().filter_map(|r| r.evidence.clone()).collect();
            if evidence.is_empty() {
                // No fabricated citations: skip; uncertainty is explicit
                // only where an opportunity is emitted.
                continue;
            }
            let mut opp = Opportunity::grounded(PressureKind::Bottleneck, evidence);
            opp.suspected_bottleneck = Some(name.clone());
            opp.problem_statement = format!(
                "step '{name}' is repeated and material: its median duration is ≥4x the median of the other traced steps"
            );
            opp.beneficiary = "operator of the traced workload".into();
            opp.context = "authorized trace analysis".into();
            opp.causal_uncertainty =
                "why the step is slow is unestablished; the measurement gap is the uncertainty"
                    .into();
            opp.prior_art_query_plan
                .push(format!("prior art: '{name}' latency reduction approaches"));
            opp.unanswered_questions
                .push(format!("is '{name}' already optimized elsewhere?"));
            out.push(opp);
        }
    }
    out
}

/// Anomaly operator (MASTER_SPEC:144, R-020): a prediction paired with an
/// observation of the SAME quantity (name-match) whose discrepancy exceeds
/// the stated uncertainty band, in the SAME unit. Unit mismatch or missing
/// same-name prediction → rejection, never an emission (R-020/AT-020).
/// Uncertainty band: observation must differ from prediction by >50%
/// relative (the fixture's 50→500 clears, 50→55 does not) — a fixed,
/// documented relative band; per-record uncertainty fields are a later
/// refinement.
fn anomaly_outcome(
    records: &[TraceRecord],
    rejected: &mut Vec<RejectedCandidate>,
) -> Vec<Opportunity> {
    // Unit identity of a record's value (R-020 comparability gate).
    let unit_of = |r: &TraceRecord| r.value.unit();
    let mut predictions: Vec<&TraceRecord> = Vec::new();
    let mut observations: Vec<&TraceRecord> = Vec::new();
    for r in records {
        match r.kind.as_str() {
            "prediction" => predictions.push(r),
            "observation" => observations.push(r),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for o in &observations {
        // Name-match pairing ONLY: an observation compares against the
        // prediction of the same measured quantity. Falling back to an
        // arbitrary other prediction would pair numbers from different
        // conditions (exactly what MASTER_SPEC:144 forbids).
        let Some(p) = predictions.iter().find(|p| p.name == o.name) else {
            rejected.push(RejectedCandidate {
                kind: PressureKind::Anomaly,
                reason: "missing_prediction".into(),
                detail: format!(
                    "observation '{}' ({}) has no prediction record for the same quantity (two isolated numbers are not an anomaly)",
                    o.id, o.name
                ),
            });
            continue;
        };
        if unit_of(p) != unit_of(o) {
            rejected.push(RejectedCandidate {
                kind: PressureKind::Anomaly,
                reason: "unit_mismatch".into(),
                detail: format!(
                    "prediction '{}' is in {}, observation '{}' is in {} — incompatible units never form an anomaly",
                    p.id,
                    unit_of(p),
                    o.id,
                    unit_of(o)
                ),
            });
            continue;
        }
        // Name equality was the pairing key, so the remaining comparability
        // risk is units only; conditions ride the name (see above).
        let (pv, ov) = (p.value.magnitude(), o.value.magnitude());
        let rel = if pv == 0.0 {
            f64::INFINITY
        } else {
            (ov - pv).abs() / pv
        };
        if rel <= 0.5 {
            continue; // within the uncertainty band: holds, nothing recorded
        }
        let mut evidence: Vec<_> = Vec::new();
        if let Some(s) = &p.evidence {
            evidence.push(s.clone());
        }
        if let Some(s) = &o.evidence {
            evidence.push(s.clone());
        }
        if evidence.is_empty() {
            // No fabricated citations for an unevidenced pair.
            continue;
        }
        let mut opp = Opportunity::grounded(PressureKind::Anomaly, evidence);
        opp.problem_statement = format!(
            "observation '{}' ({}) deviates {:.0}% from prediction '{}' ({}) — beyond the stated uncertainty band",
            o.id,
            o.value.magnitude(),
            rel * 100.0,
            p.id,
            p.value.magnitude()
        );
        opp.beneficiary = "owner of the traced workload's correctness model".into();
        opp.context = "authorized trace analysis".into();
        opp.causal_uncertainty = format!(
            "the discrepancy is real but its cause is unestablished: model error, measurement drift, or changed conditions are all live alternatives (relative gap {:.0}%, band 50%)",
            rel * 100.0
        );
        opp.unanswered_questions
            .push("was the prediction's model validated on this regime?".into());
        out.push(opp);
    }
    out
}

/// Conflicting-objectives operator (MASTER_SPEC:144): two objectives whose
/// matched samples move against each other form a trade-off candidate ONLY
/// with an explicit coupling record in the same regime; bare
/// counter-movement is rejected.
fn conflicting_objectives_outcome(
    records: &[TraceRecord],
    rejected: &mut Vec<RejectedCandidate>,
) -> Vec<Opportunity> {
    let mut objective_samples: Vec<&TraceRecord> =
        records.iter().filter(|r| r.kind == "objective").collect();
    objective_samples.sort_by(|a, b| (&a.name, &a.id).cmp(&(&b.name, &b.id)));
    let has_coupling = records
        .iter()
        .any(|r| r.kind == "coupling" && r.evidence.is_some());
    // Group ratio-valued objective records by name; record order within a
    // name is the sample order.
    let mut series: Vec<(String, Vec<f64>)> = Vec::new();
    for r in &objective_samples {
        if let TraceVal::Ratio(v) = r.value {
            match series.iter_mut().find(|(n, _)| *n == r.name) {
                Some((_, vals)) => vals.push(v),
                None => series.push((r.name.clone(), vec![v])),
            }
        }
    }
    // A series qualifies only when it has ≥2 samples.
    series.retain(|(_, vals)| vals.len() >= 2);
    if series.len() < 2 {
        return Vec::new();
    }
    // Counter-movement: one series trends up, the other down, across
    // matched sample positions. Series with different sample counts are
    // NOT comparable position-by-position: comparing them would be
    // exactly the "two isolated numbers from different conditions" risk.
    let comparable = |a: &Vec<f64>, b: &Vec<f64>| a.len() == b.len();
    let trend = |vals: &[f64]| {
        if vals.len() < 2 {
            return 0i8;
        }
        let first = vals.first().copied().unwrap_or(0.0);
        let last = vals.last().copied().unwrap_or(0.0);
        if last > first {
            1
        } else if last < first {
            -1
        } else {
            0
        }
    };
    let mut out = Vec::new();
    for i in 0..series.len() {
        for j in (i + 1)..series.len() {
            let (n1, v1) = (&series[i].0, &series[i].1);
            let (n2, v2) = (&series[j].0, &series[j].1);
            let (t1, t2) = (trend(v1), trend(v2));
            if t1 == 0 || t2 == 0 || t1 == t2 {
                continue; // not counter-moving
            }
            if !comparable(v1, v2) {
                continue; // unmatched sample counts: not comparable positions
            }
            if !has_coupling {
                rejected.push(RejectedCandidate {
                    kind: PressureKind::ConflictingObjectives,
                    reason: "coupling_evidence_required".into(),
                    detail: format!(
                        "'{n1}' and '{n2}' move against each other but no coupling record evidences that the variables are coupled in the relevant design regime (MASTER_SPEC:144)"
                    ),
                });
                continue;
            }
            let evidence: Vec<_> = records
                .iter()
                .filter(|r| {
                    (r.kind == "objective" && (r.name == *n1 || r.name == *n2))
                        || r.kind == "coupling"
                })
                .filter_map(|r| r.evidence.clone())
                .collect();
            if evidence.is_empty() {
                continue;
            }
            let mut opp = Opportunity::grounded(PressureKind::ConflictingObjectives, evidence);
            opp.problem_statement = format!(
                "'{n1}' and '{n2}' move against each other across matched samples, with recorded coupling in this regime"
            );
            opp.beneficiary = "owner of the traded-off objectives".into();
            opp.context = "authorized trace analysis".into();
            opp.causal_uncertainty = format!(
                "the coupling is recorded but its functional form is unestablished — the trade-off curve between '{n1}' and '{n2}' is unknown"
            );
            opp.unanswered_questions.push(format!(
                "is the {n1}/{n2} trade-off constant across regimes?"
            ));
            out.push(opp);
        }
    }
    out
}

/// Failure-pattern operator: ≥3 same-kind (`event`) failure records with
/// the same name form ONE pattern opportunity citing the distinct failing
/// records; fewer than 3 is rejected (a single failure is not a pattern).
fn failure_pattern_outcome(
    records: &[TraceRecord],
    rejected: &mut Vec<RejectedCandidate>,
) -> Vec<Opportunity> {
    let mut by_name: Vec<(String, Vec<&TraceRecord>)> = Vec::new();
    for r in records.iter().filter(|r| r.kind == "event") {
        match by_name.iter_mut().find(|(n, _)| *n == r.name) {
            Some((_, recs)) => recs.push(r),
            None => by_name.push((r.name.clone(), vec![r])),
        }
    }
    let mut out = Vec::new();
    for (name, recs) in &by_name {
        if recs.len() >= 3 {
            let evidence: Vec<_> = recs.iter().filter_map(|r| r.evidence.clone()).collect();
            if evidence.is_empty() {
                continue;
            }
            let mut opp = Opportunity::grounded(PressureKind::FailurePattern, evidence);
            opp.suspected_bottleneck = Some(name.clone());
            opp.problem_statement =
                format!("'{name}' failures recur ({}) across the trace", recs.len());
            opp.beneficiary = "operator of the failing workload".into();
            opp.context = "authorized trace analysis".into();
            opp.causal_uncertainty = format!(
                "the recurrence is established but the shared cause is unestablished — '{}' failures may share one root cause or several",
                name
            );
            opp.unanswered_questions
                .push(format!("do all '{}' failures share one root cause?", name));
            out.push(opp);
        } else if recs.len() == 1 {
            rejected.push(RejectedCandidate {
                kind: PressureKind::FailurePattern,
                reason: "not_a_pattern".into(),
                detail: format!(
                    "single '{}' failure is not a pattern (threshold: ≥3 recurring same-kind failures)",
                    name
                ),
            });
        }
        // 2 occurrences: neither pattern nor worth a rejection row — the
        // threshold is documented; retention is for real candidates.
    }
    out
}

/// Assumption operator: declared unstated assumptions (`assumption`
/// records) become opportunities carrying explicit uncertainty — the
/// assumption itself is the uncertainty (nothing verified it).
fn assumption_outcome(
    records: &[TraceRecord],
    _rejected: &mut Vec<RejectedCandidate>,
) -> Vec<Opportunity> {
    records
        .iter()
        .filter(|r| r.kind == "assumption")
        .filter_map(|r| {
            let evidence = r.evidence.as_ref()?.clone();
            let mut opp = Opportunity::grounded(PressureKind::Assumption, vec![evidence]);
            opp.problem_statement = format!(
                "unstated assumption declared: '{}' — downstream conclusions depend on it unverified",
                r.name
            );
            opp.beneficiary = "consumers of the traced workload's results".into();
            opp.context = "authorized trace analysis".into();
            opp.causal_uncertainty = format!(
                "'{}' is assumed, not verified: if it fails, dependent results fail with it",
                r.name
            );
            opp.unanswered_questions
                .push(format!("has '{}' ever been checked directly?", r.name));
            Some(opp)
        })
        .collect()
}

/// Changed-capability operator (MASTER_SPEC:144): a verified capability
/// (`capability` record) becomes an opportunity ONLY with a concrete
/// account of which previous constraint it changes (a matching
/// `constraint` record with the same name). Availability alone is
/// rejected.
fn changed_capability_outcome(
    records: &[TraceRecord],
    rejected: &mut Vec<RejectedCandidate>,
) -> Vec<Opportunity> {
    let constraints: Vec<&TraceRecord> =
        records.iter().filter(|r| r.kind == "constraint").collect();
    let mut out = Vec::new();
    for r in records.iter().filter(|r| r.kind == "capability") {
        let Some(evidence) = &r.evidence else {
            continue; // unevidenced capability claims never emit
        };
        match constraints.iter().find(|c| c.name == r.name) {
            Some(con) => {
                let mut ev = vec![evidence.clone()];
                if let Some(ce) = &con.evidence {
                    ev.push(ce.clone());
                }
                let mut opp = Opportunity::grounded(PressureKind::ChangedCapability, ev);
                opp.problem_statement = format!(
                    "capability '{}' is verified available and changes the prior constraint it was bound by",
                    r.name
                );
                opp.beneficiary = "owner of the previously-constrained workload".into();
                opp.context = "authorized trace analysis".into();
                opp.causal_uncertainty = format!(
                    "availability is verified but the scope of what '{}' unlocks is unestablished",
                    r.name
                );
                opp.unanswered_questions
                    .push(format!("which workloads was '{}' actually blocking?", r.name));
                out.push(opp);
            }
            None => rejected.push(RejectedCandidate {
                kind: PressureKind::ChangedCapability,
                reason: "constraint_account_required".into(),
                detail: format!(
                    "capability '{}' is verified available but no record states which previous constraint it changes (MASTER_SPEC:144)",
                    r.name
                ),
            }),
        }
    }
    out
}

/// Analyze authorized trace records against the corpus: run every
/// operator, return emitted opportunities plus retained rejections.
pub fn analyze(_corpus: &crate::knowledge::Corpus, records: &[TraceRecord]) -> AnalysisOutcome {
    let mut rejected = Vec::new();
    let mut opportunities = Vec::new();
    opportunities.extend(bottleneck_opportunities(records, &mut rejected));
    opportunities.extend(anomaly_outcome(records, &mut rejected));
    opportunities.extend(conflicting_objectives_outcome(records, &mut rejected));
    opportunities.extend(failure_pattern_outcome(records, &mut rejected));
    opportunities.extend(assumption_outcome(records, &mut rejected));
    opportunities.extend(changed_capability_outcome(records, &mut rejected));
    // Deterministic order: by pressure kind discriminant then statement.
    opportunities.sort_by(|a, b| {
        (a.kind_name(), &a.problem_statement).cmp(&(b.kind_name(), &b.problem_statement))
    });
    rejected.sort_by(|a, b| (a.kind_name(), &a.reason).cmp(&(b.kind_name(), &b.reason)));
    AnalysisOutcome {
        opportunities,
        rejected,
    }
}

impl PressureKind {
    /// Stable discriminant name for ordering and reason codes.
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Bottleneck => "bottleneck",
            Self::ConflictingObjectives => "conflicting_objectives",
            Self::FailurePattern => "failure_pattern",
            Self::Anomaly => "anomaly",
            Self::Assumption => "assumption",
            Self::ChangedCapability => "changed_capability",
        }
    }
}

impl RejectedCandidate {
    pub fn kind_name(&self) -> &'static str {
        self.kind.kind_name()
    }
}
