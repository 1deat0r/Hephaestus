//! `SandboxExecutor`: bridges the sandbox provider to T-009's
//! [`TaskExecutor`] — the integrated
//! execution path T-011 will need.

use crate::scheduler::{Task, TaskExecutor, TaskOutcome};

use super::{IsolationSpec, RunSpec, SandboxError, SandboxProvider};

/// Runs scheduler tasks as sandboxed worker jobs. The `tool` is the job's
/// declared worker function; each task's `timeout_ms` becomes the wall
/// deadline for that run.
#[derive(Debug, Clone)]
pub struct SandboxExecutor {
    provider: SandboxProvider,
    /// Job tool (``module:function``) for every task.
    tool: String,
    /// The job-level declared list (runner enforces it too).
    declared: Vec<String>,
    /// Base isolation profile (binds/env/limits); wall timeout is per task.
    spec: IsolationSpec,
}

impl SandboxExecutor {
    /// Wrap a provider with the worker tooling. `spec.extra_ro_binds` must
    /// expose the Python package (e.g. repo `python/` → `/opt/...`) and
    /// `spec.env_allowlist` should set `PYTHONPATH` accordingly.
    pub fn new(
        provider: SandboxProvider,
        tool: String,
        declared: Vec<String>,
        spec: IsolationSpec,
    ) -> Self {
        SandboxExecutor {
            provider,
            tool,
            declared,
            spec,
        }
    }
}

impl TaskExecutor for SandboxExecutor {
    fn run(&self, task: &Task) -> TaskOutcome {
        let mut spec = self.spec.clone();
        spec.wall_timeout_ms = task.timeout_ms;
        let job = serde_json::json!({
            "tool": self.tool,
            "declared_tools": self.declared,
            "input": {"task_id": task.id, "inputs": task.inputs},
        });
        let request = RunSpec {
            argv: vec![
                "python3".to_string(),
                "-m".to_string(),
                "hephaestus_workers.runner".to_string(),
                "/work/job.json".to_string(),
            ],
            spec,
            input_files: vec![("job.json".to_string(), job.to_string().into_bytes())],
        };
        // Metering (wall_ms, attestation) stays at the provider layer; the
        // scheduler outcome is just the task verdict + cost.
        match self.provider.run(request) {
            Ok(_) => TaskOutcome::Succeeded {
                cost: task.cost.clone(),
            },
            Err(SandboxError::Timeout { .. }) => TaskOutcome::TimedOut,
            Err(e) => TaskOutcome::Failed {
                reason: format!("SANDBOX: {e}"),
            },
        }
    }
}
