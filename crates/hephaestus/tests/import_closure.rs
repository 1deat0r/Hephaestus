//! R-102/AT-102 import-closure oracle (T-042): cycles, aliases,
//! relative paths, stale names, and oracle disagreement block without
//! omission.
//!
//! Integration tests at the public seam: `resolve_edges`,
//! `detect_cycles`, `check_closure`, `oracle_agreement`.

use hephaestus::importclosure::{
    ClosureMismatch, ClosureVerdict, check_closure, detect_cycles, oracle_agreement, resolve_edges,
};

#[test]
fn aliases_and_crate_paths_resolve() {
    // `use x as y` folds: the alias binds to the real target.
    let edges = resolve_edges(&[
        ("crate::app", "crate::util::fmt", Some("crate::util::fmt")),
        ("crate::app::inner", "super::util", None),
    ]);
    assert_eq!(edges[0].resolved.as_deref(), Some("crate::util::fmt"));
    // super:: from crate::app::inner -> crate::app::util
    assert_eq!(edges[1].resolved.as_deref(), Some("crate::app::util"));
}

#[test]
fn cycles_detected_without_looping() {
    let edges = resolve_edges(&[
        ("crate::a", "crate::b", None),
        ("crate::b", "crate::a", None),
        ("crate::c", "crate::a", None),
    ]);
    let cycles = detect_cycles(&edges);
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].modules.len(), 2);
}

#[test]
fn stale_and_missing_named() {
    // Declared `crate::gone` no longer imported (deletion/rename).
    // Participants: crate::app (importer) + crate::live (target).
    let edges = resolve_edges(&[("crate::app", "crate::live", None)]);
    match check_closure(&["crate::gone", "crate::app", "crate::live"], &edges) {
        ClosureVerdict::Mismatched(m) => {
            assert!(m.contains(&ClosureMismatch::Stale {
                module: "crate::gone".to_string()
            }));
        }
        other => panic!("expected mismatch, got {other:?}"),
    }
    // Missing: a graph participant absent from the declaration.
    match check_closure(&["crate::live"], &edges) {
        ClosureVerdict::Mismatched(m) => {
            assert!(m.contains(&ClosureMismatch::Missing {
                module: "crate::app".to_string()
            }));
        }
        other => panic!("expected mismatch, got {other:?}"),
    }
    // Matching declaration (all participants) -> Matched.
    assert_eq!(
        check_closure(&["crate::app", "crate::live"], &edges),
        ClosureVerdict::Matched
    );
}

#[test]
fn unsupported_resolution_blocks_without_omission() {
    // Dynamic import: unresolved -> BLOCK, with the import named
    // (nothing silently dropped).
    let edges = resolve_edges(&[("crate::app", "dyn::plugin", None)]);
    match check_closure(&["crate::app"], &edges) {
        ClosureVerdict::BlockedUnresolved(names) => {
            assert!(names[0].contains("dyn::plugin"));
            assert!(names[0].contains("dynamic-import"));
        }
        other => panic!("expected block, got {other:?}"),
    }
}

#[test]
fn oracle_disagreement_conservative_block() {
    // Declared set disagrees with the resolved participant closure:
    // oracle A refuses (no Matched closure), agreement gate blocks.
    let edges = resolve_edges(&[("crate::app", "crate::live", None)]);
    let declared = ["crate::other"];
    assert!(oracle_agreement(&declared, &edges).is_err());
    // Agreement: declaration exactly covers all participants
    // (importer + target) -> Ok.
    let declared = ["crate::app", "crate::live"];
    assert!(oracle_agreement(&declared, &edges).is_ok());
}

#[test]
fn twin_run_byte_identical() {
    let edges = resolve_edges(&[
        ("crate::a", "crate::b", None),
        ("crate::b", "crate::a", None),
    ]);
    let a = check_closure(&["crate::a", "crate::b"], &edges);
    let b = check_closure(&["crate::a", "crate::b"], &edges);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
