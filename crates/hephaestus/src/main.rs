//! `hephaestus` — the local CLI (MASTER_SPEC:42) carrying M1's exit line
//! (IMPLEMENTATION_PLAN:44): run and recover a deterministic fixture DAG.
//!
//! Usage:
//!   hephaestus fixture run         --state-dir <DIR>
//!   hephaestus fixture recover     --state-dir <DIR>
//!   hephaestus fixture mission-run --trace <FILE>
//!   hephaestus fixture pilot-batch --trace <FILE> --out <FILE>
//!   hephaestus fixture baseline-compare --trace <FILE> --out <FILE>
//!   hephaestus fixture assemble-release --trace <FILE> --pilot <FILE> --baseline <FILE> --out <FILE>
//!   hephaestus fixture freeze-campaign --trace <FILE> --pilot <FILE> --baseline <FILE> --out <FILE>
//!   hephaestus fixture verify-manifest --manifest <FILE> --trace <FILE> --pilot <FILE> --baseline <FILE>
//!   hephaestus fixture canary-watch
//!   hephaestus --help
//!
//! Exit codes: 0 success · 1 operational failure (bad state dir, refused
//! overwrite) · 2 usage error (unknown/missing arguments).
//!
//! Documentation lives here and in `--help`: README is MANIFEST-frozen
//! (81-file envelope) and its shell fences are gate-checked, so CLI docs
//! deliberately do not touch it (auto-workflow decision, cycle 9).

mod fixture;

use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: hephaestus <fixture run|fixture recover> --state-dir <DIR>\n       hephaestus <fixture mission-run> --trace <FILE>\n       hephaestus <fixture pilot-batch> --trace <FILE> --out <FILE>\n       hephaestus <fixture baseline-compare> --trace <FILE> --out <FILE>\n       hephaestus <fixture assemble-release> --trace <FILE> --pilot <FILE> --baseline <FILE> --out <FILE>\n       hephaestus <fixture freeze-campaign> --trace <FILE> --pilot <FILE> --baseline <FILE> --out <FILE>\n       hephaestus <fixture verify-manifest> --manifest <FILE> --trace <FILE> --pilot <FILE> --baseline <FILE>\n       hephaestus <fixture canary-watch>\n       hephaestus --help\n";

fn print_help() {
    println!(
        "hephaestus - local CLI (M1 exit fixture)\n\n\
         \x20 Commands:\n\
         \x20   fixture run     --state-dir <DIR>   run the deterministic fixture DAG\n\
         \x20                                       (state dir must be empty or absent)\n\
         \x20   fixture recover --state-dir <DIR>   replay the recorded ledger and print\n\
         \x20                                       states plus the recovery plan\n\
         \x20   fixture mission-run --trace <FILE>   run the E2E mission chain over a\n\
         \x20                                       world-derived trace fixture and print\n\
         \x20                                       supported + negative receipts\n\
         \x20   fixture pilot-batch --trace <FILE> --out <FILE>   run the preregistered 20-mission\n         \x20                                       fixture batch and write the pilot report\n\
         \x20   fixture baseline-compare --trace <FILE> --out <FILE>   measure deterministic\n\
         \x20                                       arms and write the comparison report\n\
         \x20   fixture assemble-release --trace <FILE> --pilot <FILE>\n\
         \x20       --baseline <FILE> --out <FILE>   gate and render the v1 dossier\n\
         \x20   fixture freeze-campaign --trace <FILE> --pilot <FILE>\n\
         \x20       --baseline <FILE> --out <FILE>   freeze campaign inputs\n\
         \x20   fixture verify-manifest --manifest <FILE> --trace <FILE>
\
         \x20       --pilot <FILE> --baseline <FILE>   verify a frozen manifest\n\
         \x20   fixture canary-watch                monitor the fixture deployment over a\n\
         \x20                                       fixed observation stream and print\n\
         \x20                                       the stop outcome + rollback receipt\n\n\
         \x20 Exit codes: 0 ok - 1 operational failure - 2 usage error\n\n{USAGE}"
    );
}

fn usage_error() -> ExitCode {
    let _ = std::io::stderr().write_all(USAGE.as_bytes());
    ExitCode::from(2)
}

fn trace_of(rest: &[String]) -> Result<PathBuf, ()> {
    match rest {
        [flag, file] if flag == "--trace" && !file.is_empty() && !file.starts_with('-') => {
            Ok(PathBuf::from(file))
        }
        _ => Err(()),
    }
}

fn state_dir_of(rest: &[String]) -> Result<PathBuf, ()> {
    match rest {
        [flag, dir] if flag == "--state-dir" && !dir.is_empty() && !dir.starts_with('-') => {
            Ok(PathBuf::from(dir))
        }
        _ => Err(()),
    }
}

/// Shared dispatch: parse args, run one command, print its JSON summary.
fn command(rest: &[String], body: fn(&std::path::Path) -> Result<String, String>) -> ExitCode {
    match state_dir_of(rest) {
        Err(()) => usage_error(),
        Ok(dir) => match body(&dir) {
            Err(e) => {
                let _ = writeln!(std::io::stderr(), "error: {e}");
                ExitCode::from(1)
            }
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
        },
    }
}

/// `fixture mission-run --trace <FILE>`: run the E2E chain twice over
/// the supplied world-derived trace (supported claim + honest negative)
/// and print both receipts as one JSON document.
fn mission_run(rest: &[String]) -> ExitCode {
    use hephaestus::dossier::record::NegativeResultKind;
    let path = match trace_of(rest) {
        Err(()) => return usage_error(),
        Ok(p) => p,
    };
    let text = match std::fs::read_to_string(&path) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: read {}: {e}", path.display());
            return ExitCode::from(1);
        }
        Ok(t) => t,
    };
    let supported = hephaestus::missionrun::run(&text, 0.25, None);
    let negative =
        hephaestus::missionrun::run(&text, 10.0, Some(NegativeResultKind::UnsupportedMechanism));
    match (supported, negative) {
        (Ok(s), Ok(n)) => {
            println!("{}", serde_json::json!({ "supported": s, "negative": n }));
            ExitCode::SUCCESS
        }
        (Err(e), _) | (_, Err(e)) => {
            let _ = writeln!(std::io::stderr(), "error: {e}");
            ExitCode::from(1)
        }
    }
}

/// `fixture canary-watch`: monitor the fixed fixture deployment over a
/// fixed observation stream (clear, clear, breach) and print the stop
/// outcome plus the rollback receipt as JSON. Deterministic: no clock,
/// no I/O, fixed digests and limits.
fn canary_watch() -> ExitCode {
    use hephaestus::selfimprove::record::{
        BoundKind, Deployment, GuardrailIndicators, Indicator, MonitorOutcome, MonitorPolicy,
    };
    let deployment = Deployment {
        challenger_id: "bound-16".to_string(),
        challenger_digest: "d16".to_string(),
        incumbent_id: "bound-8".to_string(),
        incumbent_digest: "d8".to_string(),
        grant_scope: vec!["discovery.generation_bound".to_string()],
        assessment_payload_digest: "d16".to_string(),
        observed_scope: "canary-bounded".to_string(),
    };
    let policy = MonitorPolicy {
        max_checks: 3,
        stop_rules: vec!["stop the rollout on any guardrail breach".to_string()],
        verified_incumbent_digest: "d8".to_string(),
    };
    let obs = |observed: f64| GuardrailIndicators {
        indicators: vec![Indicator {
            name: "latency_ms".to_string(),
            kind: BoundKind::AtMost,
            observed,
            limit: 100.0,
        }],
    };
    let stream = vec![obs(90.0), obs(95.0), obs(250.0)];
    match hephaestus::selfimprove::monitor_deployment(&deployment, &policy, &stream) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: {e}");
            ExitCode::from(1)
        }
        Ok(MonitorOutcome::Completed { checks }) => {
            println!(
                "{}",
                serde_json::json!({ "outcome": "completed", "checks": checks })
            );
            ExitCode::SUCCESS
        }
        Ok(MonitorOutcome::Stopped { checks, rollback }) => {
            println!(
                "{}",
                serde_json::json!({
                    "outcome": "stopped",
                    "checks": checks,
                    "rollback": {
                        "restored_incumbent_digest": rollback.restored_incumbent_digest,
                        "rolled_back_challenger_digest": rollback.rolled_back_challenger_digest,
                        "reason": rollback.reason,
                        "observations": rollback.observations,
                    },
                })
            );
            ExitCode::SUCCESS
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [help] if help == "--help" || help == "help" => {
            print_help();
            ExitCode::SUCCESS
        }
        [fixture, rest @ ..] if fixture == "fixture" => match rest.first().map(String::as_str) {
            Some("run") => command(&rest[1..], |dir| {
                fixture::run(dir).map(|s| serde_json::to_string(&s).expect("summary serializes"))
            }),
            Some("recover") => command(&rest[1..], |dir| {
                fixture::recover(dir)
                    .map(|s| serde_json::to_string(&s).expect("summary serializes"))
            }),
            Some("mission-run") => mission_run(&rest[1..]),
            Some("canary-watch") => canary_watch(),
            Some("pilot-batch") => pilot_batch(&rest[1..]),
            Some("baseline-compare") => baseline_compare(&rest[1..]),
            Some("assemble-release") => assemble_release_cmd(&rest[1..]),
            Some("freeze-campaign") => freeze_campaign_cmd(&rest[1..]),
            Some("verify-manifest") => verify_manifest_cmd(&rest[1..]),
            _ => usage_error(),
        },
        _ => usage_error(),
    }
}

/// `fixture pilot-batch --trace <FILE> --out <FILE>`: run the
/// preregistered 20-mission fixture batch and write the Markdown
/// pilot report (v1-release 01). Deterministic: twin runs emit
/// byte-identical files.
fn pilot_batch(rest: &[String]) -> ExitCode {
    let mut trace: Option<&str> = None;
    let mut out: Option<&str> = None;
    let mut args = rest.iter();
    while let Some(flag) = args.next() {
        let value = match args.next() {
            Some(v) if !v.is_empty() && !v.starts_with('-') => v,
            _ => return usage_error(),
        };
        match flag.as_str() {
            "--trace" if trace.is_none() => trace = Some(value),
            "--out" if out.is_none() => out = Some(value),
            _ => return usage_error(),
        }
    }
    let (Some(trace), Some(out)) = (trace, out) else {
        return usage_error();
    };
    let trace = PathBuf::from(trace);
    let out = PathBuf::from(out);
    if out.exists() {
        let _ = writeln!(
            std::io::stderr(),
            "error: refusing to overwrite {}",
            out.display()
        );
        return ExitCode::from(1);
    }
    let text = match std::fs::read_to_string(&trace) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: read {}: {e}", trace.display());
            return ExitCode::from(1);
        }
        Ok(t) => t,
    };
    let report = match hephaestus::campaign::run_fixture_batch(&text) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: batch: {e}");
            return ExitCode::from(1);
        }
        Ok(r) => r,
    };
    let doc = hephaestus::campaign::render_pilot_report(&report);
    match std::fs::write(&out, &doc) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: write {}: {e}", out.display());
            ExitCode::from(1)
        }
        Ok(()) => {
            println!(
                "{}",
                serde_json::json!({
                    "report": out.display().to_string(),
                    "missions": report.total_missions,
                    "bytes": doc.len(),
                })
            );
            ExitCode::SUCCESS
        }
    }
}

/// `fixture baseline-compare --trace <FILE> --out <FILE>`: measure the
/// deterministic base/cand arms over trace subsamples and write the
/// baseline comparison report (v1-release 02 S1). Correctness, tokens,
/// cost, and promotion counts replay exactly; measured latency,
/// throughput, and memory vary run to run and say so in the report.
fn baseline_compare(rest: &[String]) -> ExitCode {
    let mut trace: Option<&str> = None;
    let mut out: Option<&str> = None;
    let mut args = rest.iter();
    while let Some(flag) = args.next() {
        let value = match args.next() {
            Some(v) if !v.is_empty() && !v.starts_with('-') => v,
            _ => return usage_error(),
        };
        match flag.as_str() {
            "--trace" if trace.is_none() => trace = Some(value),
            "--out" if out.is_none() => out = Some(value),
            _ => return usage_error(),
        }
    }
    let (Some(trace), Some(out)) = (trace, out) else {
        return usage_error();
    };
    let trace = PathBuf::from(trace);
    let out = PathBuf::from(out);
    if out.exists() {
        let _ = writeln!(
            std::io::stderr(),
            "error: refusing to overwrite {}",
            out.display()
        );
        return ExitCode::from(1);
    }
    let text = match std::fs::read_to_string(&trace) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: read {}: {e}", trace.display());
            return ExitCode::from(1);
        }
        Ok(t) => t,
    };
    let envelope = hephaestus::evalsuite::Envelope {
        compute_budget: 20,
        tool_access: vec!["missionrun".to_string()],
        model_access: "none".to_string(),
        tuning_record: "fixture-fixed".to_string(),
    };
    let (base, cand, base_receipts, cand_receipts, base_compute, cand_compute) =
        match hephaestus::acceleration::measure_deterministic_arms(&text, envelope) {
            Err(e) => {
                let _ = writeln!(std::io::stderr(), "error: measure: {e}");
                return ExitCode::from(1);
            }
            Ok(p) => p,
        };
    let comparison = match hephaestus::evalsuite::compare_qualified_efficiency(&base, &cand) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: compare: {e}");
            return ExitCode::from(1);
        }
        Ok(c) => c,
    };
    let doc = hephaestus::acceleration::render_baseline_report(
        &comparison,
        &base,
        &cand,
        &base_receipts,
        &cand_receipts,
        &base_compute,
        &cand_compute,
    );
    match std::fs::write(&out, &doc) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: write {}: {e}", out.display());
            ExitCode::from(1)
        }
        Ok(()) => {
            println!(
                "{}",
                serde_json::json!({
                    "report": out.display().to_string(),
                    "baseline_correctness": comparison.baseline_correctness,
                    "candidate_correctness": comparison.candidate_correctness,
                    "false_promotions": comparison.false_promotions,
                })
            );
            ExitCode::SUCCESS
        }
    }
}

/// `fixture assemble-release --trace <FILE> --pilot <FILE> --baseline
/// <FILE> --out <FILE>`: gate and render the v1 fixture-scope dossier
/// (v1-release 03). Reads the committed campaign reports for digest
/// provenance, measures fresh from the trace, and writes the dossier.
/// Fails closed on any gate block or missing input.
fn assemble_release_cmd(rest: &[String]) -> ExitCode {
    let mut trace: Option<&str> = None;
    let mut pilot: Option<&str> = None;
    let mut baseline: Option<&str> = None;
    let mut out: Option<&str> = None;
    let mut args = rest.iter();
    while let Some(flag) = args.next() {
        let value = match args.next() {
            Some(v) if !v.is_empty() && !v.starts_with('-') => v,
            _ => return usage_error(),
        };
        match flag.as_str() {
            "--trace" if trace.is_none() => trace = Some(value),
            "--pilot" if pilot.is_none() => pilot = Some(value),
            "--baseline" if baseline.is_none() => baseline = Some(value),
            "--out" if out.is_none() => out = Some(value),
            _ => return usage_error(),
        }
    }
    let (Some(trace), Some(pilot), Some(baseline), Some(out)) = (trace, pilot, baseline, out)
    else {
        return usage_error();
    };
    let out = PathBuf::from(out);
    if out.exists() {
        let _ = writeln!(
            std::io::stderr(),
            "error: refusing to overwrite {}",
            out.display()
        );
        return ExitCode::from(1);
    }
    let read = |p: &str| match std::fs::read(p) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: read {p}: {e}");
            None
        }
        Ok(b) => Some(b),
    };
    let (Some(trace_bytes), Some(pilot_bytes), Some(baseline_bytes)) =
        (read(trace), read(pilot), read(baseline))
    else {
        return ExitCode::from(1);
    };
    let trace_text = match String::from_utf8(trace_bytes) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: trace utf8: {e}");
            return ExitCode::from(1);
        }
        Ok(s) => s,
    };
    // Provenance is a marker check, not a full parse: the inputs must
    // read as the campaign reports they claim to be. Any other file
    // refuses here, before any digest binds.
    let pilot_text = match String::from_utf8(pilot_bytes) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: pilot utf8: {e}");
            return ExitCode::from(1);
        }
        Ok(s) => s,
    };
    if !pilot_text.contains("prereg-fixture-campaign-1") {
        let _ = writeln!(
            std::io::stderr(),
            "error: --pilot is not a pilot campaign report"
        );
        return ExitCode::from(1);
    }
    let baseline_text = match String::from_utf8(baseline_bytes) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: baseline utf8: {e}");
            return ExitCode::from(1);
        }
        Ok(s) => s,
    };
    if !baseline_text.contains("Baseline comparison report") {
        let _ = writeln!(
            std::io::stderr(),
            "error: --baseline is not a baseline comparison report"
        );
        return ExitCode::from(1);
    }
    let pilot_digest = hephaestus::knowledge::sha256_hex(pilot_text.as_bytes());
    let baseline_digest = hephaestus::knowledge::sha256_hex(baseline_text.as_bytes());
    let (packet, doc) = match hephaestus::release::assemble_v1_dossier(
        &trace_text,
        &pilot_digest,
        &baseline_digest,
    ) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: assemble: {e}");
            return ExitCode::from(1);
        }
        Ok(p) => p,
    };
    match std::fs::write(&out, &doc) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: write {}: {e}", out.display());
            ExitCode::from(1)
        }
        Ok(()) => {
            println!(
                "{}",
                serde_json::json!({
                    "report": out.display().to_string(),
                    "label": format!("{:?}", packet.label),
                    "positive": packet.outcome_counts.positive,
                    "negative": packet.outcome_counts.negative,
                })
            );
            ExitCode::SUCCESS
        }
    }
}

/// `fixture freeze-campaign --trace <FILE> --pilot <FILE> --baseline
/// <FILE> --out <FILE>`: bind the fixture campaign inputs into a
/// frozen manifest (frozen-campaign 01). The frozen section replays
/// byte-identical; toolchain provenance stays labeled run-local.
fn freeze_campaign_cmd(rest: &[String]) -> ExitCode {
    let mut trace: Option<&str> = None;
    let mut pilot: Option<&str> = None;
    let mut baseline: Option<&str> = None;
    let mut out: Option<&str> = None;
    let mut args = rest.iter();
    while let Some(flag) = args.next() {
        let value = match args.next() {
            Some(v) if !v.is_empty() && !v.starts_with('-') => v,
            _ => return usage_error(),
        };
        match flag.as_str() {
            "--trace" if trace.is_none() => trace = Some(value),
            "--pilot" if pilot.is_none() => pilot = Some(value),
            "--baseline" if baseline.is_none() => baseline = Some(value),
            "--out" if out.is_none() => out = Some(value),
            _ => return usage_error(),
        }
    }
    let (Some(trace), Some(pilot), Some(baseline), Some(out)) = (trace, pilot, baseline, out)
    else {
        return usage_error();
    };
    let out = PathBuf::from(out);
    if out.exists() {
        let _ = writeln!(
            std::io::stderr(),
            "error: refusing to overwrite {}",
            out.display()
        );
        return ExitCode::from(1);
    }
    let read = |p: &str| match std::fs::read(p) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: read {p}: {e}");
            None
        }
        Ok(b) => Some(b),
    };
    let (Some(trace_bytes), Some(pilot_bytes), Some(baseline_bytes)) =
        (read(trace), read(pilot), read(baseline))
    else {
        return ExitCode::from(1);
    };
    let (Ok(trace_text), Ok(pilot_text), Ok(baseline_text)) = (
        String::from_utf8(trace_bytes),
        String::from_utf8(pilot_bytes),
        String::from_utf8(baseline_bytes),
    ) else {
        let _ = writeln!(std::io::stderr(), "error: inputs must be utf8");
        return ExitCode::from(1);
    };
    let manifest =
        match hephaestus::campaign::freeze_campaign(&trace_text, &pilot_text, &baseline_text) {
            Err(e) => {
                let _ = writeln!(std::io::stderr(), "error: freeze: {e}");
                return ExitCode::from(1);
            }
            Ok(m) => m,
        };
    let doc = match hephaestus::campaign::render_frozen_manifest(&manifest) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: render: {e}");
            return ExitCode::from(1);
        }
        Ok(d) => d,
    };
    match std::fs::write(&out, &doc) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: write {}: {e}", out.display());
            ExitCode::from(1)
        }
        Ok(()) => {
            println!(
                "{}",
                serde_json::json!({
                    "report": out.display().to_string(),
                    "preregistration_id": manifest.preregistration_id,
                    "trace_sha256": manifest.trace_sha256,
                })
            );
            ExitCode::SUCCESS
        }
    }
}

/// `fixture verify-manifest --manifest <FILE> --trace <FILE> --pilot
/// <FILE> --baseline <FILE>`: re-verify a frozen manifest against
/// live bytes and print the typed verdict (frozen-campaign 03).
/// Exit 0 on Valid, exit 1 on any stale verdict or unreadable input.
fn verify_manifest_cmd(rest: &[String]) -> ExitCode {
    let mut manifest: Option<&str> = None;
    let mut trace: Option<&str> = None;
    let mut pilot: Option<&str> = None;
    let mut baseline: Option<&str> = None;
    let mut args = rest.iter();
    while let Some(flag) = args.next() {
        let value = match args.next() {
            Some(v) if !v.is_empty() && !v.starts_with('-') => v,
            _ => return usage_error(),
        };
        match flag.as_str() {
            "--manifest" if manifest.is_none() => manifest = Some(value),
            "--trace" if trace.is_none() => trace = Some(value),
            "--pilot" if pilot.is_none() => pilot = Some(value),
            "--baseline" if baseline.is_none() => baseline = Some(value),
            _ => return usage_error(),
        }
    }
    let (Some(manifest), Some(trace), Some(pilot), Some(baseline)) =
        (manifest, trace, pilot, baseline)
    else {
        return usage_error();
    };
    let read = |p: &str| match std::fs::read(p) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: read {p}: {e}");
            None
        }
        Ok(b) => Some(b),
    };
    let (Some(manifest_bytes), Some(trace_bytes), Some(pilot_bytes), Some(baseline_bytes)) =
        (read(manifest), read(trace), read(pilot), read(baseline))
    else {
        return ExitCode::from(1);
    };
    let parsed: hephaestus::campaign::FrozenManifest = match serde_json::from_slice(&manifest_bytes)
    {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: manifest parse: {e}");
            return ExitCode::from(1);
        }
        Ok(m) => m,
    };
    let verdict =
        hephaestus::campaign::verify_manifest(&parsed, &trace_bytes, &pilot_bytes, &baseline_bytes);
    println!("{}", serde_json::json!({ "verdict": verdict }));
    if verdict == hephaestus::campaign::ManifestVerdict::Valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
