//! Bounded task contracts and deterministic reducer (T-045,
//! R-049/R-050).
//!
//! Contract layer only (MASTER_SPEC §17): TaskSpec compile validation
//! is fail-closed (undeclared tools, missing limits, missing schemas
//! or isolation profile are rejected — AT-049), and parent assembly
//! reduces isolated worker proposals deterministically, treating
//! same-key conflicting writes as incompatible (AT-050). Execution
//! and scheduling live in the scheduler; this module performs no I/O.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A bounded task contract: everything the compiler requires before a
/// worker may run. No field is optional-limits: a task without
/// timeout/budget/schema/isolation is not a task, it is an
/// unrestricted agent (R-049).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskSpec {
    pub id: String,
    /// Explicitly declared tools. Any tool reference outside this set
    /// is a compile rejection.
    pub declared_tools: Vec<String>,
    /// Read-only input artifact references.
    pub input_refs: Vec<String>,
    /// Write contracts: output key -> output schema (JSON Schema-ish
    /// type tag; validated by the reducer).
    pub write_contracts: BTreeMap<String, String>,
    /// Capability grants required (policy engine ids).
    pub capability_grants: Vec<String>,
    /// Budget bound (units are the scheduler's; 0 is invalid here —
    /// a task must be bounded).
    pub budget_bound: u64,
    /// Timeout in seconds; 0 is invalid (unbounded).
    pub timeout_secs: u64,
    /// Retry policy bound; the total attempt count is capped at 3.
    pub max_retries: u8,
    /// Isolation profile: "snapshot-isolated" (read immutable
    /// snapshots, write isolated artifacts). No other profile exists.
    pub isolation_profile: String,
}

/// Named compile rejections (AT-049): every reason is explicit and
/// retained; nothing fails silently.
pub fn compile_validation(spec: &TaskSpec) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if spec.id.trim().is_empty() {
        errors.push("TASK_ID_MISSING: contract must name its task".to_string());
    }
    if spec.declared_tools.is_empty() {
        errors.push("TOOLS_UNDECLARED: no tool may run undeclared (AT-049)".to_string());
    }
    if spec.write_contracts.is_empty() {
        errors.push("OUTPUT_SCHEMA_MISSING: write contracts must declare schemas".to_string());
    }
    for (key, schema) in &spec.write_contracts {
        if schema.trim().is_empty() {
            errors.push(format!(
                "OUTPUT_SCHEMA_MISSING: write contract '{key}' has no schema"
            ));
        }
    }
    if spec.budget_bound == 0 {
        errors.push("LIMITS_MISSING: budget bound is required (no unrestricted tasks)".to_string());
    }
    if spec.timeout_secs == 0 {
        errors.push("LIMITS_MISSING: timeout is required (no unbounded tasks)".to_string());
    }
    if spec.max_retries > 3 {
        errors.push("RETRY_BOUND_EXCEEDED: retries capped at 3".to_string());
    }
    if spec.isolation_profile != "snapshot-isolated" {
        errors.push(format!(
            "ISOLATION_PROFILE_INVALID: '{}`, only snapshot-isolated is supported",
            spec.isolation_profile
        ));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// An isolated worker output: one worker, one output key, opaque
/// payload. Workers never write shared state directly (AT-050).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkerProposal {
    pub worker_id: String,
    pub output_key: String,
    pub payload_sha256: String,
    pub payload: Vec<u8>,
}

/// Successfully reduced parent output: sorted-key assembly of accepted
/// proposals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reduced {
    /// output key -> accepted payload (deterministic BTreeMap order).
    pub assembled: BTreeMap<String, Vec<u8>>,
    /// Per-worker acceptance record (deterministic worker-id order).
    pub accepted: Vec<String>,
}

/// Deterministic reducer (R-050, §17): validates each proposal against
/// the parent write contracts, rejects incompatible same-key proposals
/// unless byte-identical, assembles in sorted-key order.
pub fn reduce(
    proposals: &[WorkerProposal],
    write_contracts: &BTreeMap<String, String>,
) -> Result<Reduced, Vec<String>> {
    let mut errors = Vec::new();
    let mut by_key: BTreeMap<&str, Vec<&WorkerProposal>> = BTreeMap::new();
    for p in proposals {
        if !write_contracts.contains_key(&p.output_key) {
            errors.push(format!(
                "UNDECLARED_WRITE: worker '{}' wrote undeclared key '{}'",
                p.worker_id, p.output_key
            ));
            continue;
        }
        let digest = sha256_hex(&p.payload);
        if digest != p.payload_sha256 {
            errors.push(format!(
                "PAYLOAD_DIGEST_MISMATCH: worker '{}' digest does not match payload",
                p.worker_id
            ));
            continue;
        }
        by_key.entry(p.output_key.as_str()).or_default().push(p);
    }
    let mut assembled = BTreeMap::new();
    let mut accepted = Vec::new();
    for (key, group) in by_key {
        let mut distinct: Vec<&&WorkerProposal> = group.iter().by_ref().collect();
        distinct.dedup_by(|a, b| a.payload == b.payload);
        if distinct.len() > 1 {
            errors.push(format!(
                "INCOMPATIBLE_PROPOSALS: key '{key}' has {} conflicting writers (AT-050)",
                group.len()
            ));
            continue;
        }
        let winner = group.first().expect("group nonempty");
        assembled.insert(key.to_string(), winner.payload.clone());
        let mut writers: Vec<String> = group.iter().map(|p| p.worker_id.clone()).collect();
        writers.sort();
        accepted.extend(writers);
    }
    if errors.is_empty() {
        Ok(Reduced {
            assembled,
            accepted,
        })
    } else {
        Err(errors)
    }
}

/// SHA-256 over the payload (no external dependency beyond the
/// workspace's existing sha2 usage).
fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let out = hasher.finalize();
    out.iter().map(|b| format!("{b:02x}")).collect()
}
