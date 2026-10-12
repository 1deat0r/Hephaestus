//! T-M1-exit ticket 01 — the real `hephaestus` binary (seam: argv + JSON).
//!
//! IMPLEMENTATION_PLAN:44 (M1 Exit) — run half. Tests spawn the built
//! binary; no internal reach-ins.

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hephaestus"))
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("m1cli-{}-{}-{}", tag, std::process::id(), nanos));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

#[test]
fn fixture_run_exits_zero_and_reports_the_pillar_signals() {
    let dir = temp_dir("run");
    let out = bin()
        .args(["fixture", "run", "--state-dir"])
        .arg(&dir)
        .output()
        .expect("spawn");
    assert!(
        out.status.success(),
        "run failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let summary: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json summary");
    assert_eq!(summary["fixture"], "m1-exit");
    // Five scripted tasks, states derived from replay:
    let states: Vec<(&str, &str)> = summary["tasks"]
        .as_array()
        .expect("tasks array")
        .iter()
        .map(|t| {
            (
                t["id"].as_str().expect("id"),
                t["state"].as_str().expect("state"),
            )
        })
        .collect();
    assert_eq!(
        states,
        vec![
            ("alpha", "succeeded"),
            ("beta", "succeeded"),
            ("gamma", "failed"),
            ("delta", "ambiguous"),
            // Budget refusal happens BEFORE dispatch: the recorder truth
            // is `planned` (nothing executed) — the refusal itself is the
            // budget.refused counter below.
            ("epsilon", "planned"),
        ]
    );
    // Budget pillar: refusal accounted; spent/unresolved within limit.
    assert_eq!(summary["budget"]["limit"], 100);
    assert_eq!(summary["budget"]["spent"], 60);
    assert_eq!(summary["budget"]["unresolved"], 40);
    assert_eq!(summary["budget"]["refused"], 1);
    assert_eq!(summary["ambiguous"], 1, "delta is the ambiguity count");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn twin_fresh_runs_are_byte_identical() {
    // Determinism (grill Q3): same fixture, two fresh dirs, same bytes.
    let a = temp_dir("twin-a");
    let b = temp_dir("twin-b");
    let run = |dir: &std::path::Path| {
        let out = bin()
            .args(["fixture", "run", "--state-dir"])
            .arg(dir)
            .output()
            .expect("spawn");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        out.stdout
    };
    let first = run(&a);
    let second = run(&b);
    assert_eq!(first, second, "summaries must be byte-identical");
    let _ = std::fs::remove_dir_all(&a);
    let _ = std::fs::remove_dir_all(&b);
}

#[test]
fn help_documents_both_subcommands() {
    let out = bin().arg("--help").output().expect("spawn");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("fixture run"), "{text}");
    assert!(text.contains("fixture recover"), "{text}");
    assert!(text.contains("fixture mission-run"), "{text}");
    assert!(text.contains("fixture pilot-batch"), "{text}");
    assert!(text.contains("fixture assemble-release"), "{text}");
    assert!(text.contains("fixture freeze-campaign"), "{text}");
    assert!(text.contains("fixture verify-manifest"), "{text}");
    assert!(text.contains("fixture canary-watch"), "{text}");
}

#[test]
fn canary_watch_stops_the_breached_rollout_with_a_rollback_receipt() {
    // dev-roadmap ticket 04: the monitor as a command -- twin runs over
    // the fixed observation stream are byte-identical and carry the stop
    // outcome plus the rollback receipt restoring the proven incumbent.
    let run_once = || {
        bin()
            .args(["fixture", "canary-watch"])
            .output()
            .expect("spawn")
    };
    let a = run_once();
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    let summary: serde_json::Value = serde_json::from_slice(&a.stdout).expect("json");
    assert_eq!(summary["outcome"], "stopped");
    assert_eq!(summary["checks"], 3);
    assert_eq!(summary["rollback"]["restored_incumbent_digest"], "d8");
    assert_eq!(summary["rollback"]["rolled_back_challenger_digest"], "d16");
    let b = run_once();
    assert_eq!(a.stdout, b.stdout, "twin runs byte-identical");
}

#[test]
fn mission_run_prints_byte_identical_receipts_twice() {
    // T-061 ticket 03: the E2E chain as a command — twin runs over the
    // world-derived trace fixture are byte-identical and carry both the
    // supported and the honest-negative receipts.
    let trace = format!(
        "{}/tests/fixtures/e2e-trace.log",
        env!("CARGO_MANIFEST_DIR")
    );
    let run_once = || {
        bin()
            .args(["fixture", "mission-run", "--trace"])
            .arg(&trace)
            .output()
            .expect("spawn")
    };
    let a = run_once();
    assert!(
        a.status.success(),
        "mission-run exits 0: {}",
        String::from_utf8_lossy(&a.stderr)
    );
    let text = String::from_utf8_lossy(&a.stdout);
    assert!(text.contains("Supported"), "supported receipt: {text}");
    assert!(text.contains("Contradicted"), "negative receipt: {text}");
    assert!(text.contains("receipt_sha256"), "bound receipt: {text}");
    let b = run_once();
    assert_eq!(a.stdout, b.stdout, "twin runs byte-identical");
}

#[test]
fn unknown_subcommand_exits_two_with_usage_on_stderr() {
    let out = bin().arg("frobnicate").output().expect("spawn");
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.to_lowercase().contains("usage"), "{err}");
    assert!(out.stdout.is_empty(), "usage belongs on stderr");
}

#[test]
fn run_refuses_a_non_empty_state_dir() {
    let dir = temp_dir("nonempty");
    std::fs::write(dir.join("foreign.txt"), "x").expect("seed");
    let out = bin()
        .args(["fixture", "run", "--state-dir"])
        .arg(&dir)
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(1), "refuses to mix histories");
    let _ = std::fs::remove_dir_all(&dir);
}

// --- Ticket 02: recover + exit-pillar proof ---

#[test]
fn run_then_recover_round_trips_and_proves_all_four_pillars() {
    // IMPLEMENTATION_PLAN:44, asserted from real CLI output.
    let dir = temp_dir("roundtrip");
    let run_out = bin()
        .args(["fixture", "run", "--state-dir"])
        .arg(&dir)
        .output()
        .expect("spawn run");
    assert!(
        run_out.status.success(),
        "{}",
        String::from_utf8_lossy(&run_out.stderr)
    );
    let run_summary: serde_json::Value = serde_json::from_slice(&run_out.stdout).expect("run json");

    let rec_out = bin()
        .args(["fixture", "recover", "--state-dir"])
        .arg(&dir)
        .output()
        .expect("spawn recover");
    assert!(
        rec_out.status.success(),
        "{}",
        String::from_utf8_lossy(&rec_out.stderr)
    );
    let summary: serde_json::Value = serde_json::from_slice(&rec_out.stdout).expect("recover json");

    assert_eq!(summary["fixture"], "m1-exit");
    assert_eq!(summary["replayed_ops"], 5, "recorded replay: full history");

    // Pillar 1: recorded replay — recover states match run states.
    assert_eq!(summary["tasks"], run_summary["tasks"]);

    // Pillar 2: denied stays denied — gamma's terminal failure is NOT
    // requeued.
    let terminal: Vec<&str> = summary["plan"]["terminal"]
        .as_array()
        .expect("terminal")
        .iter()
        .map(|v| v.as_str().expect("id"))
        .collect();
    let requeue: Vec<&str> = summary["plan"]["requeue"]
        .as_array()
        .expect("requeue")
        .iter()
        .map(|v| v.as_str().expect("id"))
        .collect();
    let unresolved: Vec<&str> = summary["plan"]["unresolved"]
        .as_array()
        .expect("unresolved")
        .iter()
        .map(|v| v.as_str().expect("id"))
        .collect();
    assert!(
        terminal.contains(&"gamma"),
        "denied stays denied: {terminal:?}"
    );
    assert!(!requeue.contains(&"gamma"));
    assert!(
        summary["plan"]["corrupt"]
            .as_array()
            .expect("corrupt")
            .is_empty()
    );
    assert!(
        summary["plan"]["cancelled"]
            .as_array()
            .expect("cancelled")
            .is_empty()
    );

    // Pillar 3: bounded budgets — refusal recorded at run time; epsilon's
    // plan outcome is AT-057-correct requeue (never dispatched).
    assert_eq!(run_summary["budget"]["refused"], 1);
    assert_eq!(run_summary["budget"]["limit"], 100);
    assert!(
        requeue.contains(&"epsilon"),
        "planned => requeue: {requeue:?}"
    );

    // Pillar 4: ambiguous non-idempotent lands unresolved, never requeued.
    assert!(unresolved.contains(&"delta"), "unresolved: {unresolved:?}");
    assert!(!requeue.contains(&"delta"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recover_on_missing_state_exits_one() {
    let dir = temp_dir("missing");
    let _ = std::fs::remove_dir_all(&dir); // ensure absent
    let out = bin()
        .args(["fixture", "recover", "--state-dir"])
        .arg(&dir)
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("no fixture state"), "{err}");
}

#[test]
fn malformed_state_dir_arguments_exit_two() {
    // Missing value and flag-like values are usage errors, not dirs.
    for args in [
        vec!["fixture", "run", "--state-dir"],
        vec!["fixture", "run", "--state-dir", "--recover"],
        vec!["fixture", "recover"],
        vec!["fixture", "run", "--bogus", "/tmp"],
    ] {
        let out = bin().args(&args).output().expect("spawn");
        assert_eq!(out.status.code(), Some(2), "args {args:?} => usage error");
    }
}

fn trace_fixture() -> std::path::PathBuf {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.join("tests/fixtures/e2e-trace.log")
}

#[test]
fn pilot_batch_writes_preregistered_report() {
    // v1-release 01 S1: predeclared metrics in, report file out.
    let dir = temp_dir("pilot");
    let out = dir.join("pilot-report.md");
    let run = bin()
        .args(["fixture", "pilot-batch", "--trace"])
        .arg(trace_fixture())
        .arg("--out")
        .arg(&out)
        .output()
        .expect("spawn");
    assert!(
        run.status.success(),
        "pilot-batch failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let text = std::fs::read_to_string(&out).expect("report written");
    assert!(text.contains("prereg-fixture-campaign-1"), "plan named");
    assert!(text.contains("20 missions"), "batch size stated");
    assert!(
        text.contains("failures retained: 0"),
        "denominator explicit"
    );
    assert!(
        text.contains("paired-difference variance"),
        "variance recorded"
    );
    assert!(text.contains("provider spend: 0"), "cost honesty explicit");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn twin_pilot_batches_are_byte_identical() {
    // v1-release 01 S2: identical inputs, identical report bytes.
    let dir = temp_dir("pilot-twin");
    let a = dir.join("a.md");
    let b = dir.join("b.md");
    for out in [&a, &b] {
        let run = bin()
            .args(["fixture", "pilot-batch", "--trace"])
            .arg(trace_fixture())
            .arg("--out")
            .arg(out)
            .output()
            .expect("spawn");
        assert!(run.status.success());
    }
    assert_eq!(
        std::fs::read(&a).expect("a"),
        std::fs::read(&b).expect("b"),
        "deterministic report"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pilot_batch_rows_match_library_receipts() {
    // Review finding 8: the report tracks the real batch, not prose.
    let dir = temp_dir("pilot-rows");
    let out = dir.join("rows.md");
    let run = bin()
        .args(["fixture", "pilot-batch", "--trace"])
        .arg(trace_fixture())
        .arg("--out")
        .arg(&out)
        .output()
        .expect("spawn");
    assert!(run.status.success());
    let text = std::fs::read_to_string(&out).expect("report");
    let rows = text
        .lines()
        .filter(|l| l.starts_with("| ") && !l.contains("mission |") && !l.contains("---"))
        .count();
    assert_eq!(rows, 20, "one table row per mission");
    let trace = std::fs::read_to_string(trace_fixture()).expect("trace");
    let report = hephaestus::campaign::run_fixture_batch(&trace).expect("batch");
    let first = &report.missions[0];
    assert!(
        text.contains(&first.receipt_sha256),
        "report carries the actual run receipt"
    );
    assert!(text.contains(&format!("{:.6}", first.value)));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pilot_batch_accepts_swapped_flag_order() {
    // Review finding 9: output tracks the call, not flag positions.
    let dir = temp_dir("pilot-swap");
    let a = dir.join("a.md");
    let b = dir.join("b.md");
    for (path, order) in [(&a, true), (&b, false)] {
        let mut cmd = bin();
        if order {
            cmd.args(["fixture", "pilot-batch", "--trace"])
                .arg(trace_fixture())
                .arg("--out")
                .arg(path);
        } else {
            cmd.args(["fixture", "pilot-batch", "--out"])
                .arg(path)
                .arg("--trace")
                .arg(trace_fixture());
        }
        assert!(cmd.output().expect("spawn").status.success());
    }
    assert_eq!(std::fs::read(&a).expect("a"), std::fs::read(&b).expect("b"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pilot_batch_refuses_bad_args_and_existing_out() {
    // Review finding 10: malformed calls and overwrites fail closed.
    let dir = temp_dir("pilot-bad");
    let out = dir.join("bad.md");
    // Missing --out: usage error.
    let run = bin()
        .args(["fixture", "pilot-batch", "--trace"])
        .arg(trace_fixture())
        .output()
        .expect("spawn");
    assert_eq!(run.status.code(), Some(2));
    // Flag-like --out value: usage error.
    let run = bin()
        .args(["fixture", "pilot-batch", "--trace"])
        .arg(trace_fixture())
        .args(["--out", "--trace"])
        .output()
        .expect("spawn");
    assert_eq!(run.status.code(), Some(2));
    // Missing trace file: operational failure.
    let run = bin()
        .args(["fixture", "pilot-batch", "--trace"])
        .arg(dir.join("absent.log"))
        .arg("--out")
        .arg(&out)
        .output()
        .expect("spawn");
    assert_eq!(run.status.code(), Some(1));
    // Existing --out: refused, never truncated.
    std::fs::write(&out, "keep").expect("seed");
    let run = bin()
        .args(["fixture", "pilot-batch", "--trace"])
        .arg(trace_fixture())
        .arg("--out")
        .arg(&out)
        .output()
        .expect("spawn");
    assert_eq!(run.status.code(), Some(1));
    assert_eq!(std::fs::read_to_string(&out).expect("kept"), "keep");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn baseline_compare_writes_measured_report() {
    // v1-release 02 S1: deterministic arms measured, report out.
    let dir = temp_dir("baseline");
    let out = dir.join("baseline-report.md");
    let run = bin()
        .args(["fixture", "baseline-compare", "--trace"])
        .arg(trace_fixture())
        .arg("--out")
        .arg(&out)
        .output()
        .expect("spawn");
    assert!(
        run.status.success(),
        "baseline-compare failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let text = std::fs::read_to_string(&out).expect("report written");
    assert!(text.contains("matched envelopes"), "envelope claim");
    assert!(text.contains("correctness"), "correctness recorded");
    assert!(text.contains("false promotions: 0"), "promotion count");
    assert!(text.contains("tokens"), "tokens recorded");
    assert!(text.contains("provider spend: 0"), "cost honesty");
    assert!(text.contains("latency"), "latency recorded");
    assert!(text.contains("peak memory"), "memory recorded");
    assert!(
        text.contains("## Compute (metered local"),
        "compute section"
    );
    assert!(text.contains("wall total"), "wall totals");
    let wall_lines: Vec<&str> = text.lines().filter(|l| l.contains("wall total")).collect();
    assert_eq!(wall_lines.len(), 2, "one compute line per arm");
    for line in wall_lines {
        let after = line.split("wall total ").nth(1).expect("total value");
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        let nanos: u64 = digits.parse().expect("numeric wall total");
        assert!(nanos > 0, "measured wall time, not zero");
    }
    assert!(text.contains("cpu total"), "cpu totals or gap");
    assert!(text.contains("Model arms: absent"), "absence explicit");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn baseline_compare_deterministic_fields_replay() {
    // Measured fields vary run to run; deterministic aggregates replay.
    let dir = temp_dir("baseline-twin");
    let a = dir.join("a.md");
    let b = dir.join("b.md");
    for out in [&a, &b] {
        let run = bin()
            .args(["fixture", "baseline-compare", "--trace"])
            .arg(trace_fixture())
            .arg("--out")
            .arg(out)
            .output()
            .expect("spawn");
        assert!(run.status.success());
    }
    // Deterministic values replay; measured latency/throughput/memory
    // vary run to run and stay out of this comparison.
    let det = |p: &std::path::Path| {
        let text = std::fs::read_to_string(p).expect("read");
        let arms: Vec<&str> = text
            .lines()
            .filter(|l| l.starts_with("baseline ") || l.starts_with("candidate "))
            .collect();
        assert_eq!(arms.len(), 2, "two arm summary lines");
        let corr: Vec<&str> = arms
            .iter()
            .map(|l| {
                l.split("correctness ")
                    .nth(1)
                    .expect("value")
                    .split(',')
                    .next()
                    .expect("end")
            })
            .collect();
        let toks: Vec<&str> = arms
            .iter()
            .map(|l| {
                l.split("tokens: ")
                    .nth(1)
                    .expect("value")
                    .split(',')
                    .next()
                    .expect("end")
            })
            .collect();
        let fp = text.contains("false promotions: 0");
        (corr.join("|"), toks.join("|"), fp)
    };
    assert_eq!(det(&a), det(&b), "deterministic aggregates replay");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn baseline_compare_refuses_bad_args_and_existing_out() {
    // Review finding: malformed calls and overwrites fail closed.
    let dir = temp_dir("baseline-bad");
    let out = dir.join("bad.md");
    let run = bin()
        .args(["fixture", "baseline-compare", "--trace"])
        .arg(trace_fixture())
        .output()
        .expect("spawn");
    assert_eq!(run.status.code(), Some(2));
    let run = bin()
        .args(["fixture", "baseline-compare", "--trace"])
        .arg(trace_fixture())
        .args(["--out", "--trace"])
        .output()
        .expect("spawn");
    assert_eq!(run.status.code(), Some(2));
    // Missing --trace: usage error.
    let run = bin()
        .args(["fixture", "baseline-compare", "--out"])
        .arg(&out)
        .output()
        .expect("spawn");
    assert_eq!(run.status.code(), Some(2));
    std::fs::write(&out, "keep").expect("seed");
    let run = bin()
        .args(["fixture", "baseline-compare", "--trace"])
        .arg(trace_fixture())
        .arg("--out")
        .arg(&out)
        .output()
        .expect("spawn");
    assert_eq!(run.status.code(), Some(1));
    assert_eq!(std::fs::read_to_string(&out).expect("kept"), "keep");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn baseline_report_carries_run_receipts() {
    // Review finding: report links the digests it claims.
    let dir = temp_dir("baseline-receipts");
    let out = dir.join("r.md");
    let run = bin()
        .args(["fixture", "baseline-compare", "--trace"])
        .arg(trace_fixture())
        .arg("--out")
        .arg(&out)
        .output()
        .expect("spawn");
    assert!(run.status.success());
    let text = std::fs::read_to_string(&out).expect("report");
    let trace = std::fs::read_to_string(trace_fixture()).expect("trace");
    let report = hephaestus::campaign::run_fixture_batch(&trace).expect("batch");
    assert!(text.contains(&report.missions[0].receipt_sha256));
    assert!(text.contains(&report.missions[1].receipt_sha256));
    // A later subsample differs: the report tracks input, not prose.
    assert_ne!(
        report.missions[2].receipt_sha256,
        report.missions[0].receipt_sha256
    );
    assert!(text.contains(&report.missions[2].receipt_sha256));
    let _ = std::fs::remove_dir_all(&dir);
}

fn repo_file(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(name)
}

#[test]
fn assemble_release_writes_v1_dossier() {
    // v1-release 03 S1: packet assembles from campaign receipts.
    let dir = temp_dir("dossier");
    let out = dir.join("dossier.md");
    let run = bin()
        .args(["fixture", "assemble-release", "--trace"])
        .arg(trace_fixture())
        .arg("--pilot")
        .arg(repo_file("docs/PILOT_CAMPAIGN_REPORT.md"))
        .arg("--baseline")
        .arg(repo_file("docs/BASELINE_COMPARISON_REPORT.md"))
        .arg("--out")
        .arg(&out)
        .output()
        .expect("spawn");
    assert!(
        run.status.success(),
        "assemble-release failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let text = std::fs::read_to_string(&out).expect("dossier written");
    assert!(text.contains("EXPERIMENTAL"), "default label");
    assert!(text.contains("prereg-fixture-campaign-1"), "campaign bound");
    assert!(text.contains("false promotions: 0"), "comparison carried");
    assert!(text.contains("positive: 10"), "outcome counts");
    assert!(text.contains("negative: 10"), "honest negatives kept");
    assert!(text.contains("pilot-report:"), "benchmark receipt pinned");
    assert!(text.contains("= 1 fraction"), "measured value pinned");
    assert!(
        text.contains(std::env::consts::ARCH),
        "reference machine pinned"
    );
    assert!(
        text.contains("Unresolved research questions"),
        "questions visible"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn twin_dossiers_replay_deterministic_sections() {
    // v1-release 03 S2: deterministic sections replay; measured
    // sections stay labeled as varying.
    let dir = temp_dir("dossier-twin");
    let a = dir.join("a.md");
    let b = dir.join("b.md");
    for out in [&a, &b] {
        let run = bin()
            .args(["fixture", "assemble-release", "--trace"])
            .arg(trace_fixture())
            .arg("--pilot")
            .arg(repo_file("docs/PILOT_CAMPAIGN_REPORT.md"))
            .arg("--baseline")
            .arg(repo_file("docs/BASELINE_COMPARISON_REPORT.md"))
            .arg("--out")
            .arg(out)
            .output()
            .expect("spawn");
        assert!(run.status.success());
    }
    // Measured latency/throughput/memory vary; only the
    // deterministic values replay.
    let det = |p: &std::path::Path| {
        let text = std::fs::read_to_string(p).expect("read");
        let arms: Vec<&str> = text
            .lines()
            .filter(|l| l.starts_with("baseline ") || l.starts_with("candidate "))
            .collect();
        assert_eq!(arms.len(), 2, "two arm summary lines");
        let corr: Vec<&str> = arms
            .iter()
            .map(|l| {
                l.split("correctness ")
                    .nth(1)
                    .expect("value")
                    .split(',')
                    .next()
                    .expect("end")
            })
            .collect();
        let label = text
            .lines()
            .find(|l| l.starts_with("Scope label:"))
            .expect("label line");
        let range = text
            .lines()
            .find(|l| l.contains("observed cand-arm value range"))
            .expect("uncertainty range line");
        let digests: Vec<&str> = text
            .lines()
            .filter(|l| l.starts_with("- pilot report: ") || l.starts_with("- baseline report: "))
            .collect();
        assert_eq!(digests.len(), 2, "both input digests");
        let pins: Vec<&str> = text
            .lines()
            .filter(|l| l.starts_with("- fixture-cand-arm"))
            .collect();
        assert_eq!(pins.len(), 1, "benchmark pin line");
        let outcomes: Vec<&str> = text
            .lines()
            .filter(|l| {
                l.starts_with("positive:")
                    || l.starts_with("negative:")
                    || l.starts_with("inconclusive:")
                    || l.starts_with("invalid:")
                    || l.starts_with("blocked:")
            })
            .collect();
        assert_eq!(outcomes.len(), 5, "all five outcome classes");
        (
            corr.join("|"),
            label.to_string(),
            outcomes.join("|"),
            range.to_string(),
            digests.join("|"),
            pins.join("|"),
        )
    };
    assert_eq!(det(&a), det(&b), "deterministic sections replay");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn assemble_release_refuses_foreign_inputs() {
    // Finding 2: provenance markers gate; any other file refuses.
    let dir = temp_dir("dossier-foreign");
    let out = dir.join("f.md");
    let run = bin()
        .args(["fixture", "assemble-release", "--trace"])
        .arg(trace_fixture())
        .arg("--pilot")
        .arg(repo_file("docs/BASELINE_COMPARISON_REPORT.md"))
        .arg("--baseline")
        .arg(repo_file("docs/PILOT_CAMPAIGN_REPORT.md"))
        .arg("--out")
        .arg(&out)
        .output()
        .expect("spawn");
    assert_eq!(run.status.code(), Some(1));
    assert!(!out.exists(), "no dossier from foreign inputs");
    let _ = std::fs::remove_dir_all(&dir);
}

fn det_lines(text: &str) -> String {
    // Deterministic sections shared by the committed reports:
    // correctness, tokens, costs, counts, labels, ranges, run
    // digests, pins. Measured latency/throughput/memory/compute
    // stay out. Dossier input-digest provenance lines stay out here;
    // callers normalize them as run-local before comparing.
    text.lines()
        .filter(|l| {
            l.starts_with("baseline ")
                || l.starts_with("candidate ")
                || l.starts_with("positive:")
                || l.starts_with("negative:")
                || l.starts_with("inconclusive:")
                || l.starts_with("invalid:")
                || l.starts_with("blocked:")
                || l.starts_with("Scope label:")
                || l.contains("observed cand-arm value range")
                || l.starts_with("- pilot report: ")
                || l.starts_with("- baseline report: ")
                || l.starts_with("- fixture-cand-arm")
                || l.starts_with("false promotions:")
                || l.starts_with("base runs:")
                || l.starts_with("candidate runs:")
                || (l.starts_with("- base ") && !l.contains("compute:"))
                || (l.starts_with("- cand ") && !l.contains("compute:"))
                || l.starts_with("- tokens:")
                || l.contains("failures retained: ")
                || l.contains("paired-difference variance")
                || l.starts_with("- arm ")
        })
        .map(|l| {
            // Strip measured values riding on deterministic lines.
            let mut kept = l.to_string();
            for marker in ["latency total ", "throughput mean ", "peak memory max "] {
                if let Some((head, _)) = kept.split_once(marker) {
                    kept = head.to_string();
                }
            }
            kept
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn reproduce_v1_committed() {
    // repro-v1 01: committed reports regenerate from the trace.
    // Pilot bytes match exactly; baseline and dossier match on
    // deterministic sections. Measured sections stay out by design.
    let dir = temp_dir("repro-v1");
    let pilot = dir.join("pilot.md");
    let baseline = dir.join("baseline.md");
    let dossier = dir.join("dossier.md");
    let run = bin()
        .args(["fixture", "pilot-batch", "--trace"])
        .arg(trace_fixture())
        .arg("--out")
        .arg(&pilot)
        .output()
        .expect("spawn");
    assert!(run.status.success());
    let run = bin()
        .args(["fixture", "baseline-compare", "--trace"])
        .arg(trace_fixture())
        .arg("--out")
        .arg(&baseline)
        .output()
        .expect("spawn");
    assert!(run.status.success());
    let run = bin()
        .args(["fixture", "assemble-release", "--trace"])
        .arg(trace_fixture())
        .arg("--pilot")
        .arg(&pilot)
        .arg("--baseline")
        .arg(&baseline)
        .arg("--out")
        .arg(&dossier)
        .output()
        .expect("spawn");
    assert!(run.status.success());
    let committed =
        |name: &str| std::fs::read_to_string(repo_file(name)).expect("committed report");
    // Pilot: byte-identical.
    assert_eq!(
        std::fs::read(&pilot).expect("fresh pilot"),
        std::fs::read(repo_file("docs/PILOT_CAMPAIGN_REPORT.md")).expect("committed pilot"),
        "pilot reproduces byte-identical"
    );
    // Baseline + dossier: deterministic sections replay. The dossier
    // regenerates against fresh inputs, so its input-digest lines
    // track the fresh files, not the committed ones: compare with
    // the fresh pilot/baseline digests in place.
    assert_eq!(
        det_lines(&std::fs::read_to_string(&baseline).expect("fresh baseline")),
        det_lines(&committed("docs/BASELINE_COMPARISON_REPORT.md")),
        "baseline deterministic sections replay"
    );
    let fresh_dossier = std::fs::read_to_string(&dossier).expect("fresh dossier");
    let committed_dossier = committed("docs/RELEASE_DOSSIER_V1.md");
    let normalize = |text: &str| {
        text.lines()
            .filter(|l| !l.starts_with("- pilot report: ") && !l.starts_with("- baseline report: "))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(
        det_lines(&normalize(&fresh_dossier)),
        det_lines(&normalize(&committed_dossier)),
        "dossier deterministic sections replay"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn freeze_campaign_writes_manifest() {
    // frozen-campaign 01 S1: inputs bound in one manifest.
    let dir = temp_dir("freeze");
    let out = dir.join("manifest.json");
    let run = bin()
        .args(["fixture", "freeze-campaign", "--trace"])
        .arg(trace_fixture())
        .arg("--pilot")
        .arg(repo_file("docs/PILOT_CAMPAIGN_REPORT.md"))
        .arg("--baseline")
        .arg(repo_file("docs/BASELINE_COMPARISON_REPORT.md"))
        .arg("--out")
        .arg(&out)
        .output()
        .expect("spawn");
    assert!(
        run.status.success(),
        "freeze failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&out).expect("manifest")).expect("json");
    assert_eq!(manifest["preregistration_id"], "prereg-fixture-campaign-1");
    assert_eq!(manifest["scope"], "fixture (not confirmatory evidence)");
    assert_eq!(manifest["package_version"], env!("CARGO_PKG_VERSION"));
    let trace = std::fs::read(trace_fixture()).expect("trace");
    assert_eq!(
        manifest["trace_sha256"],
        hephaestus::knowledge::sha256_hex(&trace).as_str()
    );
    assert!(manifest["frozen"]["plan"].is_object(), "plan frozen");
    assert!(
        manifest["provenance"]["toolchain"].is_string(),
        "provenance labeled"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn twin_freezes_match_on_frozen_section() {
    // Frozen content replays; provenance may vary by machine.
    let dir = temp_dir("freeze-twin");
    let a = dir.join("a.json");
    let b = dir.join("b.json");
    for out in [&a, &b] {
        let run = bin()
            .args(["fixture", "freeze-campaign", "--trace"])
            .arg(trace_fixture())
            .arg("--pilot")
            .arg(repo_file("docs/PILOT_CAMPAIGN_REPORT.md"))
            .arg("--baseline")
            .arg(repo_file("docs/BASELINE_COMPARISON_REPORT.md"))
            .arg("--out")
            .arg(out)
            .output()
            .expect("spawn");
        assert!(run.status.success());
    }
    let frozen = |p: &std::path::Path| {
        let v: serde_json::Value =
            serde_json::from_slice(&std::fs::read(p).expect("read")).expect("json");
        v["frozen"].clone()
    };
    assert_eq!(frozen(&a), frozen(&b), "frozen section replays");
    assert_eq!(
        std::fs::read(&a).expect("a bytes"),
        std::fs::read(&b).expect("b bytes"),
        "same-machine byte identity"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn freeze_campaign_refuses_foreign_and_bad_args() {
    // Review findings: refusal paths need proof.
    let dir = temp_dir("freeze-bad");
    let out = dir.join("f.json");
    // Foreign pilot file: refused.
    let run = bin()
        .args(["fixture", "freeze-campaign", "--trace"])
        .arg(trace_fixture())
        .arg("--pilot")
        .arg(trace_fixture())
        .arg("--baseline")
        .arg(repo_file("docs/BASELINE_COMPARISON_REPORT.md"))
        .arg("--out")
        .arg(&out)
        .output()
        .expect("spawn");
    assert_eq!(run.status.code(), Some(1));
    // Missing --out: usage error.
    let run = bin()
        .args(["fixture", "freeze-campaign", "--trace"])
        .arg(trace_fixture())
        .arg("--pilot")
        .arg(repo_file("docs/PILOT_CAMPAIGN_REPORT.md"))
        .arg("--baseline")
        .arg(repo_file("docs/BASELINE_COMPARISON_REPORT.md"))
        .output()
        .expect("spawn");
    assert_eq!(run.status.code(), Some(2));
    // Existing --out: refused, never truncated.
    std::fs::write(&out, "keep").expect("seed");
    let run = bin()
        .args(["fixture", "freeze-campaign", "--trace"])
        .arg(trace_fixture())
        .arg("--pilot")
        .arg(repo_file("docs/PILOT_CAMPAIGN_REPORT.md"))
        .arg("--baseline")
        .arg(repo_file("docs/BASELINE_COMPARISON_REPORT.md"))
        .arg("--out")
        .arg(&out)
        .output()
        .expect("spawn");
    assert_eq!(run.status.code(), Some(1));
    assert_eq!(std::fs::read_to_string(&out).expect("kept"), "keep");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn freeze_manifest_digests_match_input_bytes() {
    // Review findings: digest equality needs proof.
    let dir = temp_dir("freeze-digest");
    let out = dir.join("m.json");
    let run = bin()
        .args(["fixture", "freeze-campaign", "--trace"])
        .arg(trace_fixture())
        .arg("--pilot")
        .arg(repo_file("docs/PILOT_CAMPAIGN_REPORT.md"))
        .arg("--baseline")
        .arg(repo_file("docs/BASELINE_COMPARISON_REPORT.md"))
        .arg("--out")
        .arg(&out)
        .output()
        .expect("spawn");
    assert!(run.status.success());
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&out).expect("manifest")).expect("json");
    let sha = hephaestus::knowledge::sha256_hex;
    let trace = std::fs::read(trace_fixture()).expect("trace");
    let pilot = std::fs::read(repo_file("docs/PILOT_CAMPAIGN_REPORT.md")).expect("pilot");
    let baseline =
        std::fs::read(repo_file("docs/BASELINE_COMPARISON_REPORT.md")).expect("baseline");
    assert_eq!(manifest["trace_sha256"], sha(&trace).as_str());
    assert_eq!(manifest["pilot_report_digest"], sha(&pilot).as_str());
    assert_eq!(manifest["baseline_report_digest"], sha(&baseline).as_str());
    assert_eq!(manifest["frozen"]["trace_sha256"], sha(&trace).as_str());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn verify_manifest_cmd() {
    // frozen-campaign 03 S1: verdicts surface end to end.
    let dir = temp_dir("verify-manifest");
    // Valid path: the committed manifest against live inputs.
    let run = bin()
        .args(["fixture", "verify-manifest", "--manifest"])
        .arg(repo_file("docs/CAMPAIGN_MANIFEST.json"))
        .arg("--trace")
        .arg(trace_fixture())
        .arg("--pilot")
        .arg(repo_file("docs/PILOT_CAMPAIGN_REPORT.md"))
        .arg("--baseline")
        .arg(repo_file("docs/BASELINE_COMPARISON_REPORT.md"))
        .output()
        .expect("spawn");
    assert!(
        run.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let verdict: serde_json::Value = serde_json::from_slice(&run.stdout).expect("verdict json");
    assert_eq!(verdict["verdict"], "Valid");
    // Stale path: a tampered trace refuses with the field named.
    let mut tampered = std::fs::read(trace_fixture()).expect("trace");
    tampered.push(b'x');
    let bad_trace = dir.join("bad-trace.log");
    std::fs::write(&bad_trace, tampered).expect("write");
    let run = bin()
        .args(["fixture", "verify-manifest", "--manifest"])
        .arg(repo_file("docs/CAMPAIGN_MANIFEST.json"))
        .arg("--trace")
        .arg(&bad_trace)
        .arg("--pilot")
        .arg(repo_file("docs/PILOT_CAMPAIGN_REPORT.md"))
        .arg("--baseline")
        .arg(repo_file("docs/BASELINE_COMPARISON_REPORT.md"))
        .output()
        .expect("spawn");
    assert_eq!(run.status.code(), Some(1));
    let verdict: serde_json::Value = serde_json::from_slice(&run.stdout).expect("verdict json");
    assert_eq!(
        verdict["verdict"]["DigestMismatch"]["field"],
        "trace_sha256"
    );
    // Missing --manifest: usage error.
    let run = bin()
        .args(["fixture", "verify-manifest", "--trace"])
        .arg(trace_fixture())
        .output()
        .expect("spawn");
    assert_eq!(run.status.code(), Some(2));
    let _ = std::fs::remove_dir_all(&dir);
}
