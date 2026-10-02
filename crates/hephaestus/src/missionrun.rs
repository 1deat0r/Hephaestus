//! The E2E mission chain (T-061): M2->M3 composition shared by the
//! `e2e_m2`/`e2e_m3` tests and the `hephaestus fixture mission-run`
//! CLI. Data-only by construction: callers supply the world-derived
//! trace text; the control plane never links the hidden evaluator
//! (`evaluator_access` guard — architectural, never widened).

use crate::dossier::export;
use crate::dossier::record::{
    CostReceipt, Deviation, Dossier, Environment, EvidenceLabel, FailureEntry, Lineage,
    NegativeResultKind,
};
use crate::evaluation::{InterpretInputs, Measurements, TypedResult};
use crate::experiment::{
    AnalysisSpec, CompileRequest, Controls, PlanInputs, StoppingRule, compile as plan_compile,
};
use crate::knowledge::sha256_hex;
use crate::methods::record::{ClippingPolicy, Estimand, MethodSpec, MissingnessPolicy};
use crate::methods::{IntervalError, ProtectedRegistry, mean_difference_interval};

/// Why the chain could not complete (typed-rejection convention).
#[derive(Debug)]
pub enum ChainError {
    /// A fixture line was not `step=<name> dur_ms=<n>`.
    TraceParse(String),
    Interval(IntervalError),
    /// The frozen experiment plan refused to compile.
    PlanBlocked(String),
    /// Discovery produced no grounded opportunity from the fixture.
    Discovery(String),
    Export(crate::dossier::ExportError),
}

impl std::fmt::Display for ChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainError::TraceParse(m) => write!(f, "trace parse: {m}"),
            ChainError::Interval(e) => write!(f, "interval: {e:?}"),
            ChainError::PlanBlocked(m) => write!(f, "plan blocked: {m}"),
            ChainError::Discovery(m) => write!(f, "discovery: {m}"),
            ChainError::Export(e) => write!(f, "export: {e:?}"),
        }
    }
}

impl std::error::Error for ChainError {}

/// Everything one chain run produces — the receipts the CLI prints and
/// the e2e tests assert on.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChainReceipt {
    /// sha256 of the trace data (version-bound run receipt).
    pub receipt_sha256: String,
    /// Qualified interval, seconds.
    pub interval: (f64, f64),
    pub theta: f64,
    pub typed: TypedResult,
    /// The exported dossier JSON (Measured label inside).
    pub exported: String,
}

fn fixture_rows_ranged(
    trace_text: &str,
) -> Result<Vec<(std::ops::Range<usize>, String, u64)>, ChainError> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    for line in trace_text.split_inclusive('\n') {
        let range = offset..offset + line.len();
        offset += line.len();
        let rest = line
            .trim_end()
            .strip_prefix("step=")
            .ok_or_else(|| ChainError::TraceParse(format!("line shape: {line:?}")))?;
        let (name, dur) = rest
            .split_once(" dur_ms=")
            .ok_or_else(|| ChainError::TraceParse(format!("line shape: {line:?}")))?;
        let dur: u64 = dur
            .parse()
            .map_err(|e| ChainError::TraceParse(format!("dur_ms: {e}")))?;
        out.push((range, name.to_string(), dur));
    }
    if out.is_empty() {
        return Err(ChainError::TraceParse("empty fixture".to_string()));
    }
    Ok(out)
}

fn fixture_rows(trace_text: &str) -> Result<Vec<(String, u64)>, ChainError> {
    Ok(fixture_rows_ranged(trace_text)?
        .into_iter()
        .map(|(_, name, dur)| (name, dur))
        .collect())
}

fn stage_ms(rows: &[(String, u64)], stage: &str) -> Vec<u64> {
    rows.iter()
        .filter(|(n, _)| n == stage)
        .map(|(_, d)| *d)
        .collect()
}

fn registry() -> ProtectedRegistry {
    let registry = ProtectedRegistry::new();
    registry
        .register(MethodSpec {
            name: "mean-difference-z-interval".to_string(),
            version: "1.0.0".to_string(),
            estimand: Estimand {
                quantity: "mean duration difference heavy - light".to_string(),
                unit: "seconds".to_string(),
                denominator: "all fixture trace observations in both stages".to_string(),
                cost_fields: vec![],
            },
            assumptions: vec![
                "independent stage samples".to_string(),
                "normal approximation at z = 1.96".to_string(),
                "fixture-trace durations only".to_string(),
            ],
            alpha_allocations: vec![0.025],
            missingness: MissingnessPolicy::FailedOrMissingIsFailure,
            clipping: ClippingPolicy {
                lower: None,
                upper: None,
                report_clipped_rates: true,
            },
            implementation_digest: "impl-mean-diff-interval-1".to_string(),
        })
        .expect("method registers");
    registry
}

fn request() -> CompileRequest {
    CompileRequest {
        hypothesis_version: "h1.0.0".to_string(),
        claim_ids: vec!["C1".to_string()],
        uncertainty: "does splitting the heavy stage cut rebuild tail time".to_string(),
        protected_evaluator_digest: "eval-digest-1".to_string(),
        resource_limit: "2 CPU-hours".to_string(),
        authorization_scope: "local-fixture".to_string(),
    }
}

fn inputs() -> PlanInputs {
    PlanInputs {
        analysis: Some(AnalysisSpec {
            method: "mean-difference-z-interval".to_string(),
            decision_rule: "lower bound above theta".to_string(),
        }),
        stopping_rule: Some(StoppingRule::FixedSampleCount(30)),
        endpoints: vec!["stage_duration_seconds".to_string()],
        guardrails: vec!["no correctness regressions".to_string()],
        sampling_unit: Some("fixture observation".to_string()),
        comparator: Some("light stage baseline".to_string()),
        controls: Controls {
            positive: Some("full heavy-stage run".to_string()),
            negative: Some("no-op intervention".to_string()),
            randomization: true,
            repeatability_checks: true,
            environmental_capture: true,
            missing_observation_plan: Some("rerun failed trials once; record gaps".to_string()),
        },
    }
}

/// One full chain run: trace -> qualified interval -> frozen plan ->
/// typed result -> measured dossier export.
pub fn run(
    trace_text: &str,
    theta: f64,
    negative_kind: Option<NegativeResultKind>,
) -> Result<ChainReceipt, ChainError> {
    let rows = fixture_rows(trace_text)?;
    let heavy = stage_ms(&rows, "heavy");
    let light = stage_ms(&rows, "light");
    let interval = mean_difference_interval(&heavy, &light).map_err(ChainError::Interval)?;

    let plan = plan_compile(&request(), inputs())
        .map_err(|e| ChainError::PlanBlocked(format!("{e:?}")))?;

    let registry = registry();
    let typed = crate::evaluation::interpret(
        &InterpretInputs {
            plan: &plan,
            protected_evaluator_digest: "eval-digest-1",
            registry: &registry,
            method_name: "mean-difference-z-interval",
            method_version: "1.0.0",
        },
        &Measurements {
            interval: Some(interval),
            threshold: theta,
            quality_margin: None,
            quality_lower_bound: None,
        },
    );

    let receipt = sha256_hex(trace_text.as_bytes());
    let raw_data: Vec<(String, f64)> = rows
        .iter()
        .map(|(n, d)| (format!("{n}_duration_seconds"), *d as f64 / 1000.0))
        .collect();
    let dossier = Dossier {
        problem_and_beneficiary: "rebuild latency; build-tool users".to_string(),
        prior_art_reference: "priorart-report-fixture".to_string(),
        mechanism_explanation: "splitting the heavy stage cuts tail rebuild time".to_string(),
        hypothesis_versions: vec!["h1.0.0".to_string()],
        predeclared_tests: "frozen plan h1.0.0, mean-difference-z-interval".to_string(),
        environment: Environment {
            os: "linux".to_string(),
            tool_versions: vec![("rustc".to_string(), "1.98.1".to_string())],
            artifact_digests: vec![receipt.clone()],
        },
        raw_data,
        analysis: "mean difference heavy - light, z = 1.96 interval, seconds".to_string(),
        results_json: serde_json::to_string(&typed)
            .map_err(|e| ChainError::TraceParse(format!("typed serialize: {e}")))?,
        counterevidence: "none within the fixture scope".to_string(),
        failure_history: vec![FailureEntry {
            stage: "fixture-trace".to_string(),
            description: "none — clean fixture capture, recorded explicitly".to_string(),
            artifact_digest: "no-failure-artifact".to_string(),
        }],
        deviations: vec![Deviation {
            from_plan_field: "stopping_rule".to_string(),
            what_changed: "none — fixed sample count honored".to_string(),
            reason: "deviation-free capture recorded explicitly (R-044)".to_string(),
        }],
        uncertainty: "single fixture world; z-approx only".to_string(),
        scope_limits: "hidden-synthetic-world fixture, local only".to_string(),
        cost_ledger: vec![CostReceipt {
            kind: "human_intervention".to_string(),
            quantity: "0.1".to_string(),
            unit: "human-hours".to_string(),
            provider_charge: None,
        }],
        repro_commands: vec![
            "cargo test --test e2e_m3".to_string(),
            format!("sha256(trace fixture) == {receipt}"),
        ],
        unresolved_risks: "generalization beyond the fixture world".to_string(),
        next_justified_action: "qualified pilot on authorized repositories".to_string(),
        negative_kind,
        lineage: Lineage {
            mission_id: "m-fixture-1".to_string(),
            opportunity_id: "o-bottleneck-1".to_string(),
            mechanism_id: "mech-heavy-split-1".to_string(),
            hypothesis_version: "h1.0.0".to_string(),
            plan_id: "plan-fixture-1".to_string(),
            result_id: "res-fixture-1".to_string(),
        },
        evidence_label: EvidenceLabel::Measured {
            run_receipt: receipt.clone(),
        },
    };
    let exported = export(&dossier).map_err(ChainError::Export)?;
    Ok(ChainReceipt {
        receipt_sha256: receipt,
        interval,
        theta,
        typed,
        exported,
    })
}

/// Receipt from the M2 discovery slice at a declared generation bound
/// (T-061/T-062: the championable `discovery.generation_bound` tunable).
#[derive(Debug, Clone, serde::Serialize)]
pub struct DiscoveryReceipt {
    pub bound: u64,
    pub grounded_opportunities: usize,
    pub mechanisms_found: usize,
    pub rejections: usize,
}

/// Run the M2 slice over the trace fixture: ingest with span-cited
/// evidence -> analyze -> first grounded opportunity -> bounded
/// operator sweep at `max_per_operator`.
pub fn discovery_bound(
    trace_text: &str,
    max_per_operator: u64,
) -> Result<DiscoveryReceipt, ChainError> {
    let rows = fixture_rows_ranged(trace_text)?;
    let mut corpus = crate::knowledge::Corpus::new();
    let src = crate::knowledge::ingest_bytes(
        &mut corpus,
        "fixture://hidden-world/trace.log",
        trace_text.as_bytes().to_vec(),
        "2026-10-02T00:00:00Z",
        crate::knowledge::ParserId {
            name: "trace".to_string(),
            version: "1".to_string(),
        },
    );
    let records: Vec<crate::discovery::TraceRecord> = rows
        .iter()
        .enumerate()
        .map(|(i, (range, name, dur))| crate::discovery::TraceRecord {
            id: format!("t{i}"),
            kind: "duration".to_string(),
            name: name.clone(),
            value: crate::discovery::TraceVal::DurationMs(*dur),
            evidence: Some(crate::discovery::Span {
                source: src,
                start: range.start,
                end: range.end,
                text: trace_text[range.clone()].to_string(),
                transforms: vec![],
            }),
        })
        .collect();
    let outcome = crate::discovery::analyze(&corpus, &records);
    let grounded: Vec<&crate::discovery::Opportunity> = outcome
        .opportunities
        .iter()
        .filter(|o| !o.evidence_references.is_empty())
        .collect();
    let first = *grounded
        .first()
        .ok_or_else(|| ChainError::Discovery("no grounded opportunity".to_string()))?;
    let sweep = crate::genesis::registry::apply_all(
        first,
        vec![
            "the heavy stage dominates observed duration".to_string(),
            "splitting the heavy stage could cut tail latency".to_string(),
        ],
        vec![],
        max_per_operator as usize,
    );
    Ok(DiscoveryReceipt {
        bound: max_per_operator,
        grounded_opportunities: grounded.len(),
        mechanisms_found: sweep.mechanisms.len(),
        rejections: sweep.rejected_applicability.len(),
    })
}
