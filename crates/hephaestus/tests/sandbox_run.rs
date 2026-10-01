//! T-010 ticket 03 — worker runner e2e and safe output collection
//! (seam: `hephaestus::sandbox` + `hephaestus_workers`).
//!
//! R-059/AT-059 positive case: authorized bounded local worker with
//! collected outputs; symlink/oversize exfil classes refused or truncated.
//! Tests HARD-REQUIRE bwrap (never skip).

use hephaestus::sandbox::{IsolationSpec, NetworkPolicy, RunSpec, SandboxError, SandboxProvider};

fn crate_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn python_pkg() -> std::path::PathBuf {
    crate_dir()
        .parent()
        .expect("crates")
        .parent()
        .expect("workspace")
        .join("python")
}

fn base_spec() -> IsolationSpec {
    IsolationSpec {
        declared_tools: vec!["python3".to_string()],
        env_allowlist: vec![(
            "PYTHONPATH".to_string(),
            "/opt/hephaestus_python".to_string(),
        )],
        extra_ro_binds: vec![(
            python_pkg(),
            std::path::PathBuf::from("/opt/hephaestus_python"),
        )],
        wall_timeout_ms: 15_000,
        memory_bytes: 512 * 1024 * 1024,
        cpu_secs: 10,
        nproc: 64,
        fsize_bytes: 64 * 1024 * 1024,
        network: NetworkPolicy::Off,
        max_stdout: 64 * 1024,
        max_stderr: 64 * 1024,
        max_output_files: 8,
        max_output_file_bytes: 4096,
        max_output_bytes: 32 * 1024,
        keep_workdir: false,
    }
}

fn job_json(tool: &str) -> Vec<u8> {
    serde_json::json!({
        "tool": tool,
        "declared_tools": [
            "hephaestus_workers.tools:echo",
            "hephaestus_workers.tools:fail",
            "hephaestus_workers.tools:write_out",
        ],
        "input": {"msg": "hi"},
    })
    .to_string()
    .into_bytes()
}

fn run_worker(tool: &str) -> Result<hephaestus::sandbox::RunOutcome, SandboxError> {
    SandboxProvider::with_default_tools().run(RunSpec {
        argv: vec![
            "python3".to_string(),
            "-m".to_string(),
            "hephaestus_workers.runner".to_string(),
            "/work/job.json".to_string(),
        ],
        spec: base_spec(),
        input_files: vec![("job.json".to_string(), job_json(tool))],
    })
}

#[test]
fn worker_runs_in_sandbox_and_result_json_is_collected() {
    // AT-059 positive: bounded worker + collected outputs.
    let out = run_worker("hephaestus_workers.tools:echo").expect("worker run");
    let result = out
        .output_files
        .iter()
        .find(|(name, _)| name == "result.json")
        .expect("result.json collected");
    let parsed: serde_json::Value = serde_json::from_slice(&result.1).expect("valid json");
    assert_eq!(parsed["status"], "ok");
    assert_eq!(parsed["result"]["msg"], "hi");
    assert!(!out.outputs_truncated);
    // Attestation rode along.
    assert!(out.attestation.bwrap_version.contains("bubblewrap"));
}

#[test]
fn undeclared_job_tool_is_denied_by_the_worker() {
    // Defense in depth: the runner enforces the job's declared list too.
    let err = run_worker("os.system").expect_err("undeclared tool must deny");
    match err {
        SandboxError::WorkerFailed {
            exit_code, stderr, ..
        } => {
            assert_eq!(exit_code, 2, "worker's undeclared-tool exit: {stderr}");
            assert!(stderr.contains("TOOL_DENIED"), "{stderr}");
        }
        other => panic!("expected WorkerFailed(2), got {other:?}"),
    }
}

#[test]
fn failing_tool_surfaces_as_worker_failure() {
    let err = run_worker("hephaestus_workers.tools:fail").expect_err("must fail");
    match err {
        SandboxError::WorkerFailed {
            exit_code, stderr, ..
        } => {
            assert_eq!(exit_code, 1);
            assert!(stderr.contains("TOOL_FAILED"), "{stderr}");
        }
        other => panic!("expected WorkerFailed(1), got {other:?}"),
    }
}

#[test]
fn symlinked_outputs_are_refused_wholesale() {
    // MASTER_SPEC:389 symlink/exfil class: a symlink in /work/out refuses
    // the entire collection — nothing is copied.
    let err = SandboxProvider::with_default_tools()
        .run(RunSpec {
            argv: vec![
                "python3".to_string(),
                "-c".to_string(),
                "import os; open('/work/out/ok.txt','w').write('x'); os.symlink('/etc/passwd','/work/out/leak')".to_string(),
            ],
            spec: base_spec(),
            input_files: vec![],
        })
        .expect_err("symlink must refuse collection");
    assert!(
        matches!(&err, SandboxError::OutputRefused { name } if name == "leak"),
        "{err:?}"
    );
}

#[test]
fn output_caps_truncate_never_crash() {
    // Count cap: 5 files, max_output_files = 2.
    let mut spec = base_spec();
    spec.max_output_files = 2;
    let out = SandboxProvider::with_default_tools()
        .run(RunSpec {
            argv: vec![
                "python3".to_string(),
                "-c".to_string(),
                "[open('/work/out/f%d.txt' % i,'w').write('x'*10) for i in range(5)]".to_string(),
            ],
            spec,
            input_files: vec![],
        })
        .expect("run");
    assert_eq!(out.output_files.len(), 2, "count cap holds");
    assert!(out.outputs_truncated, "truncation flagged");

    // Per-file cap: 100-byte content, cap 16.
    let mut spec = base_spec();
    spec.max_output_file_bytes = 16;
    let out = SandboxProvider::with_default_tools()
        .run(RunSpec {
            argv: vec![
                "python3".to_string(),
                "-c".to_string(),
                "open('/work/out/big.txt','w').write('y'*100)".to_string(),
            ],
            spec,
            input_files: vec![],
        })
        .expect("run");
    let big = out
        .output_files
        .iter()
        .find(|(n, _)| n == "big.txt")
        .expect("collected");
    assert_eq!(big.1.len(), 16, "per-file cap holds");
    assert!(out.outputs_truncated);
}

#[test]
fn declared_tool_can_publish_an_artifact_file() {
    // The worker's write_out demo places a file through /work/out.
    let mut job = serde_json::from_value::<serde_json::Value>(
        serde_json::from_str(
            &String::from_utf8(job_json("hephaestus_workers.tools:write_out")).expect("utf8"),
        )
        .expect("parse"),
    )
    .expect("value");
    job["input"] = serde_json::json!({"name": "artifact.txt", "content": "payload-bytes"});
    let out = SandboxProvider::with_default_tools()
        .run(RunSpec {
            argv: vec![
                "python3".to_string(),
                "-m".to_string(),
                "hephaestus_workers.runner".to_string(),
                "/work/job.json".to_string(),
            ],
            spec: base_spec(),
            input_files: vec![("job.json".to_string(), job.to_string().into_bytes())],
        })
        .expect("run");
    let names: Vec<&str> = out.output_files.iter().map(|(n, _)| n.as_str()).collect();
    assert!(names.contains(&"artifact.txt"), "collected: {names:?}");
    let art = out
        .output_files
        .iter()
        .find(|(n, _)| n == "artifact.txt")
        .expect("artifact");
    assert_eq!(art.1, b"payload-bytes".to_vec());
}

// --- Ticket 04: scheduler adapter, glossary, CI, gates ---

use hephaestus::budget::BudgetLedger;
use hephaestus::contracts::generated::Money;
use hephaestus::sandbox::SandboxExecutor;
use hephaestus::scheduler::{
    PriorityClass, RetryPolicy, Scheduler, SchedulerConfig, Task, TaskDag, TaskExecutor,
};

fn usd(minor_units: i64) -> Money {
    Money {
        currency: "USD".to_string(),
        minor_units,
    }
}

fn worker_spec() -> IsolationSpec {
    let mut spec = base_spec();
    // The scheduler supplies the wall deadline per task; base value unused.
    spec.wall_timeout_ms = 15_000;
    spec
}

#[test]
fn scheduler_runs_real_sandboxed_work_with_budget_settled() {
    // The integrated execution path: T-009 DAG -> SandboxExecutor ->
    // bwrap -> worker -> outcomes -> BudgetLedger (R-055/AT-055).
    let mut dag = TaskDag::new(vec!["T1".to_string(), "T2".to_string()]);
    for (id, cost) in [("T1", 100), ("T2", 50)] {
        let mut t = Task {
            id: id.to_string(),
            priority: PriorityClass::Normal,
            depends_on: vec![],
            inputs: vec![],
            outputs: vec![format!("out-{id}")],
            retry: RetryPolicy { max_attempts: 1 },
            timeout_ms: 20_000,
            cost: usd(cost),
            trivial: false,
            retryable: true,
            resource_units: 0,
            exclusive: false,
        };
        t.inputs = vec![id.to_string()];
        dag.add(t).unwrap();
    }
    dag.validate().expect("valid dag");
    let executor = SandboxExecutor::new(
        SandboxProvider::with_default_tools(),
        "hephaestus_workers.tools:echo".to_string(),
        vec!["hephaestus_workers.tools:echo".to_string()],
        worker_spec(),
    );
    let mut sched = Scheduler::new(
        dag,
        BudgetLedger::new(usd(1000)).expect("budget"),
        SchedulerConfig {
            max_in_flight: 2,
            resource_capacity: 4,
        },
    )
    .expect("scheduler");
    let report = sched.run(&executor).expect("run");
    assert_eq!(report.succeeded().len(), 2, "failed: {:?}", report.failed());
    assert_eq!(
        sched.budget().spent(),
        usd(150),
        "reserve -> commit exactly"
    );
    assert_eq!(sched.budget().reserved(), usd(0));
}

#[test]
fn sandbox_timeout_maps_to_timed_out_outcome() {
    // Wall kill inside the sandbox surfaces as T-009's TimedOut.
    let task = Task {
        id: "SPIN".to_string(),
        priority: PriorityClass::Normal,
        depends_on: vec![],
        inputs: vec![],
        outputs: vec!["out".to_string()],
        retry: RetryPolicy { max_attempts: 1 },
        timeout_ms: 700,
        cost: usd(0),
        trivial: false,
        retryable: true,
        resource_units: 0,
        exclusive: false,
    };
    let executor = SandboxExecutor::new(
        SandboxProvider::with_default_tools(),
        "hephaestus_workers.tools:spin".to_string(),
        vec!["hephaestus_workers.tools:spin".to_string()],
        worker_spec(),
    );
    let outcome = executor.run(&task);
    assert!(
        matches!(outcome, hephaestus::scheduler::TaskOutcome::TimedOut),
        "{outcome:?}"
    );
}

#[test]
fn provider_denials_surface_as_failed_tasks() {
    let task = Task {
        id: "DENY".to_string(),
        priority: PriorityClass::Normal,
        depends_on: vec![],
        inputs: vec![],
        outputs: vec!["out".to_string()],
        retry: RetryPolicy { max_attempts: 1 },
        timeout_ms: 5_000,
        cost: usd(0),
        trivial: false,
        retryable: true,
        resource_units: 0,
        exclusive: false,
    };
    let executor = SandboxExecutor::new(
        SandboxProvider::with_tools(
            std::path::PathBuf::from("/nonexistent/bwrap"),
            std::path::PathBuf::from("/usr/bin/prlimit"),
        ),
        "hephaestus_workers.tools:echo".to_string(),
        vec!["hephaestus_workers.tools:echo".to_string()],
        worker_spec(),
    );
    let outcome = executor.run(&task);
    match outcome {
        hephaestus::scheduler::TaskOutcome::Failed { reason } => {
            assert!(reason.starts_with("SANDBOX: "), "{reason}");
            assert!(reason.contains("attestation"), "{reason}");
        }
        other => panic!("expected Failed(attestation), got {other:?}"),
    }
}

#[test]
fn traversal_input_names_are_refused_not_sanitized() {
    // RunSpec doc: traversal names are REFUSED (review pass 1: they were
    // silently sanitized to the basename).
    let err = SandboxProvider::with_default_tools()
        .run(RunSpec {
            argv: vec![
                "python3".to_string(),
                "-c".to_string(),
                "print('x')".to_string(),
            ],
            spec: base_spec(),
            input_files: vec![("../evil.txt".to_string(), b"payload".to_vec())],
        })
        .expect_err("traversal input name must refuse");
    assert!(matches!(err, SandboxError::InvalidSpec { .. }), "{err:?}");
}

#[test]
fn total_output_budget_caps_collection() {
    // Count and per-file caps are tested elsewhere; total bytes here.
    let mut spec = base_spec();
    spec.max_output_files = 8;
    spec.max_output_file_bytes = 4096;
    spec.max_output_bytes = 150;
    let out = SandboxProvider::with_default_tools()
        .run(RunSpec {
            argv: vec![
                "python3".to_string(),
                "-c".to_string(),
                "[open('/work/out/f%d.txt' % i,'w').write(chr(122)*100) for i in range(3)]"
                    .to_string(),
            ],
            spec,
            input_files: vec![],
        })
        .expect("run");
    let total: usize = out.output_files.iter().map(|(_, b)| b.len()).sum();
    assert!(total <= 150, "total budget exceeded: {total}");
    assert!(out.outputs_truncated, "total-cap truncation flagged");
}
