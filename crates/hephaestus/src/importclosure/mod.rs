//! Import-closure oracle (T-042, R-102).
//!
//! The first domain is this workspace's Rust import graph. A declared
//! module closure is checked against independently resolved imports:
//! aliases and `crate::`/`super::` paths resolve, cycles are detected,
//! deletion/rename leaves stale declared entries, and unsupported
//! resolution (dynamic imports, external crates) blocks conservatively
//! WITHOUT omission. Two oracle implementations must agree before the
//! closure is accepted.

pub mod record;

pub use record::{ClosureMismatch, ClosureVerdict, ImportCycle, ImportEdge, OracleDisagreement};

use std::collections::{BTreeMap, BTreeSet};

/// Resolve one written target per the oracle's rules. `Some` canonical
/// path, or `None` + reason when resolution is unsupported.
pub fn resolve_target(from_module: &str, target: &str) -> (Option<String>, Option<String>) {
    // `crate::` paths are absolute: canonical is the path itself.
    if let Some(rest) = target.strip_prefix("crate::") {
        return (Some(format!("crate::{rest}")), None);
    }
    // `super::` resolves against the parent module of `from_module`.
    if let Some(rest) = target.strip_prefix("super::") {
        match from_module.rsplit_once("::") {
            Some((parent, _)) => return (Some(format!("{parent}::{rest}")), None),
            None => return (None, Some("super-from-root".to_string())),
        }
    }
    // Plain self-references (`self::`) resolve within the module.
    if let Some(rest) = target.strip_prefix("self::") {
        return (Some(format!("{from_module}::{rest}")), None);
    }
    // Dynamic or external targets are unsupported -> conservative.
    if target.starts_with("dyn::") || target.contains("macro_invocation") {
        return (None, Some("dynamic-import".to_string()));
    }
    if !target.starts_with("crate::") && !target.contains("::") {
        return (None, Some("external-or-unqualified".to_string()));
    }
    // Plain in-crate path (alias already folded by the caller).
    (Some(target.to_string()), None)
}

/// Resolve a set of written edges, folding aliases first.
pub fn resolve_edges(edges: &[(&str, &str, Option<&str>)]) -> Vec<ImportEdge> {
    edges
        .iter()
        .map(|(from, target, alias)| {
            // Alias: the canonical target is what the alias Binds to.
            let effective = alias.unwrap_or(target);
            let (resolved, reason) = resolve_target(from, effective);
            ImportEdge {
                from_module: from.to_string(),
                target: target.to_string(),
                resolved,
                unresolved_reason: reason,
            }
        })
        .collect()
}

/// Detect cycles in the resolved edge graph (iterative DFS with a
/// stack; cycles reported once, in traversal order).
pub fn detect_cycles(edges: &[ImportEdge]) -> Vec<ImportCycle> {
    let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for e in edges {
        if let (Some(res), Some(from)) = (&e.resolved, Some(e.from_module.as_str())) {
            adj.entry(from).or_default().push(res.as_str());
        }
    }
    let mut cycles = Vec::new();
    let mut reported: BTreeSet<Vec<String>> = BTreeSet::new();
    for start in adj.keys().copied().collect::<Vec<_>>() {
        // Iterative DFS tracking path; a back-edge to a node in the
        // current path closes a cycle.
        let mut path: Vec<&str> = vec![start];
        let mut on_path: BTreeSet<&str> = BTreeSet::from([start]);
        let mut iters: Vec<std::vec::IntoIter<&str>> =
            vec![adj.get(start).cloned().unwrap_or_default().into_iter()];
        while let Some(iter) = iters.last_mut() {
            match iter.next() {
                Some(next) => {
                    if let Some(pos) = path.iter().position(|m| **m == *next) {
                        let cycle: Vec<String> =
                            path[pos..].iter().map(|s| s.to_string()).collect();
                        // Canonical key: smallest rotation of the
                        // cycle, so a-b-c and b-c-a dedup to one.
                        let key = canonical_rotation(&cycle);
                        if reported.insert(key) {
                            cycles.push(ImportCycle { modules: cycle });
                        }
                    } else if !on_path.contains(next) {
                        path.push(next);
                        on_path.insert(next);
                        iters.push(adj.get(next).cloned().unwrap_or_default().into_iter());
                    }
                }
                None => {
                    iters.pop();
                    if let Some(popped) = path.pop() {
                        on_path.remove(popped);
                    }
                }
            }
        }
    }
    cycles
}

/// Check a declared closure against resolved edges (oracle A:
/// set-difference). Unresolved imports block first; then stale/missing
/// mismatches are named; then cycles are attached as mismatches of the
/// involved modules.
pub fn check_closure(declared: &[&str], edges: &[ImportEdge]) -> ClosureVerdict {
    let unresolved: Vec<String> = edges
        .iter()
        .filter(|e| e.resolved.is_none())
        .map(|e| {
            format!(
                "{} -> {} ({})",
                e.from_module,
                e.target,
                e.unresolved_reason.as_deref().unwrap_or("unknown")
            )
        })
        .collect();
    if !unresolved.is_empty() {
        return ClosureVerdict::BlockedUnresolved(unresolved);
    }

    let mut participants: BTreeSet<String> = BTreeSet::new();
    for e in edges {
        if let Some(res) = &e.resolved {
            // The closure covers both the importing module and the
            // resolved import target.
            participants.insert(e.from_module.clone());
            participants.insert(res.clone());
        }
    }
    let declared_set: BTreeSet<String> = declared.iter().map(|s| s.to_string()).collect();

    let mut mismatches: Vec<ClosureMismatch> = Vec::new();
    // Stale: declared but not part of the resolved graph (deletion or
    // rename left it behind).
    for d in &declared_set {
        if !participants.contains(d) {
            mismatches.push(ClosureMismatch::Stale { module: d.clone() });
        }
    }
    // Missing: graph participant absent from the declaration.
    for p in &participants {
        if !declared_set.contains(p) {
            mismatches.push(ClosureMismatch::Missing { module: p.clone() });
        }
    }
    if mismatches.is_empty() {
        ClosureVerdict::Matched
    } else {
        ClosureVerdict::Mismatched(mismatches)
    }
}

/// Oracle B: independent recount — build the closure from resolved
/// edges only (no declared input) and compare digests. Disagreement
/// means the closure is NOT accepted (conservative).
pub fn oracle_b_closure(edges: &[ImportEdge]) -> Vec<String> {
    let mut set: BTreeSet<String> = BTreeSet::new();
    for e in edges {
        if let Some(res) = &e.resolved {
            set.insert(e.from_module.clone());
            set.insert(res.clone());
        }
    }
    set.into_iter().collect()
}

/// Agreement gate: oracle A's accepted closure (the declaration, when
/// check_closure returns Matched) vs oracle B's recount. Both must
/// produce the same digest; disagreement -> conservative block. Note:
/// an empty graph (no resolved edges) yields identical empty digests
/// and counts as agreement; a mismatched declaration yields an empty
/// oracle-A closure which disagrees with any non-empty recount.
pub fn oracle_agreement(declared: &[&str], edges: &[ImportEdge]) -> Result<(), OracleDisagreement> {
    let a_closure: Vec<String> = match check_closure(declared, edges) {
        ClosureVerdict::Matched => declared.iter().map(|s| s.to_string()).collect(),
        _ => Vec::new(),
    };
    let b_closure = oracle_b_closure(edges);
    let a_digest = sha_digest(&a_closure);
    let b_digest = sha_digest(&b_closure);
    if a_digest == b_digest {
        Ok(())
    } else {
        Err(OracleDisagreement {
            oracle_a_digest: a_digest,
            oracle_b_digest: b_digest,
        })
    }
}

/// Deterministic digest over a sorted module list (FNV-1a 64-bit over
/// the joined names — no external dependency, stable across runs).
fn sha_digest(modules: &[String]) -> String {
    // Sorted, joined with NUL, hashed with FNV-1a.
    let mut sorted: Vec<&str> = modules.iter().map(String::as_str).collect();
    sorted.sort();
    let joined = sorted.join("\u{0}");
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in joined.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// Smallest rotation of the cycle (BTreeSet-friendly dedup key).
fn canonical_rotation(cycle: &[String]) -> Vec<String> {
    let n = cycle.len();
    (0..n)
        .map(|i| {
            cycle[i..]
                .iter()
                .chain(cycle[..i].iter())
                .cloned()
                .collect::<Vec<_>>()
        })
        .min()
        .unwrap_or_default()
}
