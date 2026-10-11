//! Release packet assembly (T-028, R-077/R-078/R-080).
//!
//! Assembles the scoped release packet with a critical-failure gate
//! (R-077), required finite-suite uncertainty (R-078 — no
//! universal-reliability representation exists), all five outcome classes
//! (R-080), quantity-only measurements, and a reproducibility report.

pub mod record;

pub use record::{
    Finding, Measurement, OutcomeCounts, PerformanceClaim, QualificationInputs, ReleaseBlock,
    ReleasePacket, ReproducibilityReport, ScopeLabel, Severity, Uncertainty, scope_label,
};

/// Assemble the release packet (R-077 gate + completeness).
pub fn assemble_release(packet: ReleasePacket) -> Result<ReleasePacket, ReleaseBlock> {
    // R-077: a critical unresolved failure INSIDE the declared scope
    // blocks the applicable release gate. Out-of-scope criticals are
    // recorded but do not block (scope honesty).
    let blocking = packet
        .findings
        .iter()
        .find(|f| f.severity == Severity::Critical && !f.resolved && f.in_scope);
    if let Some(f) = blocking {
        return Err(ReleaseBlock::CriticalUnresolvedInScope(f.id.clone()));
    }
    // R-078: finite-suite uncertainty required; there is no field that
    // could carry a universal-reliability claim.
    if packet.uncertainty.n == 0 || packet.uncertainty.suite_id.is_empty() {
        return Err(ReleaseBlock::MissingUncertainty);
    }
    // Reproducibility report must be present with commands.
    if packet.reproducibility.repro_commands.is_empty() {
        return Err(ReleaseBlock::MissingReproducibility);
    }
    // R-076: a claim presented as achieved must pin receipt, measured
    // value, AND the recorded reference machine (section 26) — a
    // provisional target dressed up as a benchmark is refused by name.
    for claim in &packet.performance_claims {
        if let record::PerformanceClaim::MeasuredBenchmark {
            feature,
            measured_value,
            benchmark_receipt,
            reference_machine,
            ..
        } = claim
            && (benchmark_receipt.is_empty()
                || measured_value.is_empty()
                || reference_machine.is_empty())
        {
            return Err(ReleaseBlock::UnmeasuredTargetDisplayed(feature.clone()));
        }
    }
    // R-103 continuity: measurements are quantities; a priced entry (a
    // "$" in the unit) is an invented conversion — refused.
    for m in &packet.cost_latency {
        if m.unit.contains('$') {
            return Err(ReleaseBlock::PricedMeasurement);
        }
    }
    Ok(packet)
}

/// The single achieved-benchmark display funnel (R-076): only fully
/// pinned measurements can reach the dossier/UI as achieved results;
/// provisional section-26 targets are structurally excluded.
pub fn achieved_benchmarks(packet: &ReleasePacket) -> Vec<&record::PerformanceClaim> {
    packet
        .performance_claims
        .iter()
        .filter(|c| matches!(c, record::PerformanceClaim::MeasuredBenchmark { .. }))
        .collect()
}

/// Assemble the v1 fixture-scope dossier from a fresh trace run
/// plus the digests of the committed campaign reports (v1-release
/// 03). All numbers come from actual runs; absences stay explicit.
/// Returns the gated packet and its Markdown rendering.
pub fn assemble_v1_dossier(
    trace_text: &str,
    pilot_report_digest: &str,
    baseline_report_digest: &str,
) -> Result<(ReleasePacket, String), String> {
    if pilot_report_digest.is_empty() || baseline_report_digest.is_empty() {
        return Err("missing input report digest: refusing unpinned assembly".to_string());
    }
    let batch =
        crate::campaign::run_fixture_batch(trace_text).map_err(|e| format!("pilot batch: {e}"))?;
    let envelope = crate::evalsuite::Envelope {
        compute_budget: 20,
        tool_access: vec!["missionrun".to_string()],
        model_access: "none".to_string(),
        tuning_record: "fixture-fixed".to_string(),
    };
    let (base, cand, _, _) = crate::acceleration::measure_deterministic_arms(trace_text, envelope)
        .map_err(|e| format!("baseline measure: {e}"))?;
    let comparison = crate::evalsuite::compare_qualified_efficiency(&base, &cand)
        .map_err(|e| format!("baseline compare: {e}"))?;

    // Outcome counts from actual run values: positive margin work
    // lands positive; the honest-negative control lands negative.
    // All five classes stay present, possibly zero.
    let mut outcomes = record::OutcomeCounts::default();
    for o in &batch.outcomes {
        if o.failed_or_blocked {
            outcomes.blocked += 1;
            continue;
        }
        match o.value {
            Some(v) if v > 0.0 => outcomes.positive += 1,
            Some(_) => outcomes.negative += 1,
            None => outcomes.inconclusive += 1,
        }
    }

    // Uncertainty is the OBSERVED cand-arm value range over the
    // fixture suite - never a confidence interval.
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for m in &batch.missions {
        if m.arm_id == "cand" {
            lo = lo.min(m.value);
            hi = hi.max(m.value);
        }
    }
    if !lo.is_finite() || !hi.is_finite() || lo > hi {
        return Err("no finite cand-arm values: uncertainty range unmeasurable".to_string());
    }
    let machine = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    let tokens_total = comparison.baseline_tokens + comparison.candidate_tokens;
    let latency_total = comparison.baseline_latency_ms + comparison.candidate_latency_ms;
    let packet = ReleasePacket {
        label: record::scope_label(&record::QualificationInputs::default()),
        declared_scope: "fixture pilot batch + deterministic baseline comparison (v0.0.1 track)".to_string(),
        findings: vec![
            record::Finding {
                id: "compute-wall-time-unmetered".to_string(),
                severity: record::Severity::Minor,
                resolved: false,
                in_scope: true,
                description: "Compute spend and wall time are not metered; local latency stands in.".to_string(),
            },
            record::Finding {
                id: "model-arms-absent".to_string(),
                severity: record::Severity::Minor,
                resolved: false,
                in_scope: true,
                description: "No consented provider contract exists; no model arm ran.".to_string(),
            },
        ],
        uncertainty: record::Uncertainty {
            suite_id: batch.plan.preregistration_id.clone(),
            n: batch.total_missions,
            interval: (lo, hi),
        },
        outcome_counts: outcomes,
        cost_latency: vec![
            record::Measurement {
                kind: "tokens".to_string(),
                quantity: tokens_total.to_string(),
                unit: "fixed-tokenizer-tokens".to_string(),
            },
            record::Measurement {
                kind: "latency".to_string(),
                quantity: latency_total.to_string(),
                unit: "ms-wall-local-total".to_string(),
            },
            record::Measurement {
                kind: "peak-memory".to_string(),
                quantity: comparison
                    .baseline_peak_memory_bytes
                    .max(comparison.candidate_peak_memory_bytes)
                    .to_string(),
                unit: "bytes-vmhwm-max".to_string(),
            },
        ],
        reproducibility: record::ReproducibilityReport {
            environment_pins: vec![
                ("os".to_string(), std::env::consts::OS.to_string()),
                ("arch".to_string(), std::env::consts::ARCH.to_string()),
            ],
            repro_commands: vec![
                "hephaestus fixture pilot-batch --trace <TRACE> --out docs/PILOT_CAMPAIGN_REPORT.md".to_string(),
                "hephaestus fixture baseline-compare --trace <TRACE> --out docs/BASELINE_COMPARISON_REPORT.md".to_string(),
                "hephaestus fixture assemble-release --trace <TRACE> --pilot docs/PILOT_CAMPAIGN_REPORT.md --baseline docs/BASELINE_COMPARISON_REPORT.md --out docs/RELEASE_DOSSIER_V1.md".to_string(),
            ],
            artifact_verifications: vec![
                ("docs/PILOT_CAMPAIGN_REPORT.md".to_string(), !pilot_report_digest.is_empty()),
                ("docs/BASELINE_COMPARISON_REPORT.md".to_string(), !baseline_report_digest.is_empty()),
            ],
        },
        unresolved_questions: vec![
            "Do pilot margins hold on live missions with owner consent?".to_string(),
            "Do consented model arms beat deterministic baselines at matched envelopes?".to_string(),
            "What is the metered compute cost per qualified outcome?".to_string(),
            "Which scope qualifies for QUALIFIED_FOR_DECLARED_SCOPE, and with what frozen campaign?".to_string(),
        ],
        performance_claims: vec![record::PerformanceClaim::MeasuredBenchmark {
            feature: "fixture-cand-arm-positive-margin-rate".to_string(),
            metric_label: "positive-margin fraction over 10 fixture repos".to_string(),
            measured_value: comparison.candidate_correctness.to_string(),
            unit: "fraction".to_string(),
            benchmark_receipt: format!("pilot-report:{pilot_report_digest}"),
            reference_machine: machine,
        }],
    };
    let packet = assemble_release(packet).map_err(|e| format!("gate: {e:?}"))?;
    let doc = render_v1_dossier_verdict(
        &packet,
        &comparison,
        batch.failures_retained,
        "passed: no critical unresolved finding in scope",
        pilot_report_digest,
        baseline_report_digest,
    );
    Ok((packet, doc))
}

/// Render the gated packet as Markdown (v1-release 03). Sections
/// carrying measured latency, throughput, or memory vary run to
/// run; label, counts, receipts, and gate verdicts replay exactly.
pub fn render_v1_dossier(
    packet: &ReleasePacket,
    comparison: &crate::evalsuite::EfficiencyComparison,
    failures_retained: usize,
    pilot_report_digest: &str,
    baseline_report_digest: &str,
) -> String {
    render_v1_dossier_verdict(
        packet,
        comparison,
        failures_retained,
        "UNVERIFIED: rendered without passing the R-077 gate",
        pilot_report_digest,
        baseline_report_digest,
    )
}

/// Render with an explicit caller-supplied gate verdict: the header
/// never claims a pass on its own; only the gated assembler passes
/// the passing verdict.
pub fn render_v1_dossier_verdict(
    packet: &ReleasePacket,
    comparison: &crate::evalsuite::EfficiencyComparison,
    failures_retained: usize,
    gate_verdict: &str,
    pilot_report_digest: &str,
    baseline_report_digest: &str,
) -> String {
    let mut out = String::new();
    let label = match packet.label {
        record::ScopeLabel::Experimental => "EXPERIMENTAL",
        record::ScopeLabel::QualifiedForDeclaredScope => "QUALIFIED_FOR_DECLARED_SCOPE",
    };
    out.push_str("# Release dossier v1 - fixture scope\n\n");
    out.push_str(&format!(
        "Scope label: {label} (default; qualification inputs all absent).\n\n"
    ));
    out.push_str(&format!("Declared scope: {}.\n\n", packet.declared_scope));
    out.push_str(&format!(
        "Campaign: {} over {} missions; failures retained: {}.\n\n",
        packet.uncertainty.suite_id, packet.uncertainty.n, failures_retained,
    ));
    out.push_str("## Comparison carried\n\n");
    out.push_str(&format!(
        "baseline {}: correctness {:.6}, tokens: {}, cost {:.1}, latency total {} ms, throughput mean {:.3} tok/s, peak memory max {} bytes\n",
        comparison.baseline_id,
        comparison.baseline_correctness,
        comparison.baseline_tokens,
        comparison.baseline_cost,
        comparison.baseline_latency_ms,
        comparison.baseline_throughput,
        comparison.baseline_peak_memory_bytes,
    ));
    out.push_str(&format!(
        "candidate {}: correctness {:.6}, tokens: {}, cost {:.1}, latency total {} ms, throughput mean {:.3} tok/s, peak memory max {} bytes\n",
        comparison.candidate_id,
        comparison.candidate_correctness,
        comparison.candidate_tokens,
        comparison.candidate_cost,
        comparison.candidate_latency_ms,
        comparison.candidate_throughput,
        comparison.candidate_peak_memory_bytes,
    ));
    out.push_str(&format!(
        "\nfalse promotions: {} (no gate ran; count by absence).\n",
        comparison.false_promotions
    ));
    out.push_str("\n## Outcome counts (all five classes)\n\n");
    let o = &packet.outcome_counts;
    out.push_str(&format!(
        "positive: {}\nnegative: {}\ninconclusive: {}\ninvalid: {}\nblocked: {}\n",
        o.positive, o.negative, o.inconclusive, o.invalid, o.blocked,
    ));
    out.push_str("\n## Uncertainty (observed range, not a confidence interval)\n\n");
    out.push_str(&format!(
        "suite {}; n {}; observed cand-arm value range ({:.6}, {:.6}).\n",
        packet.uncertainty.suite_id,
        packet.uncertainty.n,
        packet.uncertainty.interval.0,
        packet.uncertainty.interval.1,
    ));
    out.push_str("\n## Cost and latency (quantities only)\n\n");
    for m in &packet.cost_latency {
        out.push_str(&format!("- {}: {} {}\n", m.kind, m.quantity, m.unit));
    }
    out.push_str(&format!("\n## Findings (R-077 gate: {gate_verdict})\n\n"));
    for f in &packet.findings {
        out.push_str(&format!(
            "- {} [{:?}{}] {}\n",
            f.id,
            f.severity,
            if f.resolved { ", resolved" } else { ", open" },
            f.description,
        ));
    }
    out.push_str("\n## Reproducibility\n\n");
    for (k, v) in &packet.reproducibility.environment_pins {
        out.push_str(&format!("- pin {k}: {v}\n"));
    }
    for c in &packet.reproducibility.repro_commands {
        out.push_str(&format!("- run `{c}`\n"));
    }
    for (a, ok) in &packet.reproducibility.artifact_verifications {
        out.push_str(&format!("- input {a}: digest present: {ok}\n"));
    }
    out.push_str("\n## Unresolved research questions\n\n");
    for q in &packet.unresolved_questions {
        out.push_str(&format!("- {q}\n"));
    }
    out.push_str("\n## Benchmarks pinned\n\n");
    for c in achieved_benchmarks(packet) {
        if let record::PerformanceClaim::MeasuredBenchmark {
            feature,
            metric_label,
            measured_value,
            unit,
            benchmark_receipt,
            reference_machine,
        } = c
        {
            out.push_str(&format!(
                "- {feature}: {metric_label} = {measured_value} {unit} (receipt {benchmark_receipt}; machine {reference_machine})\n",
            ));
        }
    }
    out.push_str("\n## Input digests (run-local provenance)\n\n");
    out.push_str(&format!(
        "- pilot report: {pilot_report_digest}\n- baseline report: {baseline_report_digest}\n"
    ));
    out
}
