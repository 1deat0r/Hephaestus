//! R-049/R-050 bounded task contracts + deterministic reducer (T-045).
//!
//! Integration tests at the public seam: `TaskSpec`,
//! `compile_validation`, `WorkerProposal`, `reduce`.

use hephaestus::taskplan::{TaskSpec, WorkerProposal, compile_validation, reduce};
use std::collections::BTreeMap;

fn valid_spec() -> TaskSpec {
    let mut writes = BTreeMap::new();
    writes.insert("out/report".to_string(), "object".to_string());
    TaskSpec {
        id: "task-1".to_string(),
        declared_tools: vec!["sandbox/exec".to_string()],
        input_refs: vec!["artifact:in-1".to_string()],
        write_contracts: writes,
        capability_grants: vec!["grant-sandbox".to_string()],
        budget_bound: 100,
        timeout_secs: 60,
        max_retries: 1,
        isolation_profile: "snapshot-isolated".to_string(),
    }
}

fn proposal(worker: &str, key: &str, payload: &[u8]) -> WorkerProposal {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(payload);
    WorkerProposal {
        worker_id: worker.to_string(),
        output_key: key.to_string(),
        payload_sha256: h.finalize().iter().map(|b| format!("{b:02x}")).collect(),
        payload: payload.to_vec(),
    }
}

#[test]
fn at049_undeclared_tool_and_missing_limits_rejected() {
    let mut spec = valid_spec();
    spec.budget_bound = 0; // no limits
    let err = compile_validation(&spec).unwrap_err();
    assert!(err.iter().any(|e| e.contains("LIMITS_MISSING")));

    let mut spec = valid_spec();
    spec.declared_tools.clear(); // undeclared tools
    let err = compile_validation(&spec).unwrap_err();
    assert!(err.iter().any(|e| e.contains("TOOLS_UNDECLARED")));

    let mut spec = valid_spec();
    spec.isolation_profile = "shared-mutable".to_string();
    let err = compile_validation(&spec).unwrap_err();
    assert!(err.iter().any(|e| e.contains("ISOLATION_PROFILE_INVALID")));
}

#[test]
fn valid_spec_passes_compile() {
    assert!(compile_validation(&valid_spec()).is_ok());
}

#[test]
fn at050_conflicting_same_key_rejected_identical_accepted() {
    let mut writes = BTreeMap::new();
    writes.insert("out/a".to_string(), "object".to_string());
    let contracts = writes;
    // Two workers, same key, different payloads -> incompatible.
    let err = reduce(
        &[
            proposal("w1", "out/a", b"alpha"),
            proposal("w2", "out/a", b"beta"),
        ],
        &contracts,
    )
    .unwrap_err();
    assert!(err.iter().any(|e| e.contains("INCOMPATIBLE_PROPOSALS")));
    // Identical payloads (duplicate workers) -> accepted once.
    let ok = reduce(
        &[
            proposal("w1", "out/a", b"same"),
            proposal("w2", "out/a", b"same"),
        ],
        &contracts,
    )
    .unwrap();
    assert_eq!(ok.assembled.get("out/a").unwrap(), b"same");
    assert_eq!(ok.accepted, vec!["w1", "w2"]);
    // Undeclared write is rejected.
    let err = reduce(&[proposal("w1", "out/undeclared", b"x")], &contracts).unwrap_err();
    assert!(err.iter().any(|e| e.contains("UNDECLARED_WRITE")));
}

#[test]
fn deterministic_assembly_twin_run() {
    let mut writes = BTreeMap::new();
    writes.insert("out/b".to_string(), "object".to_string());
    writes.insert("out/a".to_string(), "object".to_string());
    let contracts = writes;
    let build = || {
        let reduced = reduce(
            &[proposal("w2", "out/b", b"B"), proposal("w1", "out/a", b"A")],
            &contracts,
        )
        .unwrap();
        serde_json::to_string(&reduced).unwrap()
    };
    assert_eq!(build(), build());
    // Sorted-key order: "out/a" before "out/b" in the serialized map.
    let serialized = build();
    assert!(serialized.find("out/a").unwrap() < serialized.find("out/b").unwrap());
}
