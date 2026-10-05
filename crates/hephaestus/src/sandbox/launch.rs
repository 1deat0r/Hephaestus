//! Process launch mechanics: pure argv builders + the supervising spawn.
//!
//! The builders are pure (unit-tested without spawning); `supervise` owns
//! the only clock in this module (wall deadline) and the reader threads
//! that keep stdio from deadlocking a bounded child.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::{IsolationSpec, RunOutcome, SandboxError};

/// Locate a tool on `PATH`; a miss resolves to a sentinel path under
/// `/nonexistent` so attestation fails with a clear, testable reason.
pub(crate) fn find_tool(name: &str) -> PathBuf {
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    PathBuf::from(format!("/nonexistent/{name}"))
}

/// `prlimit` resource bounds for one run (positive values only — attest
/// validated them first). `nproc_effective` = host per-UID process count at
/// launch + `spec.nproc` (RLIMIT_NPROC is per-UID; see the module docs).
pub(crate) fn prlimit_args(spec: &IsolationSpec, nproc_effective: u64) -> Vec<String> {
    vec![
        format!("--as={}", spec.memory_bytes),
        format!("--cpu={}", spec.cpu_secs),
        format!("--nproc={}", nproc_effective),
        format!("--fsize={}", spec.fsize_bytes),
        "--".to_string(),
    ]
}

/// Current per-UID **thread** count via /proc — the kernel's RLIMIT_NPROC
/// check counts threads, not processes (this host: ~1500 threads for one
/// uid, which is why a naive process count still deadlocked bwrap).
/// Fallback 4096 if /proc is unreadable — still a bound, conservative floor.
pub(crate) fn host_process_count() -> u64 {
    let uid = current_uid();
    let mut threads = 0u64;
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return 4096;
    };
    for entry in entries.flatten() {
        if !entry
            .file_name()
            .to_string_lossy()
            .chars()
            .all(|c| c.is_ascii_digit())
        {
            continue;
        }
        let Ok(status) = std::fs::read_to_string(entry.path().join("status")) else {
            continue;
        };
        let uid_match = status
            .lines()
            .find(|l| l.starts_with("Uid:"))
            .and_then(|l| l.split_whitespace().nth(1))
            == Some(uid.as_str());
        if uid_match {
            threads += status
                .lines()
                .find(|l| l.starts_with("Threads:"))
                .and_then(|l| {
                    l.split_whitespace()
                        .nth(1)
                        .and_then(|n| n.parse::<u64>().ok())
                })
                .unwrap_or(1);
        }
    }
    threads
}

fn current_uid() -> String {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                .map(|l| l.split_whitespace().nth(1).unwrap_or("0").to_string())
        })
        .unwrap_or_else(|| "0".to_string())
}

/// The bwrap argv for one run: minimal binds, all namespaces (network off),
/// cleared environment, private workdir. NEVER a whole-root bind.
pub(crate) fn bwrap_args(spec: &IsolationSpec, staging: &Path, argv: &[String]) -> Vec<String> {
    let mut a: Vec<String> = Vec::new();
    a.push("--unshare-all".into());
    a.push("--die-with-parent".into());
    a.push("--new-session".into());
    // Base filesystem: proc, devices, a private /tmp (one-arg flags),
    // then read-only system trees (two-arg binds).
    for (flag, target) in [("--proc", "/proc"), ("--dev", "/dev"), ("--tmpfs", "/tmp")] {
        a.push(flag.into());
        a.push(target.into());
    }
    a.push("--ro-bind".into());
    a.push("/usr".into());
    a.push("/usr".into());
    for lib in ["/lib", "/lib64"] {
        if Path::new(lib).exists() {
            a.push("--ro-bind".into());
            a.push(lib.into());
            a.push(lib.into());
        }
    }
    if Path::new("/etc/ld.so.cache").exists() {
        a.push("--ro-bind".into());
        a.push("/etc/ld.so.cache".into());
        a.push("/etc/ld.so.cache".into());
    }
    for (host, sandbox) in &spec.extra_ro_binds {
        a.push("--ro-bind".into());
        a.push(host.to_string_lossy().into_owned());
        a.push(sandbox.to_string_lossy().into_owned());
    }
    // The one read-write bind: this run's private staging dir.
    a.push("--bind".into());
    a.push(staging.to_string_lossy().into_owned());
    a.push("/work".into());
    a.push("--chdir".into());
    a.push("/work".into());
    // Environment: cleared, then an explicit allowlist.
    a.push("--clearenv".into());
    a.push("--setenv".into());
    a.push("PATH".into());
    a.push("/usr/bin".into());
    a.push("--setenv".into());
    a.push("HOME".into());
    a.push("/work".into());
    a.push("--setenv".into());
    a.push("HEPHAESTUS_SANDBOX".into());
    a.push("1".into());
    for (key, value) in &spec.env_allowlist {
        a.push("--setenv".into());
        a.push(key.clone());
        a.push(value.clone());
    }
    a.push("--".into());
    a.extend(argv.iter().cloned());
    a
}

/// Spawn under prlimit+bwrap, drain stdio with caps, enforce the wall
/// deadline, and reap. Returns the raw outcome (attestation attached by
/// the caller).
pub(crate) fn supervise(
    bwrap: &Path,
    prlimit: &Path,
    spec: &IsolationSpec,
    argv: &[String],
    staging: &Path,
) -> Result<RunOutcome, SandboxError> {
    let nproc_effective = host_process_count().saturating_add(spec.nproc);
    let mut full: Vec<String> = prlimit_args(spec, nproc_effective);
    full.push(bwrap.to_string_lossy().into_owned());
    full.extend(bwrap_args(spec, staging, argv));

    let start = Instant::now();
    let mut child = Command::new(prlimit)
        .args(&full)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_clear()
        .spawn()
        .map_err(|e| SandboxError::Io {
            context: format!("spawn {} …", prlimit.display()),
            error: e,
        })?;
    // S3 wait4 handle: reap the child through libc to read ru_maxrss —
    // the kernel's exact peak for the whole lineage. try_wait must not
    // reap first: only libc::waitpid reaps from here on.
    let child_pid = child.id() as libc::pid_t;

    let stdout_cap = spec.max_stdout;
    let stderr_cap = spec.max_stderr;
    let stdout_thread = child
        .stdout
        .take()
        .map(|s| std::thread::spawn(move || drain_capped(s, stdout_cap)));
    let stderr_thread = child
        .stderr
        .take()
        .map(|s| std::thread::spawn(move || drain_capped(s, stderr_cap)));

    // S3 wait4 reap: WNOHANG polls reap the prlimit child and fill
    // rusage — the kernel's exact lineage peak (ru_maxrss). No sampler
    // thread: the kernel tracks the maximum itself. try_wait must not
    // reap first, so only wait4 reaps from here on.
    let deadline = Duration::from_millis(spec.wall_timeout_ms);
    let mut rusage: libc::rusage = unsafe { std::mem::zeroed() };
    let mut status_code: libc::c_int = 0;
    loop {
        let got = unsafe { libc::wait4(child_pid, &mut status_code, libc::WNOHANG, &mut rusage) };
        if got == child_pid {
            break;
        }
        if got < 0 {
            let e = std::io::Error::last_os_error();
            let _ = child.kill();
            let _ = child.wait();
            return Err(SandboxError::Io {
                context: "wait4".to_string(),
                error: e,
            });
        }
        if start.elapsed() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(SandboxError::Timeout {
                wall_ms: start.elapsed().as_millis(),
            });
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let wall_ms = start.elapsed().as_millis();
    // Raw status decode (POSIX): low 7 bits hold the fatal signal,
    // bit 7 flags a core dump, the next byte holds a clean exit code.
    let signaled = status_code & 0x7f != 0;
    let signal = if signaled {
        Some(status_code & 0x7f)
    } else {
        None
    };
    let exit_code = if signaled {
        -1
    } else {
        (status_code >> 8) & 0xff
    };
    let peak_rss_bytes = (rusage.ru_maxrss.max(0) as u64).saturating_mul(1024);

    let (stdout, stdout_truncated) = stdout_thread
        .map(|t| t.join().unwrap_or((Vec::new(), false)))
        .unwrap_or((Vec::new(), false));
    let (stderr, stderr_truncated) = stderr_thread
        .map(|t| t.join().unwrap_or((Vec::new(), false)))
        .unwrap_or((Vec::new(), false));
    let stdout = String::from_utf8_lossy(&stdout).into_owned();
    let stderr = String::from_utf8_lossy(&stderr).into_owned();

    // S3 receipts: RLIMIT_AS refuses the over-limit allocation before
    // RSS grows, so a limit-caused death arrives as a plain nonzero
    // exit with a low peak. The (peak, limit, stderr) triple is the
    // enforcement receipt; oom_killed names only peak-breach/signal.
    let oom_killed = peak_rss_bytes >= spec.memory_bytes || signal.is_some();
    if exit_code != 0 {
        return Err(SandboxError::WorkerFailed {
            exit_code,
            stdout,
            stderr,
            wall_ms,
            peak_rss_bytes,
            memory_limit_bytes: spec.memory_bytes,
            oom_killed,
        });
    }
    Ok(RunOutcome {
        stdout,
        stderr,
        stdout_truncated,
        stderr_truncated,
        exit_code,
        wall_ms,
        peak_rss_bytes,
        attestation: super::Attestation {
            bwrap_version: String::new(),
            prlimit_path: prlimit.to_path_buf(),
            network: "unshared-off".to_string(),
            argv_preview: Vec::new(),
            limits: Vec::new(),
            wall_timeout_ms: spec.wall_timeout_ms,
        },
        output_files: Vec::new(),
        outputs_truncated: false,
        work_dir: None,
    })
}

/// Best-effort child-pid scan of /proc. wait4 replaced this probe in
/// the supervise path; the unit tests below keep it as a regression
/// reader for live values and recycled pids.
#[cfg(test)]
fn read_rss_kb(pid: u32) -> u64 {
    lineage_pids(pid)
        .iter()
        .map(|p| {
            std::fs::read_to_string(format!("/proc/{p}/status"))
                .ok()
                .and_then(|s| {
                    s.lines()
                        .find(|l| l.starts_with("VmRSS:"))
                        .and_then(|l| l.split_whitespace().nth(1))
                        .and_then(|n| n.parse::<u64>().ok())
                })
                .unwrap_or(0)
        })
        .max()
        .unwrap_or(0)
}

/// The pid plus its live child pids, via /proc. Best effort: a child
/// that exits mid-scan simply drops out.
#[cfg(test)]
fn lineage_pids(pid: u32) -> Vec<u32> {
    let mut out = vec![pid];
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return out;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(child) = name.parse::<u32>() else {
            continue;
        };
        let ppid = std::fs::read_to_string(entry.path().join("status"))
            .ok()
            .and_then(|s| {
                s.lines().find(|l| l.starts_with("PPid:")).and_then(|l| {
                    l.split_whitespace()
                        .nth(1)
                        .and_then(|n| n.parse::<u32>().ok())
                })
            });
        if ppid == Some(pid) {
            out.push(child);
        }
    }
    out
}

/// Read at most `cap + 1` bytes (to detect truncation), then keep draining
/// into a scratch buffer so a chatty child can never deadlock the pipes;
/// memory stays bounded at `cap + 1`.
fn drain_capped<R: Read>(mut stream: R, cap: usize) -> (Vec<u8>, bool) {
    let mut buf = Vec::with_capacity(cap.min(4096) + 1);
    let mut limited = (&mut stream).take((cap + 1) as u64);
    let _ = limited.read_to_end(&mut buf);
    let truncated = buf.len() > cap;
    buf.truncate(cap);
    if !truncated {
        // EOF reached within the cap — nothing more to drain.
        // (take() stopped exactly at EOF.)
        return (buf, false);
    }
    // Discard the rest so the child can finish or be killed at the deadline.
    let mut scratch = [0u8; 4096];
    loop {
        match stream.read(&mut scratch) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
    }
    (buf, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> IsolationSpec {
        IsolationSpec {
            declared_tools: vec!["python3".to_string()],
            env_allowlist: vec![("EXTRA".to_string(), "1".to_string())],
            extra_ro_binds: vec![],
            wall_timeout_ms: 1000,
            memory_bytes: 1024,
            cpu_secs: 1,
            nproc: 4,
            fsize_bytes: 1024,
            network: crate::sandbox::NetworkPolicy::Off,
            max_stdout: 100,
            max_stderr: 100,
            max_output_files: 8,
            max_output_file_bytes: 4096,
            max_output_bytes: 16384,
            keep_workdir: false,
        }
    }

    #[test]
    fn rss_sampler_reads_a_live_process() {
        // S3 regression: the /proc reader returns a positive value for
        // this live test process.
        let pid = std::process::id();
        assert!(read_rss_kb(pid) > 0, "live process has resident bytes");
    }

    #[test]
    fn rss_sampler_reads_zero_for_a_dead_pid() {
        // S3 regression: a recycled pid reads zero, never an error.
        assert_eq!(read_rss_kb(u32::MAX), 0);
    }

    #[test]
    fn argv_builder_isolation_invariants() {
        let args = bwrap_args(&spec(), Path::new("/staging"), &["python3".into()]);
        let joined = args.join(" ");
        assert!(joined.contains("--unshare-all"));
        assert!(joined.contains("--clearenv"));
        assert!(joined.contains("--die-with-parent"));
        assert!(joined.contains("--new-session"));
        assert!(joined.contains("--bind /staging /work"));
        assert!(joined.contains("--ro-bind /usr /usr"));
        assert!(!joined.contains("--ro-bind / /"), "never a whole-root bind");
        assert!(!joined.contains("--share-net"), "network stays off");
        assert!(joined.contains("--setenv PATH /usr/bin"));
        assert!(joined.contains("--setenv HEPHAESTUS_SANDBOX 1"));
        assert!(joined.contains("--setenv EXTRA 1"), "allowlist applied");
        assert!(joined.ends_with("-- python3"), "argv after --");
        // Home and temp are never bind sources.
        assert!(
            !joined.contains(
                std::env::home_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .as_ref()
            )
        );
    }

    #[test]
    fn prlimit_args_carry_every_bound() {
        let args = prlimit_args(&spec(), 165);
        assert_eq!(
            args,
            vec![
                "--as=1024".to_string(),
                "--cpu=1".to_string(),
                "--nproc=165".to_string(),
                "--fsize=1024".to_string(),
                "--".to_string(),
            ]
        );
    }
}
