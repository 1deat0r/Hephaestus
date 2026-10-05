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
    assert!(text.contains("fixture canary-watch"), "{text}");
}

#[test]
fn canary_watch_stops_the_breached_rollout_with_a_rollback_receipt() {
    // dev-roadmap ticket 04: the monitor as a command -- twin runs over
    // the fixed observation stream are byte-identical and carry the stop
    // outcome plus the rollback receipt restoring the proven incumbent.
    let run_once = || bin().args(["fixture", "canary-watch"]).output().expect("spawn");
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
