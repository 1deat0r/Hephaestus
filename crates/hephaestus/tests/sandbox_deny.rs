//! T-010 ticket 01 — sandbox refusals and boundary probes (deny-first,
//! seam: `hephaestus::sandbox`).
//!
//! Obligations: R-059/AT-059 (sandbox escape, secret leakage, undeclared
//! networking), R-110/AT-110 (attestation denies dispatch). MASTER_SPEC:389
//! failure classes. Tests HARD-REQUIRE bwrap — absence fails, never skips.

use hephaestus::sandbox::{IsolationSpec, NetworkPolicy, RunSpec, SandboxError, SandboxProvider};

/// Provider with default tool resolution (bwrap/prlimit from PATH).
fn provider() -> SandboxProvider {
    SandboxProvider::with_default_tools()
}

fn base_spec() -> IsolationSpec {
    IsolationSpec {
        declared_tools: vec!["python3".to_string()],
        env_allowlist: vec![],
        extra_ro_binds: vec![],
        wall_timeout_ms: 10_000,
        memory_bytes: 512 * 1024 * 1024,
        cpu_secs: 10,
        nproc: 64,
        fsize_bytes: 64 * 1024 * 1024,
        network: NetworkPolicy::Off,
        max_stdout: 64 * 1024,
        max_stderr: 64 * 1024,
        max_output_files: 16,
        max_output_file_bytes: 1024 * 1024,
        max_output_bytes: 8 * 1024 * 1024,
        keep_workdir: false,
    }
}

fn run(argv: &[&str]) -> Result<hephaestus::sandbox::RunOutcome, SandboxError> {
    let spec = base_spec();
    provider().run(RunSpec {
        argv: argv.iter().map(|s| s.to_string()).collect(),
        spec,
        input_files: vec![],
    })
}

#[test]
fn network_is_unreachable_inside_the_sandbox() {
    // R-059 / AT-059: undeclared networking must fail.
    let out = run(&[
        "python3",
        "-c",
        "import socket; s=socket.socket(); s.settimeout(3); s.connect(('1.1.1.1', 80))",
    ]);
    assert!(
        out.is_err(),
        "TCP connect must fail without a network: {out:?}"
    );

    let out = run(&[
        "python3",
        "-c",
        "import socket; print(socket.gethostbyname('example.com'))",
    ]);
    assert!(out.is_err(), "DNS must fail: {out:?}");
}

#[test]
fn host_environment_never_leaks_inside() {
    // R-059: environment-variable leakage (MASTER_SPEC:389).
    // SAFETY: single-threaded test process; we remove it immediately after.
    unsafe { std::env::set_var("T010_SECRET", "hunter2-host-value") };
    let out = run(&[
        "python3",
        "-c",
        "import os; print(os.environ.get('T010_SECRET', 'NONE'))",
    ])
    .expect("run");
    assert!(
        out.stdout.contains("NONE"),
        "secret leaked: {:?}",
        out.stdout
    );
    unsafe { std::env::remove_var("T010_SECRET") };
}

#[test]
fn mounts_are_read_only_except_workdir() {
    let out = run(&["python3", "-c", "open('/usr/T010-RO-test','w').write('x')"]);
    assert!(out.is_err(), "write to /usr must fail: {out:?}");

    let out = run(&[
        "python3",
        "-c",
        "open('ok.txt','w').write('x'); print('WROTE')",
    ])
    .expect("run");
    assert!(out.stdout.contains("WROTE"), "workdir must be writable");
}

#[test]
fn traversal_cannot_reach_host_bound_trees() {
    // Path traversal class from MASTER_SPEC:389: `../` from the workdir
    // cannot cross into a read-only HOST bind (/usr). The sandbox root
    // itself is an ephemeral tmpfs (documented limitation — writes there
    // are namespaced and vanish with the run; the host is unreachable
    // except through explicit binds).
    let out = run(&[
        "python3",
        "-c",
        "import os\ntry:\n os.open('/work/../usr/T010-escape', os.O_WRONLY|os.O_CREAT)\n print('ESCAPED')\nexcept OSError:\n print('DENIED')",
    ])
    .expect("run");
    assert!(
        out.stdout.contains("DENIED"),
        "traversal crossed into a host bind: {:?}",
        out.stdout
    );
    assert!(
        !std::path::Path::new("/usr/T010-escape").exists(),
        "host file created!"
    );
    // And a pure tmpfs-root write never reaches the host either.
    let out = run(&[
        "python3",
        "-c",
        "import os\nos.open('/T010-tmpfs-root', os.O_WRONLY|os.O_CREAT)\nprint('WROTE-ROOT')",
    ])
    .expect("run");
    assert!(out.stdout.contains("WROTE-ROOT"));
    assert!(
        !std::path::Path::new("/T010-tmpfs-root").exists(),
        "tmpfs-root write reached the host!"
    );
}

#[test]
fn undeclared_tools_never_spawn() {
    // base_spec declares ONLY python3 — anything else is undeclared.
    let err = provider()
        .run(RunSpec {
            argv: vec!["definitely-not-declared".to_string()],
            spec: base_spec(),
            input_files: vec![],
        })
        .expect_err("undeclared must deny");
    assert!(
        matches!(err, SandboxError::UndeclaredTool { .. }),
        "{err:?}"
    );
}

#[test]
fn attestation_denies_before_any_spawn() {
    // AT-110 static half: missing tool => dispatch denied.
    let broken = SandboxProvider::with_tools(
        std::path::PathBuf::from("/nonexistent/bwrap"),
        std::path::PathBuf::from("/usr/bin/prlimit"),
    );
    let err = broken
        .run(RunSpec {
            argv: vec!["python3".to_string()],
            spec: base_spec(),
            input_files: vec![],
        })
        .expect_err("missing bwrap must deny");
    assert!(
        matches!(err, SandboxError::AttestationFailed { .. }),
        "{err:?}"
    );

    // Spec violations also deny: non-Off network, zero limits.
    let mut spec = base_spec();
    spec.network = NetworkPolicy::On;
    let provider = provider();
    let err = provider
        .run(RunSpec {
            argv: vec!["python3".to_string()],
            spec,
            input_files: vec![],
        })
        .expect_err("network On unsupported");
    assert!(matches!(err, SandboxError::Unsupported { .. }), "{err:?}");

    let mut spec = base_spec();
    spec.wall_timeout_ms = 0;
    let err = provider
        .run(RunSpec {
            argv: vec!["python3".to_string()],
            spec,
            input_files: vec![],
        })
        .expect_err("zero timeout");
    assert!(matches!(err, SandboxError::InvalidSpec { .. }), "{err:?}");
}

#[test]
fn wall_timeout_kills_a_runaway() {
    let mut spec = base_spec();
    spec.wall_timeout_ms = 700;
    spec.cpu_secs = 60;
    let err = provider()
        .run(RunSpec {
            argv: vec![
                "python3".to_string(),
                "-c".to_string(),
                "while True: pass".to_string(),
            ],
            spec,
            input_files: vec![],
        })
        .expect_err("runaway must be killed");
    assert!(matches!(err, SandboxError::Timeout { .. }), "{err:?}");
}

// --- Ticket 02: DoS/dependency matrix and resource metering ---

#[test]
fn memory_bomb_dies_at_the_declared_bound() {
    let mut spec = base_spec();
    spec.memory_bytes = 64 * 1024 * 1024;
    let err = provider()
        .run(RunSpec {
            argv: vec![
                "python3".to_string(),
                "-c".to_string(),
                "x = bytearray(10**10)".to_string(),
            ],
            spec,
            input_files: vec![],
        })
        .expect_err("memory bomb must die");
    match err {
        SandboxError::WorkerFailed {
            exit_code, wall_ms, ..
        } => {
            assert!(exit_code != 0, "must fail nonzero");
            assert!(wall_ms < 10_000, "died boundedly in {wall_ms} ms");
        }
        other => panic!("expected bounded WorkerFailed, got {other:?}"),
    }
}

#[test]
fn fork_bomb_stops_at_the_nproc_allowance() {
    // RLIMIT_NPROC = host threads + spec.nproc: the payload gets exactly
    // spec.nproc NEW threads beyond what the host already runs.
    let mut spec = base_spec();
    spec.nproc = 8;
    let out = provider()
        .run(RunSpec {
            argv: vec![
                "python3".to_string(),
                "-c".to_string(),
                "import subprocess, sys\nok=0\nfail=0\n\
                 for i in range(200):\n\
                 \x20 try:\n\
                 \x20  subprocess.run([sys.executable,'-c','pass'],capture_output=True,timeout=5); ok+=1\n\
                 \x20 except Exception: fail+=1\n\
                 print(f'OK={{ok}} FAIL={{fail}}')".to_string(),
            ],
            spec,
            input_files: vec![],
        });
    match out {
        Ok(o) => {
            let fail = o
                .stdout
                .split("FAIL=")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse::<u32>().ok())
                .expect("FAIL count printed");
            assert!(fail > 0, "fork bomb ran unbounded: {:?}", o.stdout);
        }
        // The bomb may also exhaust the wall/cpu bound — still BOUNDED.
        Err(SandboxError::WorkerFailed { wall_ms, .. }) => {
            assert!(wall_ms < 20_000, "bounded bomb death in {wall_ms} ms");
        }
        Err(SandboxError::Timeout { wall_ms }) => {
            assert!(wall_ms < 20_000, "bounded timeout at {wall_ms} ms");
        }
        Err(other) => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn cpu_runaway_dies_at_the_rlimit_within_the_wall_deadline() {
    let mut spec = base_spec();
    spec.cpu_secs = 1;
    spec.wall_timeout_ms = 30_000; // rlimit must fire BEFORE the wall kill
    let err = provider()
        .run(RunSpec {
            argv: vec![
                "python3".to_string(),
                "-c".to_string(),
                "while True: pass".to_string(),
            ],
            spec,
            input_files: vec![],
        })
        .expect_err("cpu runaway must die");
    match err {
        // Signal death surfaces as -1 (raw) or 128+signum (prlimit's
        // convention) — never a clean exit code.
        SandboxError::WorkerFailed {
            exit_code, wall_ms, ..
        } => {
            assert!(
                exit_code == -1 || (128..256).contains(&exit_code),
                "killed by signal, not a normal exit: {exit_code}"
            );
            assert!(wall_ms < 10_000, "rlimit fired within {wall_ms} ms");
        }
        other => panic!("expected signal kill, got {other:?}"),
    }
}

#[test]
fn huge_writes_hit_the_fsize_bound() {
    let mut spec = base_spec();
    spec.fsize_bytes = 1024 * 1024;
    let out = provider().run(RunSpec {
        argv: vec![
            "python3".to_string(),
            "-c".to_string(),
            "f=open('big.bin','wb')\n\
             \x20try:\n\
             \x20 f.write(b'x'*100_000_000); print('WROTE-ALL')\n\
             \x20except OSError:\n\
             \x20 print('FSIZE-DENIED')"
                .to_string(),
        ],
        spec,
        input_files: vec![],
    });
    match out {
        Ok(o) => assert!(
            o.stdout.contains("FSIZE-DENIED"),
            "fsize not enforced: {:?}",
            o.stdout
        ),
        // Killed mid-write: still a BOUNDED death (review pass 1: the arm
        // asserted nothing before).
        Err(SandboxError::WorkerFailed { wall_ms, .. }) => {
            assert!(wall_ms < 10_000, "bounded write death in {wall_ms} ms");
        }
        Err(other) => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn dependency_installation_cannot_succeed_without_network() {
    // MASTER_SPEC:389 dependency-installation class: install must fail.
    let err = provider()
        .run(RunSpec {
            argv: vec![
                "python3".to_string(),
                "-m".to_string(),
                "pip".to_string(),
                "install".to_string(),
                "requests".to_string(),
            ],
            spec: base_spec(),
            input_files: vec![],
        })
        .expect_err("pip install must not succeed");
    assert!(
        matches!(err, SandboxError::WorkerFailed { .. }),
        "install path: {err:?}"
    );
}

#[test]
fn only_the_sandbox_python_environment_is_visible() {
    // Dependency class: host site-packages (this repo's .venv, home) cannot
    // be imported from or even seen on sys.path.
    let out = provider()
        .run(RunSpec {
            argv: vec![
                "python3".to_string(),
                "-c".to_string(),
                "import sys, json; print(json.dumps(sys.path))".to_string(),
            ],
            spec: base_spec(),
            input_files: vec![],
        })
        .expect("run");
    assert!(
        !out.stdout.contains(".venv"),
        "host venv visible: {}",
        out.stdout
    );
    assert!(
        !out.stdout.contains("ideator"),
        "home paths visible: {}",
        out.stdout
    );
    // No host cwd either: cwd is /work.
    let out = provider()
        .run(RunSpec {
            argv: vec![
                "python3".to_string(),
                "-c".to_string(),
                "import os; print(os.getcwd())".to_string(),
            ],
            spec: base_spec(),
            input_files: vec![],
        })
        .expect("run");
    assert!(out.stdout.contains("/work"), "cwd: {}", out.stdout);
}

// --- Review pass 1 remediation tests ---

#[test]
fn attestation_refuses_host_exposing_binds() {
    // mod.rs: "NEVER a whole-root bind" — now enforced, not just documented.
    let home = std::env::home_dir().expect("home");
    // (shape-checked per case:)
    for bad in [
        std::path::PathBuf::from("/"),
        std::path::PathBuf::from("/home"),
        home.clone(),
        std::path::PathBuf::from("/root/.ssh"),
        std::env::temp_dir(),
    ] {
        let mut spec = base_spec();
        spec.extra_ro_binds = vec![(bad.clone(), std::path::PathBuf::from("/x"))];
        let result = provider().run(RunSpec {
            argv: vec!["python3".to_string()],
            spec,
            input_files: vec![],
        });
        assert!(
            matches!(result, Err(SandboxError::InvalidSpec { .. })),
            "bind {:?} must be refused, got {result:?}",
            bad
        );
    }
}

#[test]
fn subprocess_children_stay_inside_the_network_namespace() {
    // MASTER_SPEC:389 subprocess class: a spawned child inherits the
    // isolation — it cannot egress either.
    let out = run(&[
        "python3",
        "-c",
        "import subprocess, sys\n\
         r = subprocess.run([sys.executable, '-c', 'import socket; socket.create_connection((\"1.1.1.1\",80),2)'], capture_output=True)\n\
         print('CHILD_EGRESS' if r.returncode == 0 else 'CHILD_NO_NET')",
    ])
    .expect("run");
    assert!(out.stdout.contains("CHILD_NO_NET"), "{:?}", out.stdout);
}

#[test]
fn dependency_install_failure_has_a_pinned_cause() {
    // Ticket 02 AC (amended): pin WHY install is impossible — either pip is
    // absent from the interpreter or (pip present) the network blocks it.
    let probe = provider().run(RunSpec {
        argv: ["python3", "-m", "pip", "--version"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        spec: base_spec(),
        input_files: vec![],
    });
    let pip_present = match &probe {
        Ok(_) => true,
        Err(SandboxError::WorkerFailed { stderr, .. }) => {
            assert!(
                stderr.contains("No module named pip") || stderr.contains("pip"),
                "probe failure must name pip: {stderr}"
            );
            false
        }
        other => panic!("unexpected probe result: {other:?}"),
    };
    let err = provider()
        .run(RunSpec {
            argv: vec![
                "python3".to_string(),
                "-m".to_string(),
                "pip".to_string(),
                "install".to_string(),
                // PEP 668 refuses before the network layer, and a
                // preinstalled dist (CI images ship several) would satisfy
                // the requirement without touching one; bypass the policy
                // check with a package that cannot be present so the only
                // remaining failure cause is blocked egress (ticket-02 AC).
                "--break-system-packages".to_string(),
                "hephaestus-nonexistent-pkg-7f3a".to_string(),
            ],
            spec: base_spec(),
            input_files: vec![],
        })
        .expect_err("install must fail");
    assert!(matches!(err, SandboxError::WorkerFailed { .. }));
    if pip_present {
        // pip exists inside => failure is the network (proven separately by
        // the egress tests).
        if let SandboxError::WorkerFailed { stderr, .. } = &err {
            let s = stderr.to_lowercase();
            assert!(
                s.contains("network")
                    || s.contains("resolution")
                    || s.contains("retr")
                    || s.contains("temporary failure")
                    || s.contains("unreachable")
                    || s.contains("connection"),
                "pip present; failure must be network-caused: {stderr}"
            );
        }
    } else {
        // pip absent => install impossible for the pinned reason above.
        if let SandboxError::WorkerFailed { stderr, .. } = &err {
            assert!(stderr.contains("No module named pip"), "{stderr}");
        }
    }
}

#[test]
fn path_qualified_declared_tools_normalize_to_basenames() {
    // Pass-2 test gap: a declared entry written as a path still matches a
    // path-form argv[0] by basename.
    let mut spec = base_spec();
    spec.declared_tools = vec!["/usr/bin/python3".to_string()];
    let out = provider()
        .run(RunSpec {
            argv: vec![
                "/usr/bin/python3".to_string(),
                "-c".to_string(),
                "print('QUAL')".to_string(),
            ],
            spec,
            input_files: vec![],
        })
        .expect("path-qualified declared tool must run");
    assert!(out.stdout.contains("QUAL"));
}
