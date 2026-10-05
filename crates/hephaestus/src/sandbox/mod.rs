//! The sandbox provider (T-010): OS isolation for Python workers.
//!
//! # Sandbox limitations (MASTER_SPEC:389 demands these be documented)
//!
//! - Depends on **unprivileged user namespaces** (kernel config and host
//!   policy may disable them — attestation then denies, never degrades).
//! - **No seccomp-bpf profile yet**: syscall filtering beyond namespace and
//!   rlimit bounds is future work.
//! - **No brokered egress**: network is all-or-nothing off
//!   ([`NetworkPolicy::Off`]); AT-110's "allowed brokered routes" arrive
//!   with a network broker, later.
//! - Host-kernel attack surface remains (namespaces are kernel objects).
//! - **Not a multi-tenant boundary**: one principal's code, not mutually
//!   distrusting tenants.
//! - The sandbox **root is an ephemeral tmpfs** (bwrap's default when `/`
//!   is not bound): writes to unbound paths succeed *inside* the namespace
//!   and vanish with it; host trees are reachable only through the explicit
//!   read-only binds, which the traversal test proves unwritable.
//! - rlimits are advisory at the edges (kernel OOM killer behavior is not
//!   fully controlled by `RLIMIT_AS`).
//! - **`RLIMIT_NPROC` is per-UID**: the host's own processes count toward
//!   it, so the provider applies `spec.nproc` as a payload allowance *above
//!   the current per-UID count at launch* (declared 64 means "64 more than
//!   the host already runs", not "64 total") — an absolute cap below the
//!   host count would stop bwrap itself from starting (found the hard way:
//!   ~1500 host *threads* + `--nproc=64` = EAGAIN on namespace creation —
//!   the kernel counts threads, so the provider sums `Threads:` per
//!   matching /proc entry).
//!
//! Static attestation ([`SandboxProvider::attest`]) verifies tool presence
//! and spec self-consistency before every dispatch (R-110); it cannot
//! observe a host mount namespace mutated after startup — effective
//! boundary probes are exercised by the deny-first test matrix.

pub mod executor;
pub mod launch;

pub use executor::SandboxExecutor;

use std::path::{Path, PathBuf};
use std::process::Stdio;

use launch::{bwrap_args, find_tool, prlimit_args};

/// Network posture. Only [`Off`](NetworkPolicy::Off) is supported in M1 —
/// any other value is refused fail-closed (brokered routes are later work).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetworkPolicy {
    /// New network namespace with no interfaces (no egress).
    #[default]
    Off,
    /// Reserved for the future broker — refused today.
    On,
}

/// Everything the provider enforces around one run (MASTER_SPEC:333's
/// isolation profile, timeout, resource limits, tool list, output caps).
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSpec {
    /// Allowed `argv[0]` basenames; nothing else may spawn.
    pub declared_tools: Vec<String>,
    /// Extra environment variables granted inside (PATH/HOME/sandbox marker
    /// are always set by the provider).
    pub env_allowlist: Vec<(String, String)>,
    /// Extra read-only binds: (host path, sandbox path).
    pub extra_ro_binds: Vec<(PathBuf, PathBuf)>,
    /// Wall-clock kill deadline in milliseconds (> 0).
    pub wall_timeout_ms: u64,
    /// `RLIMIT_AS` in bytes (> 0).
    pub memory_bytes: u64,
    /// `RLIMIT_CPU` seconds (> 0).
    pub cpu_secs: u64,
    /// `RLIMIT_NPROC` (> 0).
    pub nproc: u64,
    /// `RLIMIT_FSIZE` in bytes (> 0).
    pub fsize_bytes: u64,
    /// Network posture; must be [`NetworkPolicy::Off`].
    pub network: NetworkPolicy,
    /// Byte cap for captured stdout (> 0).
    pub max_stdout: usize,
    /// Byte cap for captured stderr (> 0).
    pub max_stderr: usize,
    /// Maximum collected output files (> 0).
    pub max_output_files: usize,
    /// Per-file byte cap for collected outputs (> 0).
    pub max_output_file_bytes: usize,
    /// Total collected output bytes across files (> 0).
    pub max_output_bytes: usize,
    /// Keep the staging workdir after the run (test diagnostics only).
    pub keep_workdir: bool,
}

/// One requested execution.
#[derive(Debug, Clone, PartialEq)]
pub struct RunSpec {
    /// argv vector — executed directly, never through a shell.
    pub argv: Vec<String>,
    /// Isolation profile for this run.
    pub spec: IsolationSpec,
    /// Files staged into `/work` before the spawn (basename only —
    /// traversal names are refused).
    pub input_files: Vec<(String, Vec<u8>)>,
}

/// A successful, bounded run.
#[derive(Debug, Clone, PartialEq)]
pub struct RunOutcome {
    /// Captured stdout (lossy UTF-8), byte-capped.
    pub stdout: String,
    /// Captured stderr (lossy UTF-8), byte-capped.
    pub stderr: String,
    /// Stdout hit its byte cap.
    pub stdout_truncated: bool,
    /// Stderr hit its byte cap.
    pub stderr_truncated: bool,
    /// Exit code (0 — success path).
    pub exit_code: i32,
    /// Supervised wall time.
    pub wall_ms: u128,
    /// Peak resident-set bytes. S3 measures this against the hard limit.
    pub peak_rss_bytes: u64,
    /// What attestation verified before the spawn.
    pub attestation: Attestation,
    /// Collected files from `/work/out` (symlinks refused before any of
    /// these are returned; caps may truncate — see `outputs_truncated`).
    pub output_files: Vec<(String, Vec<u8>)>,
    /// At least one output cap (count/per-file/total) truncated collection.
    pub outputs_truncated: bool,
    /// Staging directory (kept only when `keep_workdir` was set).
    pub work_dir: Option<PathBuf>,
}

/// Evidence gathered by [`SandboxProvider::attest`] before each dispatch.
#[derive(Debug, Clone, PartialEq)]
pub struct Attestation {
    /// `bwrap --version` output (trimmed).
    pub bwrap_version: String,
    /// Absolute prlimit tool used for resource bounds.
    pub prlimit_path: PathBuf,
    /// Effective network posture (always "unshared-off" in M1).
    pub network: String,
    /// Preview of the bwrap argv (post-prlimit) for audit logs — the
    /// staging path is symbolic and the effective nproc reflects the host
    /// thread count at attestation time; the launch re-computes both.
    pub argv_preview: Vec<String>,
    /// Enforced limits, as (name, value) pairs.
    pub limits: Vec<(String, u64)>,
    /// Wall timeout in milliseconds.
    pub wall_timeout_ms: u64,
}

/// Every refusal — typed, never a panic.
#[derive(Debug)]
pub enum SandboxError {
    /// `attest()` failed: missing tool or failed tool probe.
    AttestationFailed {
        /// Why dispatch was denied.
        reason: String,
    },
    /// The spec itself is invalid (non-positive limits, empty tools…).
    InvalidSpec {
        /// What is wrong.
        reason: String,
    },
    /// A requested capability that M1 does not support (network On).
    Unsupported {
        /// The unsupported request.
        what: String,
    },
    /// argv\[0\] is not on the declared-tools list — nothing spawned.
    UndeclaredTool {
        /// The refused tool basename.
        tool: String,
    },
    /// Wall-clock deadline hit; the process was killed and reaped.
    Timeout {
        /// Observed wall time before the kill.
        wall_ms: u128,
    },
    /// The payload exited nonzero (provider layer reports it as failure).
    WorkerFailed {
        /// Exit code (`-1` when killed by a signal — e.g. RLIMIT_CPU).
        exit_code: i32,
        /// Capped stdout.
        stdout: String,
        /// Capped stderr.
        stderr: String,
        /// Supervised wall time before exit (metering).
        wall_ms: u128,
        /// Measured peak resident-set bytes (wait4 ru_maxrss). S3 receipt.
        peak_rss_bytes: u64,
        /// Hard memory limit for the run. S3 receipt.
        memory_limit_bytes: u64,
        /// True when the peak reached the limit or the worker died by
        /// signal. RLIMIT_AS refuses over-limit allocations before RSS
        /// grows, so a limit-caused death can also arrive as a plain
        /// nonzero exit with a low peak; that receipt is the
        /// (peak, limit, stderr) triple, not this flag. S3.
        oom_killed: bool,
    },
    /// Output collection refused: a symlink (or other non-regular entry)
    /// in `/work/out` — nothing is copied (exfil class, MASTER_SPEC:389).
    OutputRefused {
        /// The offending entry name.
        name: String,
    },
    /// Filesystem/spawn failure with context.
    Io {
        /// What was being done.
        context: String,
        /// The underlying error.
        error: std::io::Error,
    },
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SandboxError::AttestationFailed { reason } => {
                write!(f, "attestation denied dispatch: {reason}")
            }
            SandboxError::InvalidSpec { reason } => write!(f, "invalid isolation spec: {reason}"),
            SandboxError::Unsupported { what } => write!(f, "unsupported in M1: {what}"),
            SandboxError::UndeclaredTool { tool } => {
                write!(f, "tool not declared: {tool}")
            }
            SandboxError::Timeout { wall_ms } => {
                write!(f, "wall timeout after {wall_ms} ms")
            }
            SandboxError::WorkerFailed {
                exit_code,
                stdout,
                stderr,
                wall_ms,
                peak_rss_bytes,
                memory_limit_bytes,
                oom_killed,
            } => write!(
                f,
                "worker exited {exit_code} after {wall_ms} ms; stdout={:?} stderr={:?}; peak_rss={peak_rss_bytes}B limit={memory_limit_bytes}B oom_killed={oom_killed}",
                truncate(stdout, 200),
                truncate(stderr, 200)
            ),
            SandboxError::OutputRefused { name } => {
                write!(f, "refused output entry (symlink/non-regular): {name}")
            }
            SandboxError::Io { context, error } => write!(f, "{context}: {error}"),
        }
    }
}

impl std::error::Error for SandboxError {}

fn truncate(s: &str, n: usize) -> &str {
    if s.len() <= n {
        return s;
    }
    // Char-boundary safe: never panic in Display on multibyte output
    // (review pass 1).
    let mut end = n.min(s.len());
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// The bwrap-backed sandbox provider.
#[derive(Debug, Clone)]
pub struct SandboxProvider {
    bwrap: PathBuf,
    prlimit: PathBuf,
}

impl Default for SandboxProvider {
    fn default() -> Self {
        Self::with_default_tools()
    }
}

impl SandboxProvider {
    /// Resolve `bwrap` and `prlimit` from `PATH` (a missing tool resolves
    /// to a sentinel path so that `attest` denies with a clear reason —
    /// tests never skip, they fail).
    pub fn with_default_tools() -> Self {
        SandboxProvider {
            bwrap: find_tool("bwrap"),
            prlimit: find_tool("prlimit"),
        }
    }

    /// Explicit tool paths (tests use this to deny deliberately).
    pub fn with_tools(bwrap: PathBuf, prlimit: PathBuf) -> Self {
        SandboxProvider { bwrap, prlimit }
    }

    /// The bwrap tool path this provider will attest to.
    pub fn bwrap_path(&self) -> &Path {
        &self.bwrap
    }

    /// Static attestation (R-110): spec self-consistency + tool presence.
    /// `run` calls this first and denies on `Err`.
    pub fn attest(&self, spec: &IsolationSpec) -> Result<Attestation, SandboxError> {
        if spec.network != NetworkPolicy::Off {
            return Err(SandboxError::Unsupported {
                what: format!("network policy {:?}", spec.network),
            });
        }
        if spec.declared_tools.is_empty() {
            return Err(SandboxError::InvalidSpec {
                reason: "declared_tools must not be empty".to_string(),
            });
        }
        for (name, value) in [
            ("wall_timeout_ms", spec.wall_timeout_ms),
            ("memory_bytes", spec.memory_bytes),
            ("cpu_secs", spec.cpu_secs),
            ("nproc", spec.nproc),
            ("fsize_bytes", spec.fsize_bytes),
            ("max_stdout", spec.max_stdout as u64),
            ("max_stderr", spec.max_stderr as u64),
            ("max_output_files", spec.max_output_files as u64),
            ("max_output_file_bytes", spec.max_output_file_bytes as u64),
            ("max_output_bytes", spec.max_output_bytes as u64),
        ] {
            if value == 0 {
                return Err(SandboxError::InvalidSpec {
                    reason: format!("{name} must be > 0"),
                });
            }
        }
        let home = std::env::home_dir().unwrap_or_default();
        for (host, _) in &spec.extra_ro_binds {
            // Never bind: the whole root, any home tree (own or other
            // users'), or temp — host secrets must stay unbound (review
            // pass 1: "/" and "/home" previously slipped through).
            let forbidden = host == Path::new("/")
                || host.starts_with(&home)
                || host.starts_with("/home")
                || host.starts_with("/root")
                || host.starts_with(std::env::temp_dir());
            if forbidden {
                return Err(SandboxError::InvalidSpec {
                    reason: format!(
                        "bind {} would expose host material — refused",
                        host.display()
                    ),
                });
            }
        }
        if !executable(&self.bwrap) {
            return Err(SandboxError::AttestationFailed {
                reason: format!("bwrap not executable at {}", self.bwrap.display()),
            });
        }
        if !executable(&self.prlimit) {
            return Err(SandboxError::AttestationFailed {
                reason: format!("prlimit not executable at {}", self.prlimit.display()),
            });
        }
        let probe = std::process::Command::new(&self.bwrap)
            .arg("--version")
            .stdin(Stdio::null())
            .output()
            .map_err(|e| SandboxError::AttestationFailed {
                reason: format!("bwrap --version failed: {e}"),
            })?;
        if !probe.status.success() {
            return Err(SandboxError::AttestationFailed {
                reason: "bwrap --version exited nonzero".to_string(),
            });
        }
        let bwrap_version = String::from_utf8_lossy(&probe.stdout).trim().to_string();
        let argv_preview = {
            // Preview only: exact staging path and final effective nproc are
            // computed at launch (doc'd on the field).
            let mut full = prlimit_args(
                spec,
                launch::host_process_count().saturating_add(spec.nproc),
            );
            full.push(self.bwrap.to_string_lossy().into_owned());
            full.extend(bwrap_args(
                spec,
                Path::new("/work-staging"),
                &spec.declared_tools,
            ));
            full
        };
        Ok(Attestation {
            bwrap_version,
            prlimit_path: self.prlimit.clone(),
            network: "unshared-off".to_string(),
            argv_preview,
            limits: vec![
                ("memory_bytes".to_string(), spec.memory_bytes),
                ("cpu_secs".to_string(), spec.cpu_secs),
                ("nproc".to_string(), spec.nproc),
                ("fsize_bytes".to_string(), spec.fsize_bytes),
            ],
            wall_timeout_ms: spec.wall_timeout_ms,
        })
    }

    /// Attest, then supervise one isolated run (fail-closed: no spawn on
    /// any refusal).
    pub fn run(&self, request: RunSpec) -> Result<RunOutcome, SandboxError> {
        let attestation = self.attest(&request.spec)?;
        let tool = request
            .argv
            .first()
            .map(|a| {
                Path::new(a)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| a.clone())
            })
            .ok_or_else(|| SandboxError::InvalidSpec {
                reason: "argv must not be empty".to_string(),
            })?;
        let declared_ok = request.spec.declared_tools.iter().any(|d| {
            Path::new(d)
                .file_name()
                .map(|n| n == tool.as_str())
                .unwrap_or(false)
        });
        if !declared_ok {
            return Err(SandboxError::UndeclaredTool { tool });
        }

        // Private staging dir: the only host path bound read-write.
        let staging = new_staging_dir()?;
        // Stage inputs (basename only — traversal names never touch disk).
        // A refusal here must not leak the staging dir (pass-2 fresh leak).
        for (name, bytes) in &request.input_files {
            let staged: Result<(), SandboxError> = (|| {
                let base = Path::new(name)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .filter(|n| !n.is_empty() && n.as_str() == name)
                    .ok_or_else(|| SandboxError::InvalidSpec {
                        reason: format!("input file name {name:?} is not a plain basename"),
                    })?;
                std::fs::write(staging.join(&base), bytes).map_err(|e| SandboxError::Io {
                    context: format!("stage input {base}"),
                    error: e,
                })?;
                Ok(())
            })();
            if let Err(e) = staged {
                if !request.spec.keep_workdir {
                    let _ = std::fs::remove_dir_all(&staging);
                }
                return Err(e);
            }
        }
        let outcome = launch::supervise(
            &self.bwrap,
            &self.prlimit,
            &request.spec,
            &request.argv,
            &staging,
        );
        let keep = request.spec.keep_workdir;
        match outcome {
            Ok(mut ok) => {
                ok.attestation = attestation;
                // Safe output collection BEFORE cleanup: symlinks refuse
                // the whole batch; caps truncate (never crash). A refusal
                // must still clean up (review pass 1: `?` leaked staging).
                match collect_outputs(&staging.join("out"), &request.spec) {
                    Err(e) => {
                        if !keep {
                            let _ = std::fs::remove_dir_all(&staging);
                        }
                        return Err(e);
                    }
                    Ok((files, truncated)) => {
                        ok.output_files = files;
                        ok.outputs_truncated = truncated;
                    }
                }
                if !keep {
                    let _ = std::fs::remove_dir_all(&staging);
                    ok.work_dir = None;
                } else {
                    ok.work_dir = Some(staging);
                }
                Ok(ok)
            }
            Err(e) => {
                if !keep {
                    let _ = std::fs::remove_dir_all(&staging);
                }
                Err(e)
            }
        }
    }
}

/// (collected files, truncation happened)
type OutputCollection = (Vec<(String, Vec<u8>)>, bool);

/// Collect `/work/out`: regular files only (symlink/dir → refuse the whole
/// batch), per-file/total/count caps applied with a truncation flag.
fn collect_outputs(out_dir: &Path, spec: &IsolationSpec) -> Result<OutputCollection, SandboxError> {
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    let mut truncated = false;
    let mut total = 0usize;
    let mut entries: Vec<_> = std::fs::read_dir(out_dir)
        .map_err(|e| SandboxError::Io {
            context: format!("read {}", out_dir.display()),
            error: e,
        })?
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let meta = std::fs::symlink_metadata(entry.path()).map_err(|e| SandboxError::Io {
            context: format!("stat {name}"),
            error: e,
        })?;
        if !meta.file_type().is_file() {
            return Err(SandboxError::OutputRefused { name });
        }
        if files.len() >= spec.max_output_files {
            truncated = true;
            continue;
        }
        let mut bytes = std::fs::read(entry.path()).map_err(|e| SandboxError::Io {
            context: format!("read {name}"),
            error: e,
        })?;
        if bytes.len() > spec.max_output_file_bytes {
            bytes.truncate(spec.max_output_file_bytes);
            truncated = true;
        }
        if total + bytes.len() > spec.max_output_bytes {
            let allowed = spec.max_output_bytes.saturating_sub(total);
            bytes.truncate(allowed);
            truncated = true;
        }
        total += bytes.len();
        files.push((name, bytes));
    }
    Ok((files, truncated))
}

fn executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

fn new_staging_dir() -> Result<PathBuf, SandboxError> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!(
        "t010-{}-{nanos}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(dir.join("out")).map_err(|e| SandboxError::Io {
        context: format!("create staging {}", dir.display()),
        error: e,
    })?;
    Ok(dir)
}
